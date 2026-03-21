# Hot Reload

Firework's hot reload feature enables instant code updates during development without restarting the server or losing state.

## Overview

Hot reload watches your source files and automatically recompiles and reloads your application when changes are detected, maintaining:
- Active connections
- Application state
- WebSocket connections (optional)
- In-memory data (with state preservation)

## Features

- **Instant Feedback**: See changes in ~1-2 seconds
- **State Preservation**: Keep application state between reloads
- **Zero Configuration**: Works out of the box
- **Smart Watching**: Only rebuilds what changed
- **Connection Preservation**: Active HTTP connections continue
- **WebSocket Support**: Optional WebSocket reconnection

## Quick Start

### Using the CLI (Recommended)

```bash
# Start with hot reload
fwk dev

# Or explicitly
fwk run dev --hot-reload
```

### Programmatic Usage

```rust
use firework::prelude::*;

#[tokio::main]
async fn main() {
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

Then run with the `firework-dev` binary:
```bash
cargo run --bin firework-dev --features hot-reload
```

## Configuration

### Firework.toml

```toml
[hot_reload]
enabled = true
watch_paths = ["src", "Cargo.toml"]
ignore_paths = ["target", ".git"]
debounce_ms = 500
```

### Programmatic Configuration

```rust
use firework::hot_reload::HotReload;

#[tokio::main]
async fn main() {
    let hot_reload = HotReload::new()
        .watch_path("src")
        .watch_path("Cargo.toml")
        .ignore_path("target")
        .debounce_ms(500);
    
    routes!()
        .with_hot_reload(hot_reload)
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

## State Preservation

### Using HotReloadState

Preserve application state between reloads:

```rust
use firework::prelude::*;
use firework::hot_reload_state::HotReloadState;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
struct AppState {
    counter: u32,
    users: Vec<String>,
}

#[tokio::main]
async fn main() {
    // Initialize state that persists across reloads
    let state = HotReloadState::new(AppState::default());
    
    routes!()
        .state(state)
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}

#[get("/counter")]
async fn get_counter(state: HotReloadState<AppState>) -> Response {
    let count = state.read().await.counter;
    json!({"counter": count})
}

#[post("/counter/increment")]
async fn increment_counter(state: HotReloadState<AppState>) -> Response {
    let mut app_state = state.write().await;
    app_state.counter += 1;
    json!({"counter": app_state.counter})
}
```

### State Lifecycle

```rust
use firework::hot_reload_state::HotReloadState;

#[derive(Default, Clone)]
struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    fn load_from_disk() -> Self {
        // Load persistent state
        Cache { data: HashMap::new() }
    }
    
    fn save_to_disk(&self) {
        // Save state before reload
    }
}

#[tokio::main]
async fn main() {
    let cache = HotReloadState::new(Cache::load_from_disk());
    
    // Save state on shutdown/reload
    let cache_clone = cache.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        cache_clone.read().await.save_to_disk();
    });
    
    routes!()
        .state(cache)
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

## How It Works

### File Watching

Hot reload uses the `notify` crate to watch for filesystem changes:

```rust
// Watches these by default:
- src/**/*.rs
- Cargo.toml
- Firework.toml

// Ignores:
- target/
- .git/
- *.swp, *~
```

### Reload Process

1. **Detect Change**: File watcher detects modification
2. **Debounce**: Wait for 500ms (configurable) for more changes
3. **Rebuild**: Run `cargo build` in background
4. **Graceful Shutdown**: Finish pending requests
5. **State Transfer**: Preserve marked state
6. **Reload**: Start new version with preserved state
7. **Reconnect**: Restore connections (if enabled)

### Performance

- **First Build**: ~3-5 seconds (full compile)
- **Incremental Builds**: ~1-2 seconds (only changed files)
- **State Transfer**: < 10ms
- **Zero Downtime**: Pending requests complete normally

## Advanced Usage

### Custom Reload Hooks

```rust
use firework::prelude::*;

#[on_reload]
async fn before_reload() {
    println!("Saving state before reload...");
    // Flush caches, save data, etc.
}

#[on_reload]
async fn after_reload() {
    println!("Reload complete!");
    // Warm up caches, reconnect services, etc.
}
```

### Conditional Hot Reload

```rust
#[tokio::main]
async fn main() {
    let server = routes!();
    
    if cfg!(debug_assertions) {
        // Development: with hot reload
        server.with_hot_reload().listen("127.0.0.1:8080").await.unwrap();
    } else {
        // Production: no hot reload
        server.listen("0.0.0.0:8080").await.unwrap();
    }
}
```

### Watch Additional Files

```rust
use firework::hot_reload::HotReload;

#[tokio::main]
async fn main() {
    let hot_reload = HotReload::new()
        .watch_path("src")
        .watch_path("templates")
        .watch_path("config.toml")
        .on_change(|path| {
            println!("File changed: {:?}", path);
        });
    
    routes!()
        .with_hot_reload(hot_reload)
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

## WebSocket Integration

### Automatic Reconnection

```rust
use firework::prelude::*;

#[ws("/ws")]
async fn websocket_handler(mut ws: WebSocket) {
    // WebSocket will automatically reconnect on hot reload
    while let Some(msg) = ws.recv().await {
        ws.send(msg).await.ok();
    }
}
```

### Manual Reconnection Handling

```javascript
// Client-side JavaScript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onclose = () => {
    // Reconnect after hot reload
    setTimeout(() => {
        console.log('Reconnecting...');
        connectWebSocket();
    }, 1000);
};
```

## Best Practices

### 1. Use State Preservation for Critical Data

```rust
// ❌ Bad: Lost on reload
static COUNTER: AtomicU32 = AtomicU32::new(0);

// ✅ Good: Preserved on reload
let counter = HotReloadState::new(0u32);
```

### 2. Separate Dev and Prod Configs

```rust
#[tokio::main]
async fn main() {
    let server = routes!();
    
    #[cfg(debug_assertions)]
    let server = server.with_hot_reload();
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

### 3. Use Relative Paths

```rust
// ❌ Bad: Breaks on reload
let file = File::open("/absolute/path/file.txt")?;

// ✅ Good: Works with hot reload
let file = File::open("./file.txt")?;
```

### 4. Clean Up Resources

```rust
#[on_reload]
async fn cleanup() {
    // Close file handles
    // Disconnect from services
    // Clear temporary data
}
```

## Troubleshooting

### Hot Reload Not Triggering

**Problem**: Changes not detected

**Solutions**:
- Check file is in watched paths
- Verify file isn't in ignore list
- Check filesystem permissions
- Try increasing debounce time

```toml
[hot_reload]
debounce_ms = 1000  # Increase if changes detected too early
```

### Compilation Errors Prevent Reload

**Problem**: Server stops on compile error

**Solution**: Fix compilation errors. Hot reload will retry automatically.

```bash
# View compilation output
fwk dev --verbose
```

### State Not Preserving

**Problem**: State resets on reload

**Solutions**:
- Use `HotReloadState` wrapper
- Ensure state is `Clone` or `Serialize`
- Check state isn't dropped before reload

```rust
// Make sure state outlives the reload
let state = HotReloadState::new(MyState::default());
let state_clone = state.clone(); // Keep reference
```

### Slow Reloads

**Problem**: Reloads take too long

**Solutions**:
- Use incremental compilation
- Reduce dependencies
- Use faster linker (mold, lld)

```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[profile.dev]
incremental = true
```

## Performance Optimization

### Fast Compilation

```toml
# Cargo.toml
[profile.dev]
opt-level = 1        # Some optimizations
incremental = true   # Faster rebuilds
```

### Linker Configuration

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### Workspace Optimization

For large projects:
```toml
[workspace]
members = ["core", "plugins/*"]

[profile.dev.package."*"]
opt-level = 2  # Optimize dependencies
```

## Comparison with Other Solutions

| Feature | Firework Hot Reload | cargo-watch | systemfd |
|---------|-------------------|--------------|-----------|
| State Preservation | ✅ Yes | ❌ No | ⚠️ Partial |
| WebSocket Reconnect | ✅ Yes | ❌ No | ❌ No |
| Zero Config | ✅ Yes | ⚠️ Manual | ⚠️ Manual |
| Incremental Builds | ✅ Yes | ✅ Yes | ✅ Yes |
| Connection Preservation | ✅ Yes | ❌ No | ✅ Yes |

## Examples

See complete examples:
- [examples/hot_reload_example.rs](../../examples/hot_reload_example.rs)
- [examples/hot_reload_state_example.rs](../../examples/hot_reload_state_example.rs)

## See Also

- [Configuration Guide](./configuration.md)
- [State Management](./state.md)
- [WebSockets](./websockets.md)
