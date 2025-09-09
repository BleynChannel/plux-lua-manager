//! Vtable handling for Lua plugins

use mlua::{Lua, MultiValue, Value};
use plux_rs::{Registry, function::FunctionOutput};

use crate::{
    error::ManagerError,
    lua::conversion::{lua_to_plux, plux_to_lua},
};

/// Register vtable functions.
pub fn register_vtable(lua: &Lua, vtable: &Registry<FunctionOutput>) -> Result<(), ManagerError> {
    let globals = lua.globals();

    for function in vtable.iter() {
        let function_name = function.name();
        let function = function.clone();
        let f = lua.create_function(move |ctx, lua_args: MultiValue| {
            let mut args = vec![];
            for arg in lua_args.iter().map(lua_to_plux) {
                args.push(arg?);
            }

            let output = function
                .call(&args)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                .map(|var| plux_to_lua(&var, ctx));

            match output {
                Some(out) => Ok(out?),
                None => Ok(Value::Nil),
            }
        })?;

        globals.set(function_name, f)?;
    }

    Ok(())
}
