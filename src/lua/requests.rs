//! Request handling for Lua plugins

use std::sync::{Arc, Mutex};

use mlua::{Lua, MultiValue, Value};
use plux_rs::{
    Requests,
    function::{Arg, DynamicFunction, Request},
};

use super::conversion::{lua_to_plux, plux_to_lua};
use crate::error::{ManagerError, PluginError};

/// Registers functions that the plugin has requested
pub fn register_requests(
    lua: &Arc<Mutex<Lua>>,
    requests: &Requests,
) -> Result<Vec<DynamicFunction>, ManagerError> {
    requests.iter().try_fold(vec![], |mut registered, request| {
        let function = register_request(lua, request)?;
        registered.push(function);
        Ok(registered)
    })
}

/// Registers a single request
fn register_request(
    lua: &Arc<Mutex<Lua>>,
    request: &Request,
) -> Result<DynamicFunction, ManagerError> {
    let lua_function = lua
        .lock()
        .unwrap()
        .globals()
        .get(request.name.clone())
        .map(|value: Value| match value {
            Value::Function(f) => Ok(f.clone()),
            Value::Nil => Err(ManagerError::Plugin(PluginError::SourceError(format!(
                "Request `{}` does not exist",
                request.name
            )))),
            _ => Err(ManagerError::Plugin(PluginError::SourceError(format!(
                "`{}` should be a function",
                request.name
            )))),
        })
        .map_err(|e| ManagerError::Lua(e))
        .flatten()?;

    let lua_clone = lua.clone();

    let function = DynamicFunction::new(
        request.name.clone(),
        request
            .inputs
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                let name = format!("arg_{}", index);
                Arg::new(name, ty.clone())
            })
            .collect(),
        request
            .output
            .map(|output| Arg::new("output", output.clone())),
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

    Ok(function)
}
