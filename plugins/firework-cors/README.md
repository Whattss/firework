# Firework CORS Plugin

Cross-Origin Resource Sharing (CORS) middleware for Firework framework.

## Features

- ✅ Simple one-liner setup for development
- ✅ Full configuration control for production
- ✅ Preflight request handling (OPTIONS)
- ✅ Multiple origins support
- ✅ Credentials support (cookies, auth headers)
- ✅ Custom headers configuration
- ✅ Zero dependencies (except firework core)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
firework = { path = "../../" }
firework-cors = { path = "../../plugins/firework-cors" }
```

## Configuration

### Option 1: From Code (Builder Pattern)

```rust
let cors = CorsPlugin::new()
    .allow_origin("https://myapp.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allow_credentials(true);
```

### Option 2: From Firework.toml

Create a `Firework.toml` in your project root:

```toml
[plugins.cors]
allowed_origins = ["https://myapp.com", "https://admin.myapp.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"]
allowed_headers = ["Content-Type", "Authorization"]
exposed_headers = ["X-Total-Count", "X-Page-Number"]
max_age = 3600
allow_credentials = true
```

Then in your code:

```rust
use firework_cors::CorsPlugin;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Load from Firework.toml
    let cors = CorsPlugin::from_config().await;
    
    firework::register_plugin(Arc::new(cors));
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
```

### Option 3: Permissive (Development Only)

```rust
// Allow all origins - DO NOT USE IN PRODUCTION
firework::register_plugin(Arc::new(CorsPlugin::permissive()));
```

## Quick Start

### Development (Allow All)

```rust
use firework::prelude::*;
use firework_cors::CorsPlugin;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Allow all origins (development only!)
    firework::register_plugin(Arc::new(CorsPlugin::permissive()));
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
```

### Production (Configured)

```rust
use firework_cors::CorsPlugin;

let cors = CorsPlugin::new()
    .allow_origin("https://myapp.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allow_headers(vec!["Content-Type", "Authorization"])
    .allow_credentials(true)
    .max_age(3600);

firework::register_plugin(Arc::new(cors));
```

### Multiple Origins

```rust
let cors = CorsPlugin::new()
    .allow_origins(vec![
        "https://app.example.com",
        "https://admin.example.com",
        "https://mobile.example.com",
    ])
    .allow_methods(vec!["GET", "POST"])
    .allow_credentials(true);
```

### Strict Mode

```rust
// Most restrictive - only same-origin by default
let cors = CorsPlugin::strict()
    .allow_origin("https://myapp.com");
```

## API Reference

### Constructors

- `CorsPlugin::new()` - Default configuration (permissive)
- `CorsPlugin::permissive()` - Allow all origins (development)
- `CorsPlugin::strict()` - Restrictive defaults (production)

### Configuration Methods

All methods use builder pattern:

```rust
let cors = CorsPlugin::new()
    .allow_origin(origin)              // Single origin
    .allow_origins(vec![...])          // Multiple origins
    .allow_methods(vec![...])          // HTTP methods
    .allow_headers(vec![...])          // Request headers
    .expose_headers(vec![...])         // Response headers
    .allow_credentials(bool)           // Enable cookies/auth
    .max_age(seconds)                  // Preflight cache time
```

## Examples

### API with Frontend

```rust
use firework::prelude::*;
use firework_cors::CorsPlugin;
use std::sync::Arc;

#[get("/api/users")]
async fn get_users() -> Response {
    json!({"users": ["Alice", "Bob"]})
}

#[tokio::main]
async fn main() {
    // Allow frontend to access API
    let cors = CorsPlugin::new()
        .allow_origin("http://localhost:3000")
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allow_headers(vec!["Content-Type", "Authorization"])
        .allow_credentials(true);
    
    firework::register_plugin(Arc::new(cors));
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
```

### Mobile App Backend

```rust
let cors = CorsPlugin::new()
    .allow_origins(vec![
        "capacitor://localhost",        // iOS
        "ionic://localhost",            // Ionic
        "http://localhost",             // Android
    ])
    .allow_credentials(true);
```

### Public API

```rust
// Allow anyone to call your API
let cors = CorsPlugin::permissive()
    .expose_headers(vec!["X-Total-Count", "X-Page-Number"]);
```

## How It Works

1. **Preflight Requests**: Handles OPTIONS requests automatically
2. **Origin Validation**: Checks if request origin is allowed
3. **Header Injection**: Adds appropriate CORS headers to responses
4. **Credentials**: Properly handles cookies and auth headers

### Request Flow

```
Browser                    Server
  |                          |
  |--- OPTIONS (preflight)-->|
  |<-- 204 + CORS headers ---|
  |                          |
  |--- GET /api/data ------->|
  |<-- 200 + data + CORS ----|
```

## Security Notes

⚠️ **Never use `permissive()` in production!**

```rust
// ❌ BAD - allows any origin
CorsPlugin::permissive()

// ✅ GOOD - explicit origins
CorsPlugin::new()
    .allow_origin("https://myapp.com")
```

⚠️ **Credentials + Wildcard = Error**

```rust
// ❌ BAD - browsers reject this
CorsPlugin::new()
    .allow_origin("*")
    .allow_credentials(true)  // Won't work!

// ✅ GOOD - specific origin with credentials
CorsPlugin::new()
    .allow_origin("https://myapp.com")
    .allow_credentials(true)
```

## Common Patterns

### SPA (Single Page Application)

```rust
CorsPlugin::new()
    .allow_origin("http://localhost:3000")  // Dev
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allow_headers(vec!["Content-Type"])
```

### Authenticated API

```rust
CorsPlugin::new()
    .allow_origin("https://app.example.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
    .allow_headers(vec!["Content-Type", "Authorization"])
    .allow_credentials(true)
    .max_age(86400)  // 24 hours
```

### Microservices

```rust
CorsPlugin::new()
    .allow_origins(vec![
        "https://service-a.internal",
        "https://service-b.internal",
        "https://service-c.internal",
    ])
```

## Troubleshooting

### Frontend can't access API

1. Check browser console for CORS errors
2. Verify origin is in allowed list
3. Check if credentials are needed

### Cookies not working

```rust
// Make sure both are set:
.allow_origin("https://exact-origin.com")  // Not "*"
.allow_credentials(true)
```

### Custom headers blocked

```rust
// Add to allowed headers:
.allow_headers(vec![
    "Content-Type",
    "Authorization",
    "X-Custom-Header",  // Add your header
])
```

## License

MIT
