//! Error handling for the Plux Lua Manager.
//!
//! This module defines the error types used throughout the Plux Lua Manager.
//! It provides structured error handling for different components of the plugin system.
//!
//! # Error Types
//!
//! - [`ConfigError`]: Errors related to plugin configuration
//! - [`PluginError`]: Errors specific to plugin operations
//! - [`ManagerError`]: Top-level error type that can represent any error in the manager

use mlua::Error as LuaError;
use thiserror::Error;

/// Errors that can occur when working with plugin configuration.
///
/// This enum represents various error conditions that can occur when
/// loading or parsing plugin configuration files.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// The configuration file was not found in the expected location.
    #[error("Config file not found")]
    NotFound,

    /// The configuration file could not be parsed as valid TOML.
    #[error("Invalid config format: {0}")]
    InvalidFormat(#[from] toml::de::Error),

    /// An I/O error occurred while reading the configuration file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur during plugin operations.
///
/// This enum represents various error conditions that can occur when
/// loading, initializing, or executing plugins.
#[derive(Error, Debug)]
pub enum PluginError {
    /// The plugin source code could not be loaded or compiled.
    #[error("Source error: {0}")]
    SourceError(String),

    /// An I/O error occurred while working with plugin files.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// An error occurred while registering plugin functions.
    #[error("Plugin register function error: {0}")]
    RegisterFunctionError(#[from] plux::utils::PluginRegisterFunctionError),
}

/// The top-level error type for the Lua manager.
///
/// This enum represents all possible errors that can occur when working
/// with the Lua manager. It can be converted from more specific error types.
#[derive(Error, Debug)]
pub enum ManagerError {
    /// An error originating from the Lua runtime.
    #[error("Lua error: {0}")]
    Lua(#[from] LuaError),

    /// An error related to plugin configuration.
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    /// An error related to plugin operations.
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),
}