//! Example showing the plugin system with advanced features
//! 
//! Demonstrates:
//! - Plugin with lifecycle hooks
//! - Async extractors without block_in_place
//! - Multiple extractors composition
//! - Dependency management
//! - Priority-based execution

use firework::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

// Example custom plugin with state
#[derive(Clone)]
struct MetricsPlugin {
    request_count: Arc<RwLock<u64>>,
}

impl MetricsPlugin {
    fn new() -> Self {
        Self {
            request_count: Arc::new(RwLock::new(0)),
        }
    }
    
    async fn increment(&self) {
        *self.request_count.write().await += 1;
    }
    
    async fn get_count(&self) -> u64 {
        *self.request_count.read().await
    }
}

// Implement Plugin trait
#[async_trait::async_trait]
impl Plugin for MetricsPlugin {
    fn name(&self) -> &'static str {
        "Metrics"
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Metrics",
            version: "1.0.0",
            author: "Firework Team",
            description: "Request metrics tracking plugin",
        }
    }
    
    fn priority(&self) -> i32 {
        10 // Run very early
    }
    
    async fn on_init(&self) -> PluginResult<()> {
        println!("[Metrics] Initializing metrics plugin");
        Ok(())
    }
    
    async fn on_start(&self) -> PluginResult<()> {
        println!("[Metrics] Metrics plugin ready");
        Ok(())
    }
    
    async fn on_request(&self, _req: &mut Request) -> PluginResult<()> {
        self.increment().await;
        Ok(())
    }
    
    async fn on_shutdown(&self) -> PluginResult<()> {
        let count = self.get_count().await;
        println!("[Metrics] Total requests processed: {}", count);
        Ok(())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Custom extractor for metrics
struct Metrics(Arc<MetricsPlugin>);

#[async_trait::async_trait]
impl FromRequest for Metrics {
    async fn from_request(_req: &mut Request, _res: &mut Response) -> Result<Self> {
        let registry = plugin_registry().read().await;
        let plugin = registry.get::<MetricsPlugin>()
            .ok_or_else(|| Error::Internal("Metrics plugin not registered".into()))?;
        
        // Clone the plugin (it's Arc internally, so cheap)
        Ok(Metrics(Arc::new(plugin.clone())))
    }
}

// Routes
#[get("/")]
async fn index(_req: Request, _res: Response) -> Response {
    Response::new(StatusCode::Ok, b"Welcome to Firework!")
        .with_header("Content-Type", "text/plain")
}

#[get("/metrics")]
async fn get_metrics(_req: Request, _res: Response) -> Response {
    // Access plugin directly
    let registry = plugin_registry().read().await;
    if let Some(plugin) = registry.get::<MetricsPlugin>() {
        let count = plugin.get_count().await;
        let body = format!("{{\"requests\": {}}}", count);
        return Response::new(StatusCode::Ok, body.into_bytes())
            .with_header("Content-Type", "application/json");
    }
    
    Response::new(StatusCode::InternalServerError, b"Metrics not available")
}

#[get("/health")]
async fn health(_req: Request, _res: Response) -> Response {
    Response::new(StatusCode::Ok, b"OK")
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎆 Firework Plugin System Example");
    println!("====================================\n");
    
    // Register plugin
    let metrics = Arc::new(MetricsPlugin::new());
    register_plugin_async(metrics).await?;
    
    // Initialize all plugins
    let mut registry = plugin_registry().write().await;
    registry.validate_all().await?;
    registry.init_all().await?;
    registry.start_all().await?;
    drop(registry);
    
    println!("\n📊 Plugin Statistics:");
    let registry = plugin_registry().read().await;
    for metadata in registry.list_plugins() {
        println!("  - {} v{} by {}", metadata.name, metadata.version, metadata.author);
        if !metadata.description.is_empty() {
            println!("    {}", metadata.description);
        }
    }
    drop(registry);
    
    println!("\n🚀 Starting server on http://127.0.0.1:3000");
    println!("Try:");
    println!("  curl http://127.0.0.1:3000/");
    println!("  curl http://127.0.0.1:3000/metrics");
    println!("  curl http://127.0.0.1:3000/health");
    
    let server = routes![index, get_metrics, health];
    server.listen("127.0.0.1:3000").await?;
    
    // Shutdown plugins gracefully
    let registry = plugin_registry().read().await;
    registry.shutdown_all().await?;
    
    Ok(())
}
