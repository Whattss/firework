# 🔌 Plugin System

Extend Firework with powerful plugins.

---

## Creating a Plugin

```rust
use firework::prelude::*;

#[derive(Clone)]
pub struct MyPlugin {
    config: String,
}

#[async_trait::async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &'static str {
        "MyPlugin"
    }
    
    async fn on_init(&self) -> PluginResult<()> {
        println!("[MyPlugin] Initializing...");
        Ok(())
    }
    
    async fn on_start(&self) -> PluginResult<()> {
        println!("[MyPlugin] Started!");
        Ok(())
    }
    
    async fn on_shutdown(&self) -> PluginResult<()> {
        println!("[MyPlugin] Shutting down...");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

---

## Registering Plugins

```rust
#[tokio::main]
async fn main() {
    // Register plugin
    register_plugin_async(Arc::new(MyPlugin {
        config: "example".into(),
    })).await.unwrap();
    
    // Start server
    routes!().listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Plugin Hooks

```rust
// Called once during initialization
async fn on_init(&self) -> PluginResult<()> { Ok(()) }

// Called before server starts
async fn on_start(&self) -> PluginResult<()> { Ok(()) }

// Called on graceful shutdown
async fn on_shutdown(&self) -> PluginResult<()> { Ok(()) }

// Called on hot reload
async fn on_reload(&self) -> PluginResult<()> { Ok(()) }

// Called before each request
async fn on_request(&self, req: &mut Request) -> PluginResult<()> { Ok(()) }

// Called after each request
async fn on_response(&self, req: &Request, res: &mut Response) -> PluginResult<()> { Ok(()) }
```
