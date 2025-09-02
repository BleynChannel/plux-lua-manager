//! Type conversion between Lua and Rust types

use mlua::{IntoLua, Lua, Value};
use plux::variable::Variable;

/// Converts a Lua value to a Rust Variable
pub fn lua_to_plux(lua_value: &Value) -> mlua::Result<Variable> {
    match lua_value {
        Value::Nil => Ok(Variable::Null),
        Value::Boolean(var) => Ok(Variable::Bool(*var)),
        Value::LightUserData(_) => Err(mlua::Error::RuntimeError(
            "Unsupported variable type".to_string(),
        )),
        Value::Integer(var) => Ok(Variable::I32(*var as i32)),
        Value::Number(var) => Ok(Variable::F32(*var as f32)),
        Value::String(var) => Ok(Variable::String(var.to_str()?.to_string())),
        Value::Table(var) => {
            let mut list = vec![];
            for pair in var.clone().pairs::<Value, Value>() {
                list.push(lua_to_plux(&pair?.1)?);
            }
            Ok(Variable::List(list))
        }
        Value::Function(_) => Err(mlua::Error::RuntimeError(
            "Unsupported variable type".to_string(),
        )),
        Value::Thread(_) => Err(mlua::Error::RuntimeError(
            "Unsupported variable type".to_string(),
        )),
        Value::UserData(_) => Err(mlua::Error::RuntimeError(
            "Unsupported variable type".to_string(),
        )),
        Value::Other(_) => Err(mlua::Error::RuntimeError(
            "Unsupported variable type".to_string(),
        )),
        Value::Error(err) => Err(*err.clone()),
    }
}

/// Converts a Rust Variable to a Lua value
pub fn plux_to_lua(variable: &Variable, lua: &Lua) -> mlua::Result<Value> {
    match variable {
        Variable::Null => Ok(Value::Nil),
        Variable::I8(var) => var.into_lua(lua),
        Variable::I16(var) => var.into_lua(lua),
        Variable::I32(var) => var.into_lua(lua),
        Variable::I64(var) => var.into_lua(lua),
        Variable::U8(var) => var.into_lua(lua),
        Variable::U16(var) => var.into_lua(lua),
        Variable::U32(var) => var.into_lua(lua),
        Variable::U64(var) => var.into_lua(lua),
        Variable::F32(var) => var.into_lua(lua),
        Variable::F64(var) => var.into_lua(lua),
        Variable::Bool(var) => var.into_lua(lua),
        Variable::Char(var) => var.to_string().into_lua(lua),
        Variable::String(var) => var.clone().into_lua(lua),
        Variable::List(var) => var
            .iter()
            .map(|v| plux_to_lua(v, lua))
            .collect::<mlua::Result<Vec<_>>>()?
            .into_lua(lua),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_conversion() {
        let lua = Lua::new();

        // Test null
        let var = Variable::Null;
        let lua_val = plux_to_lua(&var, &lua).unwrap();
        assert!(matches!(lua_val, Value::Nil));

        // Test boolean
        let var = Variable::Bool(true);
        let lua_val = plux_to_lua(&var, &lua).unwrap();
        assert_eq!(lua_val, Value::Boolean(true));

        // Test integer
        let var = Variable::I32(42);
        let lua_val = plux_to_lua(&var, &lua).unwrap();
        assert_eq!(lua_val, Value::Integer(42));

        // Test float
        let var = Variable::F32(3.14);
        let lua_val = plux_to_lua(&var, &lua).unwrap();
        assert_eq!(lua_val, Value::Number(3.14));

        // Test string
        let var = Variable::String("test".to_string());
        let lua_val = plux_to_lua(&var, &lua).unwrap();
        assert_eq!(lua_val, Value::String(lua.create_string("test").unwrap()));
    }

    #[test]
    fn test_complex_conversion() {
        let lua = Lua::new();

        // Test array
        let var = Variable::List(vec![
            Variable::F32(1.0),
            Variable::String("two".to_string()),
            Variable::Bool(true),
        ]);

        let lua_val = plux_to_lua(&var, &lua).unwrap();
        if let Value::Table(t) = lua_val {
            assert_eq!(t.get::<i64>(1).unwrap(), 1);
            assert_eq!(t.get::<String>(2).unwrap(), "two");
            assert_eq!(t.get::<bool>(3).unwrap(), true);
        } else {
            panic!("Expected table");
        }
    }
}
