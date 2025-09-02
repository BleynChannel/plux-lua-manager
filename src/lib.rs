#![warn(missing_docs)]
#![doc(html_logo_url = "https://your-logo-url.com/logo.png")]
#![doc(html_favicon_url = "https://your-logo-url.com/favicon.ico")]

//! # Plux Lua Manager
//!
//! A high-performance Lua plugin manager for Plux, providing a safe and efficient way to load, manage, and interact with Lua plugins.
//!
//! ## Quick Start
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
//!
//! For more examples, see the [examples](https://github.com/BleynChannel/August/tree/master/managers/plux-lua-manager/examples) directory.

mod config;
mod error;
mod lua;
mod manager;

pub use config::*;
pub use error::*;
pub use manager::*;

#[doc(hidden)]
pub mod prelude {
    pub use crate::error::*;
    pub use crate::manager::LuaManager;
}
