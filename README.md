# Firework 🔥

A blazingly fast, declarative web framework for Rust with hot-reload, WebSockets, and plugin architecture.

**200k+ requests/sec** | Zero-cost abstractions | Ergonomic APIs

## Quick Start

```bash
cargo install --path fwk
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
async fn user(Path(id): Path<u32>) -> Json<User> {
    Json(User::find(id).await)
}

#[tokio::main]
async fn main() {
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

## Features

- **Fast**: 200k+ req/s with keep-alive
- **Declarative**: `#[get("/path")]` route macros
- **Flexible**: Multiple handler signatures
- **WebSockets**: Built-in WebSocket support
- **Hot Reload**: Instant feedback during development
- **Plugins**: Official SeaORM & Auth plugins
- **Type-Safe**: Extractors for Path, Query, JSON, etc.

## Documentation

See `core/README.md` for detailed documentation.

## Project Structure

- `core/` - Framework core
- `macros/` - Procedural macros
- `plugins/` - Official plugins (seaorm, auth)
- `fwk/` - CLI tool
- `undergun/` - Real-world example app

## License

MIT
