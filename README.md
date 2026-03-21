# Firework 🔥

A blazingly fast, production-ready web framework for Rust with modern features and exceptional developer experience.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**200k+ requests/sec** | Zero-cost abstractions | Plugin architecture | Hot reload

## Features

### Core Framework
- **Ultra Fast**: 200k+ req/s with optimized routing and connection handling
- **Hot Reload**: Instant feedback during development with state preservation
- **Declarative**: Clean `#[get("/path")]` macros with auto-registration
- **Route Scopes**: Organize routes with `#[scope("/api")]` and middleware
- **WebSockets**: Built-in WebSocket support with rooms and broadcasting
- **Type-Safe**: Powerful extractors for Path, Query, JSON, Headers, Forms
- **Validation**: Input validation with custom validators
- **File Uploads**: Multipart form handling with size limits and type filtering
- **Cookies**: Full cookie support with HttpOnly, Secure, SameSite

### Official Plugins
- **Auth**: JWT authentication with Argon2 password hashing
- **SeaORM**: Seamless database integration with connection pooling
- **CORS**: Configurable cross-origin resource sharing
- **Compression**: Gzip + Brotli compression (70-85% size reduction)
- **Security**: Production-ready security headers (HSTS, CSP, X-Frame-Options)
- **Vite**: Auto-start Vite dev server with HMR proxy
- **DataLoader**: Solve N+1 queries with batching and caching
- **Proxy**: Reverse proxy with load balancing

### Developer Experience
- **CLI Tool**: Project scaffolding, hot reload, route inspection, OpenAPI export
- **Testing**: Built-in test client for integration testing
- **Documentation**: Comprehensive guides and examples
- **Config**: TOML-based configuration system

## Quick Start

### Installation

```bash
# Install CLI
cargo install --path firework-cli

# Create new project
fwk new my-app --template api
cd my-app

# Run with hot reload
fwk dev
```

### Hello World

```rust
use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello, Firework! 🔥"
}

#[get("/users/:id")]
async fn user(Path(id): Path<u32>) -> String {
    format!("User {}", id)
}

#[tokio::main]
async fn main() {
    let server = routes!();
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

### Route Scopes & Modules

```rust
use firework::prelude::*;

// Declarative scopes with middleware
#[scope("/api", middleware = [auth_middleware])]
mod api {
    use super::*;

    #[get("/users")]
    async fn list_users() -> Response {
        // Route: /api/users
        json!({"users": []})
    }

    #[get("/users/:id")]
    async fn get_user(Path(id): Path<u32>) -> Response {
        // Route: /api/users/:id
        json!({"id": id})
    }

    // Nested scopes
    #[scope("/admin", middleware = [admin_middleware])]
    mod admin {
        use super::*;

        #[get("/dashboard")]
        async fn dashboard() -> Response {
            // Route: /api/admin/dashboard
            json!({"page": "dashboard"})
        }
    }
}

#[middleware]
async fn auth_middleware(req: &mut Request, res: &mut Response) -> Flow {
    // Runs for all /api/* routes
    Flow::Continue
}
```

### JSON API Example

```rust
use firework::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
}

#[derive(Serialize)]
struct User {
    id: u32,
    email: String,
}

#[post("/users")]
async fn create_user(
    Validated(Json(user)): Validated<Json<CreateUser>>
) -> Response {
    // User is validated automatically
    let new_user = User {
        id: 1,
        email: user.email,
    };
    
    Response::new(StatusCode::Created, vec![])
        .json(new_user)
}
```

### Production Setup

```rust
use firework::prelude::*;
use firework_cors::CorsPlugin;
use firework_compress::CompressionPlugin;
use firework_security::SecurityHeadersPlugin;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Register production plugins
    firework::register_plugin(Arc::new(CorsPlugin::default()));
    firework::register_plugin(Arc::new(CompressionPlugin::auto()));
    firework::register_plugin(Arc::new(SecurityHeadersPlugin::strict()));
    
    routes!()
        .listen("0.0.0.0:8080")
        .await
        .expect("Failed to start server");
}
```

## Documentation

- [Getting Started Guide](docs/getting-started/installation.md)
- [API Reference](docs/api/)
- [Plugin Development](docs/plugins/custom.md)
- [Examples](examples/)

## CLI Commands

```bash
# Create new project
fwk new my-app --template [basic|api|fullstack]

# Development mode with hot reload
fwk dev

# List all routes
fwk routes --verbose

# Check for route conflicts
fwk routes --check

# Export OpenAPI spec
fwk routes --export openapi

# Show route statistics
fwk routes --stats

# Run custom scripts
fwk run script-name
```

## Available Plugins

| Plugin | Description | Status |
|--------|-------------|--------|
| **firework-auth** | JWT + Argon2 authentication | ✅ Stable |
| **firework-seaorm** | Database ORM integration | ✅ Stable |
| **firework-cors** | CORS middleware | ✅ Stable |
| **firework-compress** | Gzip/Brotli compression | ✅ Stable |
| **firework-security** | Security headers | ✅ Stable |
| **firework-vite** | Vite dev server integration | ✅ Stable |
| **firework-dataloader** | N+1 query solving | ✅ Stable |
| **firework-proxy** | Reverse proxy | ⚠️ Beta |

## Benchmarks

```
Framework: Firework v0.1.0
Machine: 16 cores, 32GB RAM

Simple route (GET /):     ~200,000 req/s
JSON serialization:       ~180,000 req/s
Path parameters:          ~175,000 req/s
With middleware (CORS):   ~160,000 req/s
```

*Run your own: `cargo run --example benchmark_server --release`*

## Project Structure

```
firework/
├── src/                    # Framework core (~10k LOC)
│   ├── server.rs          # HTTP server
│   ├── router.rs          # Radix tree router
│   ├── request.rs         # Request handling
│   ├── response.rs        # Response building
│   ├── extract.rs         # Type extractors
│   ├── validation.rs      # Input validation
│   ├── upload.rs          # File uploads
│   ├── websocket.rs       # WebSocket support
│   └── ...
├── firework-macros/       # Procedural macros
├── firework-cli/          # CLI tool (fwk)
├── plugins/               # Official plugins
│   ├── firework-auth/
│   ├── firework-seaorm/
│   ├── firework-cors/
│   ├── firework-compress/
│   ├── firework-security/
│   ├── firework-vite/
│   ├── firework-dataloader/
│   └── firework-proxy/
├── examples/              # Usage examples
└── docs/                  # Documentation
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](docs/contributing/development.md) first.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Links

- **Documentation**: [docs/](docs/)
- **Examples**: [examples/](examples/)
- **Changelog**: Coming soon
- **Roadmap**: Coming soon

---

Built with ❤️ by the Firework community
