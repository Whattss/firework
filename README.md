# Firework 🔥

A blazingly fast, declarative web framework for Rust with hot-reload, WebSockets, and plugin architecture.

**200k+ requests/sec** | Zero-cost abstractions | Ergonomic APIs

## Quick Start

```bash
cargo install --path firework-cli
fwk new my-app
cd my-app
cargo run
```

## Example

```rust
use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello, Firework!"
}

#[get("/users/:id")]
async fn user(Path(id): Path<u32>) -> String {
    format!("User {}", id)
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("Server running on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

## Features

- **Fast**: 200k+ req/s with keep-alive and optimizations
- **Declarative**: `#[get("/path")]` route macros with auto-registration
- **Flexible**: Multiple handler signatures with extractors
- **WebSockets**: Built-in WebSocket support with `#[ws("/path")]`
- **Hot Reload**: Instant feedback during development
- **Plugins**: Official SeaORM & Auth plugins
- **Type-Safe**: Extractors for Path, Query, JSON, Headers, etc.
- **Minimal**: ~4k LOC core

## Project Structure

- `src/` - Framework core
- `examples/` - Usage examples
- `firework-macros/` - Procedural macros
- `firework-cli/` - CLI tool
- `plugins/` - Official plugins (firework-seaorm, firework-auth)
- `undergun/` - Real-world example app

## License

MIT
