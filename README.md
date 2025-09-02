# Plux Lua Manager

[![Crates.io](https://img.shields.io/crates/v/plux-lua-manager)](https://crates.io/crates/plux-lua-manager)
[![Documentation](https://docs.rs/plux-lua-manager/badge.svg)](https://docs.rs/plux-lua-manager)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Lua plugin manager for Plux, providing a safe and efficient way to load, manage, and interact with Lua plugins.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
plux = { git = "https://github.com/BleynChannel/plux-rs.git", branch = "master" }
plux-lua-manager = { git = "https://github.com/BleynChannel/plux-lua-manager", branch = "master" }
```

## Features

The crate supports different Lua versions through feature flags:

- `lua54` (default): Use Lua 5.4
- `lua53`: Use Lua 5.3
- `lua52`: Use Lua 5.2
- `lua51`: Use Lua 5.1

## Quick Start

```rust
use plux::{function::Request, Loader};
use plux_lua_manager::prelude::*;

// Declaring a function for lua plugin
#[plux::function]
fn add(_: (), a: &i32, b: &i32) -> i32 {
    a + b
}

// Declaring a function for lua plugin
#[plux::function]
fn sub(_: (), a: &i32, b: &i32) -> i32 {
    a - b
}

fn main() -> anyhow::Result<()> {
    let mut loader = Loader::new();
    loader.context(move |mut ctx| {
        // Registering functions for lua plugin
        ctx.register_function(add());
        ctx.register_function(sub());

        // Registering a entrypoint for lua plugin
        ctx.register_request(Request::new("main".to_string(), vec![], None));
        
        // Registering the manager
        ctx.register_manager(LuaManager::new()).unwrap();
    });

    // Loading a plugin
    let plugin = loader
        .load_plugin_now(
            get_plugin_path("example", "0.1.0", "lua") // Plugin bundle is 'example-v0.1.0.lua'
                .to_str()
                .unwrap(),
        )
        .map(|bundle| loader.get_plugin_by_bundle(&bundle).unwrap())
        .unwrap();

    // Calling the 'main' request
    plugin.call_request("main", &[])
}
```

## Plugin Structure

A typical plugin has the following structure:

```
my_plugin/
├── config.toml   # Plugin configuration
├── main.lua      # Main plugin code
└── ...           # Additional Lua modules/resources
```

### `config.toml` Example

```toml
name = "my_plugin"
description = "A sample plugin"
author = "Your Name"
version = "0.1.0"

# Dependencies
[dependencies]
other_plugin = "^1.0.0"

# Optional dependencies
[optional_dependencies]
optional_feature = "^2.0.0"
```

### 'main.lua' Example

```lua
-- Declaring a function for Plux
function mul(a, b)
	return a * b;
end

function main()
	print("4 + 6 = " .. add(4, 6)); -- Use the function from Plux
	print("9 - 3 = " .. sub(9, 3)); -- Use the function from Plux
	print("8 * 3 = " .. mul(8, 3));
end

-- Returning the functions to Plux
return {
	{ name = "mul", inputs = {"a", "b"}, func = mul },
	{ name = "main", inputs = {}, func = main }
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
