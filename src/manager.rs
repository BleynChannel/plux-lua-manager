//! Core plugin manager implementation for Lua plugins.
//!
//! This module provides the main [`LuaManager`] type which implements the `Manager`
//! trait from the `plux` crate, enabling the loading, management, and execution
//! of Lua plugins in a safe and controlled environment.
//!
//! # Features
//!
//! - **Plugin Lifecycle Management**: Load, unload, and reload plugins at runtime
//! - **Sandboxed Execution**: Each plugin runs in an isolated environment
//! - **Dependency Injection**: Inject Rust functions and values into the Lua environment
//! - **Error Handling**: Comprehensive error handling and reporting
//! - **Thread Safety**: Safe concurrent execution of multiple plugins
//!
//! # Examples
//!
//! ```no_run
//! ...
//! 
//! use plux_lua_manager::prelude::*;
//! 
//! ...
//! 
//! loader.context(move |mut ctx| {
//!     ctx.register_manager(LuaManager::new()).unwrap();
//! });
//! 
//! ...
//! ```

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use hashbrown::HashMap;
use log;
use mlua::{Function, Lua, MultiValue, Table, Value};
use plux::{
    Api, Bundle, Manager, Plugin, StdInfo,
    context::LoadPluginContext,
    function::{Arg, DynamicFunction, FunctionOutput},
    utils::ManagerResult,
    variable::VariableType,
};

use crate::error::{ManagerError, PluginError};
use crate::{
    config::load_config,
    lua::{api, requests, vtable},
};

use crate::lua::conversion::{lua_to_plux, plux_to_lua};

/// The main manager type for Lua plugins.
///
/// This struct is responsible for managing the lifecycle of Lua plugins,
/// including loading, unloading, and interacting with them. It maintains
/// a mapping of plugin bundles to their respective Lua states.
///
/// # Thread Safety
///
/// `LuaManager` is `Send` and `Sync`, allowing it to be used safely across
/// thread boundaries. Each plugin's Lua state is protected by a mutex to
/// ensure thread safety.
pub struct LuaManager {
    /// Map of bundle identifiers to their Lua states
    lua_refs: HashMap<Bundle, Arc<Mutex<Lua>>>,
}

impl Default for LuaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaManager {
    /// Creates a new instance of `LuaManager`.
    ///
    /// # Examples
    ///
    /// ```
    /// use plux_lua_manager::prelude::LuaManager;
    ///
    /// let manager = LuaManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            lua_refs: HashMap::new(),
        }
    }

    /// Loads and executes the plugin's source code.
    fn load_src(
        &self,
        lua: &Arc<Mutex<Lua>>,
        api: Arc<Api<FunctionOutput, StdInfo>>,
        path: PathBuf,
    ) -> Result<(), ManagerError> {
        let main_path = path.join("main.lua");
        if !main_path.exists() {
            return Err(ManagerError::Plugin(PluginError::SourceError(
                "main.lua not found".to_string(),
            )));
        }

        let lua_guard = lua.lock().unwrap();

        // Set the package path to include the plugin's directory
        let package_path = format!(
            "{}/?.lua;{}/?/init.lua",
            path.to_string_lossy(),
            path.to_string_lossy()
        );

        lua_guard
            .load(&format!(
                r#"
                package.path = '{};' .. package.path
            "#,
                package_path
            ))
            .exec()?;

        // Execute the main script
        let src = std::fs::read_to_string(main_path)
            .map_err(|e| ManagerError::Plugin(PluginError::IoError(e)))?;
        let result: Vec<Table> = lua_guard.load(&src).eval()?;

        // Register the plugin functions
        let plugin = api.get_plugin_mut_by_bundle(api.plugin()).unwrap();
        for info in result.into_iter() {
            let name: String = info.get("name")?;
            let inputs: Vec<String> = info.get("inputs")?;
            let lua_function: Function = info.get("func")?;

            let lua_clone = lua.clone();
            let function = DynamicFunction::new(
                name.clone(),
                inputs
                    .iter()
                    .map(|name| Arg::new(name, VariableType::Let))
                    .collect(),
                Some(Arg::new("output", VariableType::Let)),
                move |args| {
                    let mut lua_args = vec![];
                    for arg in args {
                        lua_args.push(plux_to_lua(arg, &*lua_clone.lock().unwrap())?);
                    }

                    let result = match lua_function.call::<Value>(MultiValue::from_vec(lua_args))? {
                        Value::Nil => Ok(None),
                        value => Ok(Some(lua_to_plux(&value)?)),
                    };
                    result
                },
            );

            plugin
                .register_function(function)
                .map_err(|e| ManagerError::Plugin(PluginError::RegisterFunctionError(e)))?;
        }

        Ok(())
    }
}

impl<'a> Manager<'a, FunctionOutput, StdInfo> for LuaManager {
    /// Returns the format identifier for this manager ("lua").
    fn format(&self) -> &'static str {
        "lua"
    }

    /// Registers a new plugin.
    fn register_plugin(&mut self, context: plux::RegisterPluginContext) -> ManagerResult<StdInfo> {
        let (_, info) = load_config(&context.path).map_err(|e| ManagerError::Config(e))?;

        log::info!("Registering plugin: {}", context.bundle);
        Ok(info)
    }

    /// Unregisters a plugin.
    fn unregister_plugin(
        &mut self,
        plugin: &Plugin<'a, FunctionOutput, StdInfo>,
    ) -> ManagerResult<()> {
        let bundle = &plugin.info().bundle;
        log::info!("Unregistering plugin: {}", bundle);
        Ok(())
    }

    /// Loads a plugin into memory.
    fn load_plugin(
        &mut self,
        mut context: LoadPluginContext<'a, '_, FunctionOutput, StdInfo>,
        api: Api<FunctionOutput, StdInfo>,
    ) -> ManagerResult<()> {
        let bundle = context.plugin().info().bundle.clone();
        log::info!("Loading plugin: {}", bundle);

        let lua = Arc::new(Mutex::new(Lua::new()));
        let api = Arc::new(api);

        // Initialize the Lua environment
        {
            let lua_guard = lua.lock().unwrap();

            vtable::register_vtable(&lua_guard, api.registry())?;

            // Register the API
            api::register_api(&lua_guard, &api)?;
        }

        // Load the plugin's source code
        self.load_src(&lua, api.clone(), context.plugin().info().path.clone())?;

        // Register any requested functions
        let requests = requests::register_requests(&lua, context.requests())?;
        for request in requests {
            context.register_request(request)?;
        }

        // Store the Lua state
        self.lua_refs.insert(bundle, lua);

        Ok(())
    }

    /// Unloads a plugin from memory.
    fn unload_plugin(
        &mut self,
        plugin: &Plugin<'a, FunctionOutput, StdInfo>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bundle = &plugin.info().bundle;
        log::info!("Unloading plugin: {}", bundle);

        // Remove the Lua state
        self.lua_refs.remove(bundle);

        Ok(())
    }
}
