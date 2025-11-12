use crate::{Request, Response};
use std::any::Any;
use std::sync::Arc;

/// Plugin error type
#[derive(Debug)]
pub struct PluginError(pub String);

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plugin error: {}", self.0)
    }
}

impl std::error::Error for PluginError {}

/// Plugin result type
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub description: &'static str,
}

/// Plugin lifecycle hooks
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &'static str;
    
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name(),
            version: "1.0.0",
            author: "Unknown",
            description: "",
        }
    }
    
    /// Plugin priority (higher = runs first)
    fn priority(&self) -> i32 {
        0
    }
    
    /// Dependencies (plugin names that must be loaded first)
    fn depends_on(&self) -> Vec<&'static str> {
        Vec::new()
    }
    
    /// Called when plugin is registered
    async fn on_init(&self) -> PluginResult<()> {
        Ok(())
    }
    
    /// Called before server starts
    async fn on_start(&self) -> PluginResult<()> {
        Ok(())
    }
    
    /// Called on server shutdown
    async fn on_shutdown(&self) -> PluginResult<()> {
        Ok(())
    }
    
    /// Called on hot reload (if supported)
    async fn on_reload(&self) -> PluginResult<()> {
        Ok(())
    }
    
    /// Called before each request (optional middleware-like hook)
    async fn on_request(&self, _req: &mut Request) -> PluginResult<()> {
        Ok(())
    }
    
    /// Called after each request (optional middleware-like hook)
    async fn on_response(&self, _req: &Request, _res: &mut Response) -> PluginResult<()> {
        Ok(())
    }
    
    /// Called when a new connection is accepted (for stream-level plugins)
    async fn on_stream_accept(&self, _stream: &mut tokio::net::TcpStream) -> PluginResult<()> {
        Ok(())
    }
    
    /// Get plugin state (for accessing plugin-specific data)
    fn as_any(&self) -> &dyn Any;
}

/// Plugin registry
pub struct PluginRegistry {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    /// Register a plugin
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        println!("[PLUGIN] Registering: {}", plugin.name());
        self.plugins.push(plugin);
    }
    
    /// Sort plugins by priority and dependencies
    fn sort_plugins(&mut self) -> PluginResult<()> {
        // Simple priority-based sort for now
        self.plugins.sort_by(|a, b| b.priority().cmp(&a.priority()));
        Ok(())
    }
    
    /// Validate plugin dependencies
    pub async fn validate_all(&mut self) -> PluginResult<()> {
        let plugin_names: Vec<&'static str> = self.plugins.iter()
            .map(|p| p.name())
            .collect();
        
        for plugin in &self.plugins {
            for dep in plugin.depends_on() {
                if !plugin_names.contains(&dep) {
                    return Err(PluginError(format!(
                        "Plugin '{}' depends on '{}' which is not registered",
                        plugin.name(),
                        dep
                    )));
                }
            }
        }
        
        self.sort_plugins()?;
        Ok(())
    }
    
    /// Initialize all plugins
    pub async fn init_all(&self) -> PluginResult<()> {
        for plugin in &self.plugins {
            println!("[PLUGIN] Initializing: {}", plugin.name());
            plugin.on_init().await?;
        }
        Ok(())
    }
    
    /// Start all plugins
    pub async fn start_all(&self) -> PluginResult<()> {
        for plugin in &self.plugins {
            println!("[PLUGIN] Starting: {}", plugin.name());
            plugin.on_start().await?;
        }
        Ok(())
    }
    
    /// Shutdown all plugins
    pub async fn shutdown_all(&self) -> PluginResult<()> {
        for plugin in self.plugins.iter().rev() {
            println!("[PLUGIN] Shutting down: {}", plugin.name());
            plugin.on_shutdown().await?;
        }
        Ok(())
    }
    
    /// Reload all plugins
    pub async fn reload_all(&self) -> PluginResult<()> {
        for plugin in &self.plugins {
            println!("[PLUGIN] Reloading: {}", plugin.name());
            plugin.on_reload().await?;
        }
        Ok(())
    }
    
    /// Run on_request hooks
    pub async fn on_request(&self, req: &mut Request) -> PluginResult<()> {
        for plugin in &self.plugins {
            plugin.on_request(req).await?;
        }
        Ok(())
    }
    
    /// Run on_response hooks
    pub async fn on_response(&self, req: &Request, res: &mut Response) -> PluginResult<()> {
        for plugin in &self.plugins {
            plugin.on_response(req, res).await?;
        }
        Ok(())
    }
    
    /// Run on_stream_accept hooks
    pub async fn on_stream_accept(&self, stream: &mut tokio::net::TcpStream) -> PluginResult<()> {
        for plugin in &self.plugins {
            plugin.on_stream_accept(stream).await?;
        }
        Ok(())
    }
    
    /// Get plugin by type
    pub fn get<T: Plugin + 'static>(&self) -> Option<&T> {
        for plugin in &self.plugins {
            if let Some(p) = plugin.as_any().downcast_ref::<T>() {
                return Some(p);
            }
        }
        None
    }
    
    /// List all plugin metadata
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.iter()
            .map(|p| p.metadata())
            .collect()
    }
    
    /// Get all plugins
    pub fn plugins(&self) -> &[Arc<dyn Plugin>] {
        &self.plugins
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global plugin registry
static PLUGIN_REGISTRY: std::sync::OnceLock<tokio::sync::RwLock<PluginRegistry>> = std::sync::OnceLock::new();

pub fn registry() -> &'static tokio::sync::RwLock<PluginRegistry> {
    PLUGIN_REGISTRY.get_or_init(|| tokio::sync::RwLock::new(PluginRegistry::new()))
}

/// Register a plugin globally
pub fn register_plugin(plugin: Arc<dyn Plugin>) {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            registry().write().await.register(plugin);
        })
    });
}

/// Register a plugin asynchronously  
pub async fn register_plugin_async(plugin: Arc<dyn Plugin>) -> PluginResult<()> {
    registry().write().await.register(plugin);
    Ok(())
}

/// Get a plugin by type
pub async fn get_plugin<T: Plugin + 'static>() -> Option<Arc<dyn Plugin>> {
    let registry = registry().read().await;
    for plugin in registry.plugins() {
        if plugin.as_any().is::<T>() {
            return Some(plugin.clone());
        }
    }
    None
}

/// Auto-register all plugins from distributed slice
pub fn auto_register_plugins() {
    use crate::PLUGIN_FACTORIES;
    
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mut reg = registry().write().await;
            for factory in PLUGIN_FACTORIES {
                println!("[PLUGIN] Auto-registering: {}", factory.name);
                let plugin = (factory.create)();
                reg.register(plugin);
            }
        })
    });
}
