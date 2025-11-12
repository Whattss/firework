# Firework 🔥

A **blazingly fast**, ergonomic, and feature-rich Rust web framework.

## Performance 🚀

**202,000+ requests/sec** on commodity hardware with keep-alive connections.

Competitive with Actix-Web while maintaining excellent developer experience.

See [BENCHMARK_RESULTS.md](BENCHMARK_RESULTS.md) for detailed metrics.

## Features

- Fast async HTTP server with keep-alive support (**202k+ req/s**)
- Powerful routing with radix tree and parameter extraction
- Flexible middleware system (sync & async, pre & post)
- **WebSocket support** with rooms and broadcasting
- Plugin architecture with auto-registration
- Built-in SeaORM integration
- Hot reload for rapid development
- Comprehensive testing utilities
- Static file serving
- Streaming responses
- Scoped routes with dedicated middleware
- Configuration system (TOML)
- Type-safe extractors (JSON, Path, Query, etc.)

## Installation

Add Firework to your `Cargo.toml`:

```toml
[dependencies]
firework = "0.1"
tokio = { version = "1", features = ["full"] }
linkme = "0.3"  # Required for route auto-registration
```

**Note:** `linkme` is required for the routing macros to work. This is a technical limitation of Rust's proc-macro system.

## Quick Start

```rust
use firework::prelude::*;

#[get("/")]
async fn index(_req: Request, res: Response) -> Response {
    res.text("Hello, Firework!")
}

#[get("/api/hello/:name")]
async fn hello(req: Request, res: Response) -> Response {
    let name = req.param("name").unwrap_or("World");
    res.json(serde_json::json!({
        "message": format!("Hello, {}!", name)
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = Server::new("127.0.0.1:8080");
    
    routes!(server => [index, hello]);
    
    server.run().await
}
```

## WebSocket Support 🔌

Real-time communication made easy:

```rust
use firework::prelude::*;

#[ws("/chat")]
async fn chat_handler(mut ws: WebSocket) {
    ws.send_text("Welcome!").await.ok();
    
    while let Some(msg) = ws.recv().await {
        match msg {
            WebSocketMessage::Text(text) => {
                ws.send_text(&text).await.ok(); // Echo
            }
            WebSocketMessage::Close => break,
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

See [WebSocket Guide](docs/WEBSOCKETS.md) for more details.

## Hot Reload

Enable hot reload for rapid development:

```bash
# Run the hot reload example
cargo run --example hot_reload_example --features hot-reload

# Or use the demo script
./scripts/demo-hot-reload.sh
```

Then try editing `examples/hot_reload_example.rs` while the server is running and watch it automatically rebuild!

**In your own code:**

```rust
use firework::prelude::*;

#[tokio::main]
async fn main() {
    #[cfg(feature = "hot-reload")]
    {
        HotReload::new()
            .enable()
            .watch_path("src")
            .start()
            .await
            .ok();
    }

    let server = Server::new();
    // ... setup routes
    server.listen("127.0.0.1:8080").await.ok();
}
```

See [docs/HOT_RELOAD.md](docs/HOT_RELOAD.md) for more details.

## Examples

Run examples with:

```bash
# Basic routing
cargo run --example hello_world

# WebSocket chat
cargo run --example websocket_chat --features websockets

# Database integration
cargo run --example seaorm_example

# Hot reload
cargo run --example hot_reload_example --features hot-reload

# Testing
cargo test --features testing
```

## Documentation

- [WebSocket Guide](docs/WEBSOCKETS.md)
- [Hot Reload Guide](docs/HOT_RELOAD.md)
- [Performance Benchmarks](BENCHMARK_RESULTS.md)
- API documentation: `cargo doc --open`

## License

MIT
