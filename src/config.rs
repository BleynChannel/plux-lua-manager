//! Configuration handling for Lua plugins.
//!
//! This module provides types and functions for loading and validating
//! plugin configuration files. Each Lua plugin should include a `config.toml`
//! file in its root directory that specifies metadata and dependencies.
//!
//! # Example
//!
//! ```toml
//! # config.toml
//! name = "my_plugin"
//! description = "A sample plugin"
//! author = "Plugin Author"
//! version = "0.1.0"
//!
//! [dependencies]
//! other_plugin = "^1.0.0"
//!
//! [optional_dependencies]
//! optional_feature = "^2.0.0"
//! ```

use std::{collections::HashMap, path::PathBuf};

use plux::{Depend, StdInfo};
use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

/// Plugin configuration loaded from a `config.toml` file.
///
/// This struct represents the configuration for a Lua plugin, including
/// metadata like name, description, and version, as well as dependency
/// information.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The name of the plugin.
    ///
    /// This should be a unique identifier for the plugin, using only
    /// alphanumeric characters and underscores.
    pub name: String,

    /// A brief description of what the plugin does.
    pub description: String,

    /// The author of the plugin.
    pub author: String,

    /// An optional SPDX license identifier.
    pub license: Option<String>,

    /// Required dependencies for this plugin.
    ///
    /// Maps plugin names to version requirements using semantic versioning.
    /// Example: `other_plugin = "^1.0.0"`
    pub depends: Option<HashMap<String, VersionReq>>,

    /// Optional dependencies for this plugin.
    ///
    /// These dependencies are not required for the plugin to function,
    /// but may enable additional features if available.
    pub optional_depends: Option<HashMap<String, VersionReq>>,
}

/// Loads and validates a plugin's configuration.
///
/// This function reads the `config.toml` file from the specified plugin
/// directory, parses it into a `Config` struct, and converts it into
/// the standard plugin information format used by the plugin system.
///
/// # Arguments
///
/// * `plugin_path` - Path to the plugin directory containing `config.toml`
///
/// # Errors
///
/// Returns an error if:
/// - The config file is missing or unreadable
/// - The config file contains invalid TOML
/// - Required fields are missing or have invalid values
pub fn load_config(plugin_path: &PathBuf) -> Result<(Config, StdInfo), ConfigError> {
    let config_path = plugin_path.join("config.toml");
    if !config_path.exists() {
        return Err(ConfigError::NotFound);
    }

    let config_content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_content)?;

    let info = StdInfo {
        depends: config.depends.clone().map_or(vec![], |depends| {
            depends
                .into_iter()
                .map(|(id, version)| Depend::new(id, version))
                .collect()
        }),
        optional_depends: config.optional_depends.clone().map_or(vec![], |depends| {
            depends
                .into_iter()
                .map(|(id, version)| Depend::new(id, version))
                .collect()
        }),
    };

    Ok((config, info))
}
