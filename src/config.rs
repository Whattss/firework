use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_workers")]
    pub workers: usize,
}

fn default_address() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_workers() -> usize {
    num_cpus::get()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: default_address(),
            port: default_port(),
            workers: default_workers(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    
    #[serde(default)]
    pub plugins: HashMap<String, toml::Value>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Try to load from default locations: Firework.toml, firework.toml
    pub fn load_default() -> Self {
        for path in &["Firework.toml", "firework.toml"] {
            if Path::new(path).exists() {
                match Self::from_file(path) {
                    Ok(config) => {
                        println!("[CONFIG] Loaded configuration from {}", path);
                        return config;
                    }
                    Err(e) => {
                        eprintln!("[CONFIG] Error loading {}: {}", path, e);
                    }
                }
            }
        }
        
        println!("[CONFIG] Using default configuration");
        Self::default()
    }
    
    /// Get plugin configuration
    pub fn plugin_config(&self, plugin_name: &str) -> Option<&toml::Value> {
        self.plugins.get(plugin_name)
    }
    
    /// Get plugin configuration as a specific type
    pub fn plugin_config_as<T: for<'de> Deserialize<'de>>(&self, plugin_name: &str) -> Option<T> {
        self.plugins.get(plugin_name).and_then(|value| {
            T::deserialize(value.clone()).ok()
        })
    }
    
    /// Get server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}

/// Global configuration registry
static CONFIG: std::sync::OnceLock<RwLock<Config>> = std::sync::OnceLock::new();

pub fn config() -> &'static RwLock<Config> {
    CONFIG.get_or_init(|| RwLock::new(Config::load_default()))
}

/// Initialize configuration from a specific file
pub async fn init_config<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::from_file(path)?;
    *config().write().await = cfg;
    Ok(())
}

/// Get a clone of the current configuration
pub async fn get_config() -> Config {
    config().read().await.clone()
}

/// Plugin configuration trait
#[async_trait::async_trait]
pub trait PluginConfig: Send + Sync {
    /// Get plugin configuration from global config
    async fn load_config(&self, plugin_name: &str) -> Option<toml::Value> {
        let cfg = config().read().await;
        cfg.plugin_config(plugin_name).cloned()
    }
    
    /// Get typed plugin configuration
    async fn load_config_as<T: for<'de> Deserialize<'de>>(&self, plugin_name: &str) -> Option<T> {
        let cfg = config().read().await;
        cfg.plugin_config_as(plugin_name)
    }
}

/// Default implementation for all types
impl<T: Send + Sync> PluginConfig for T {}

/// Helper functions for loading plugin config without needing an instance
pub async fn load_plugin_config(plugin_name: &str) -> Option<toml::Value> {
    let cfg = config().read().await;
    cfg.plugin_config(plugin_name).cloned()
}

pub async fn load_plugin_config_as<T: for<'de> Deserialize<'de>>(plugin_name: &str) -> Option<T> {
    let cfg = config().read().await;
    cfg.plugin_config_as(plugin_name)
}
