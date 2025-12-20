//! Blinc configuration file handling

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Top-level Blinc configuration (blinc.toml)
#[derive(Debug, Deserialize, Serialize)]
pub struct BlincConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub dev: DevConfig,
    #[serde(default)]
    pub targets: TargetsConfig,
}

/// Project metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Build configuration
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BuildConfig {
    /// Entry point file (relative to project root)
    #[serde(default = "default_entry")]
    pub entry: String,
    /// Output directory
    #[serde(default = "default_output")]
    pub output: String,
    /// Additional source directories to include
    #[serde(default)]
    pub include: Vec<String>,
    /// Files/patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_entry() -> String {
    "src/main.blinc".to_string()
}

fn default_output() -> String {
    "target".to_string()
}

/// Development server configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct DevConfig {
    /// Hot-reload port
    #[serde(default = "default_port")]
    pub port: u16,
    /// Enable hot-reload
    #[serde(default = "default_true")]
    pub hot_reload: bool,
    /// Watch additional directories
    #[serde(default)]
    pub watch: Vec<String>,
}

fn default_port() -> u16 {
    3000
}

fn default_true() -> bool {
    true
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            hot_reload: true,
            watch: Vec::new(),
        }
    }
}

/// Target-specific configuration
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TargetsConfig {
    #[serde(default)]
    pub desktop: Option<DesktopConfig>,
    #[serde(default)]
    pub android: Option<AndroidConfig>,
    #[serde(default)]
    pub ios: Option<IosConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DesktopConfig {
    #[serde(default)]
    pub window_title: Option<String>,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub resizable: bool,
}

fn default_width() -> u32 {
    800
}

fn default_height() -> u32 {
    600
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AndroidConfig {
    pub package: String,
    #[serde(default = "default_min_sdk")]
    pub min_sdk: u32,
    #[serde(default = "default_target_sdk")]
    pub target_sdk: u32,
}

fn default_min_sdk() -> u32 {
    24
}

fn default_target_sdk() -> u32 {
    34
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IosConfig {
    pub bundle_id: String,
    #[serde(default = "default_ios_target")]
    pub deployment_target: String,
}

fn default_ios_target() -> String {
    "15.0".to_string()
}

impl BlincConfig {
    /// Load configuration from a directory (looks for blinc.toml)
    pub fn load_from_dir(path: &Path) -> Result<Self> {
        let config_path = if path.is_file() {
            path.to_path_buf()
        } else {
            path.join("blinc.toml")
        };

        if !config_path.exists() {
            anyhow::bail!(
                "No blinc.toml found in {}. Run `blinc init` to create one.",
                path.display()
            );
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        let config: BlincConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", config_path.display()))?;

        Ok(config)
    }

    /// Create a new configuration with the given project name
    pub fn new(name: &str) -> Self {
        Self {
            project: ProjectConfig {
                name: name.to_string(),
                version: default_version(),
                description: None,
                authors: Vec::new(),
            },
            build: BuildConfig::default(),
            dev: DevConfig::default(),
            targets: TargetsConfig::default(),
        }
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize config")
    }
}
