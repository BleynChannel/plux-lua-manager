//! API registration for Lua plugins

use std::sync::Arc;

use mlua::{Lua, MultiValue, Table, Value};
use plux_rs::{function::FunctionOutput, Api, StdInfo};
use semver::Version;

use crate::error::ManagerError;
use crate::lua::conversion::{lua_to_plux, plux_to_lua};

/// Registers the plugin API in the Lua environment
pub fn register_api(
    lua: &Lua,
    api: &Arc<Api<FunctionOutput, StdInfo>>,
) -> Result<(), ManagerError> {
    let globals = lua.globals();

    // Create the API table
    let api_table = lua.create_table()?;

    // Register the API functions
    register_call_function_depend(lua, api.clone(), &api_table)?;
    register_call_function_optional_depend(lua, api.clone(), &api_table)?;

    // Set the table in the global namespace
    globals.set("api", api_table)?;

    Ok(())
}

fn register_call_function_depend(
    lua: &Lua,
    api: Arc<Api<FunctionOutput, StdInfo>>,
    api_table: &Table,
) -> Result<(), ManagerError> {
    let f = lua.create_function(
        move |ctx, (id, version, name, args): (String, String, String, MultiValue)| {
            let version =
                Version::parse(&version).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

            let args = args
                .iter()
                .map(lua_to_plux)
                .collect::<Result<Vec<_>, _>>()?;

            let output = api
                .call_function_depend(&id, &version, &name, args.as_slice())
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                .map(|var| plux_to_lua(&var, ctx));

            match output {
                Some(out) => Ok(out?),
                None => Ok(Value::Nil),
            }
        },
    )?;
    api_table.set("call_function_depend", f)?;
    Ok(())
}

fn register_call_function_optional_depend(
    lua: &Lua,
    api: Arc<Api<FunctionOutput, StdInfo>>,
    api_table: &Table,
) -> Result<(), ManagerError> {
    let f = lua.create_function(
        move |ctx, (id, version, name, args): (String, String, String, MultiValue)| {
            let version =
                Version::parse(&version).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

            let args = args
                .iter()
                .map(lua_to_plux)
                .collect::<Result<Vec<_>, _>>()?;

            let output = api
                .call_function_optional_depend(&id, &version, &name, args.as_slice())
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

            match output {
                Some(out) => Ok({
                    let output = out
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                        .map(|var| plux_to_lua(&var, ctx));

                    match output {
                        Some(out) => (true, out?),
                        None => (true, Value::Nil),
                    }
                }),
                None => Ok((false, Value::Nil)),
            }
        },
    )?;

    api_table.set("call_function_optional_depend", f)?;
    Ok(())
}
