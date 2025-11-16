# 🔧 Creating Custom Plugins

Build your own Firework plugins.

---

## Plugin Trait

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    
    async fn on_init(&self) -> PluginResult<()> { Ok(()) }
    async fn on_start(&self) -> PluginResult<()> { Ok(()) }
    async fn on_shutdown(&self) -> PluginResult<()> { Ok(()) }
    async fn on_reload(&self) -> PluginResult<()> { Ok(()) }
    
    async fn on_request(&self, req: &mut Request) -> PluginResult<()> { Ok(()) }
    async fn on_response(&self, req: &Request, res: &mut Response) -> PluginResult<()> { Ok(()) }
    
    fn as_any(&self) -> &dyn Any;
}
```

---

## Example: Analytics Plugin

```rust
use firework::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AnalyticsPlugin {
    stats: Arc<RwLock<Stats>>,
}

#[derive(Default)]
struct Stats {
    total_requests: u64,
    total_errors: u64,
}

impl AnalyticsPlugin {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(Stats::default())),
        }
    }
    
    pub async fn get_stats(&self) -> Stats {
        self.stats.read().await.clone()
    }
}

#[async_trait]
impl Plugin for AnalyticsPlugin {
    fn name(&self) -> &'static str {
        "Analytics"
    }
    
    async fn on_init(&self) -> PluginResult<()> {
        println!("[Analytics] Plugin initialized");
        Ok(())
    }
    
    async fn on_request(&self, _req: &mut Request) -> PluginResult<()> {
        self.stats.write().await.total_requests += 1;
        Ok(())
    }
    
    async fn on_response(&self, _req: &Request, res: &mut Response) -> PluginResult<()> {
        if res.status.code() >= 400 {
            self.stats.write().await.total_errors += 1;
        }
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

---

## Using the Plugin

```rust
#[tokio::main]
async fn main() {
    // Register
    let analytics = Arc::new(AnalyticsPlugin::new());
    register_plugin_async(analytics.clone()).await.unwrap();
    
    // Access in handlers
    #[get("/stats")]
    async fn stats(Extract(plugin): Extract<AnalyticsPlugin>) -> Json<Stats> {
        Json(plugin.get_stats().await)
    }
    
    routes!().listen("127.0.0.1:8080").await.unwrap();
}
```
