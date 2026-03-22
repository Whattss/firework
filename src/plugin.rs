use crate::{Request, Response};
use std::any::Any;
use std::collections::{HashMap, HashSet};
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
    /// Return Some(Response) to short-circuit routing and return immediately
    async fn on_request(&self, _req: &mut Request, _res: &mut Response) -> PluginResult<Option<Response>> {
        Ok(None)
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
        let mut name_to_idx: HashMap<&'static str, usize> = HashMap::new();
        for (idx, plugin) in self.plugins.iter().enumerate() {
            if name_to_idx.insert(plugin.name(), idx).is_some() {
                return Err(PluginError(format!(
                    "Duplicate plugin name '{}' detected",
                    plugin.name()
                )));
            }
        }

        let plugin_count = self.plugins.len();
        let mut indegree = vec![0usize; plugin_count];
        let mut outgoing = vec![Vec::<usize>::new(); plugin_count];

        for (idx, plugin) in self.plugins.iter().enumerate() {
            for dep in plugin.depends_on() {
                let Some(&dep_idx) = name_to_idx.get(dep) else {
                    return Err(PluginError(format!(
                        "Plugin '{}' depends on '{}' which is not registered",
                        plugin.name(),
                        dep
                    )));
                };
                indegree[idx] += 1;
                outgoing[dep_idx].push(idx);
            }
        }

        let mut ready: Vec<usize> = (0..plugin_count).filter(|&i| indegree[i] == 0).collect();
        let mut ordered = Vec::with_capacity(plugin_count);

        while !ready.is_empty() {
            ready.sort_by(|a, b| {
                self.plugins[*b]
                    .priority()
                    .cmp(&self.plugins[*a].priority())
                    .then_with(|| self.plugins[*a].name().cmp(self.plugins[*b].name()))
            });

            let next = ready.remove(0);
            ordered.push(next);

            for &dependent in &outgoing[next] {
                indegree[dependent] = indegree[dependent].saturating_sub(1);
                if indegree[dependent] == 0 {
                    ready.push(dependent);
                }
            }
        }

        if ordered.len() != plugin_count {
            let blocked: Vec<&str> = (0..plugin_count)
                .filter(|&i| indegree[i] > 0)
                .map(|i| self.plugins[i].name())
                .collect();
            return Err(PluginError(format!(
                "Circular plugin dependency detected involving: {}",
                blocked.join(", ")
            )));
        }

        let mut sorted = Vec::with_capacity(plugin_count);
        for idx in ordered {
            sorted.push(Arc::clone(&self.plugins[idx]));
        }
        self.plugins = sorted;
        Ok(())
    }
    
    /// Validate plugin dependencies
    pub async fn validate_all(&mut self) -> PluginResult<()> {
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
            return Some(Arc::clone(plugin));  // Arc::clone is cheap (just increments refcount)
        }
    }
    None
}

/// Auto-register all plugins from distributed slice
pub fn auto_register_plugins() -> PluginResult<()> {
    use crate::PLUGIN_FACTORIES;
    
    tokio::task::block_in_place(|| -> PluginResult<()> {
        tokio::runtime::Handle::current().block_on(async {
            let mut reg = registry().write().await;
            let mut existing: HashSet<String> = reg
                .plugins
                .iter()
                .map(|p| p.name().to_string())
                .collect();
            for factory in PLUGIN_FACTORIES {
                if !existing.insert(factory.name.to_string()) {
                    continue;
                }
                println!("[PLUGIN] Auto-registering: {}", factory.name);
                let plugin = (factory.create)();
                reg.register(plugin);
            }
            reg.validate_all().await?;
            Ok(())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: &'static str,
        deps: Vec<&'static str>,
        priority: i32,
    }

    #[async_trait::async_trait]
    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            self.name
        }

        fn priority(&self) -> i32 {
            self.priority
        }

        fn depends_on(&self) -> Vec<&'static str> {
            self.deps.clone()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn validate_detects_missing_dependencies() {
        let mut registry = PluginRegistry::new();
        registry.register(Arc::new(TestPlugin {
            name: "auth",
            deps: vec!["db"],
            priority: 10,
        }));

        let err = registry.validate_all().await.expect_err("should fail");
        assert!(err.0.contains("depends on 'db'"));
    }

    #[tokio::test]
    async fn validate_detects_cycles() {
        let mut registry = PluginRegistry::new();
        registry.register(Arc::new(TestPlugin {
            name: "a",
            deps: vec!["b"],
            priority: 10,
        }));
        registry.register(Arc::new(TestPlugin {
            name: "b",
            deps: vec!["a"],
            priority: 10,
        }));

        let err = registry.validate_all().await.expect_err("should fail");
        assert!(err.0.contains("Circular plugin dependency"));
    }

    #[tokio::test]
    async fn validate_applies_topological_order_with_priority_tiebreak() {
        let mut registry = PluginRegistry::new();
        registry.register(Arc::new(TestPlugin {
            name: "db",
            deps: vec![],
            priority: 10,
        }));
        registry.register(Arc::new(TestPlugin {
            name: "auth",
            deps: vec!["db"],
            priority: 100,
        }));
        registry.register(Arc::new(TestPlugin {
            name: "metrics",
            deps: vec![],
            priority: 50,
        }));

        registry.validate_all().await.expect("must pass");
        let names: Vec<&str> = registry.plugins().iter().map(|p| p.name()).collect();

        let db_pos = names.iter().position(|n| *n == "db").unwrap();
        let auth_pos = names.iter().position(|n| *n == "auth").unwrap();
        assert!(db_pos < auth_pos, "db must be initialized before auth");
    }
}
