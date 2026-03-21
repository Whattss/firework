# Configuration Guide

Firework uses a `Firework.toml` file for centralized configuration management. This allows you to configure server settings, database connections, plugins, and custom options without touching code.

## Quick Start

Create a `Firework.toml` file in your project root:

```toml
[server]
address = "127.0.0.1"
port = 8080
workers = 8

[plugins.seaorm]
database_url = "sqlite://data.db"

[plugins.auth]
jwt_secret = "your-secret-key-min-32-characters-long"
jwt_expiration_hours = 24
```

Then load it in your code:

```rust
use firework::prelude::*;

#[tokio::main]
async fn main() {
    // Load configuration from Firework.toml
    let config = firework::init_config().await;
    
    // Or get specific config
    let server_config = firework::get_config().server;
    println!("Server: {}:{}", server_config.address, server_config.port);
    
    routes!()
        .listen(&format!("{}:{}", server_config.address, server_config.port))
        .await
        .unwrap();
}
```

## Configuration Sections

### [server]

Core server settings.

```toml
[server]
# Listen address (default: "127.0.0.1")
address = "127.0.0.1"

# Listen port (default: 8080)
port = 8080

# Number of worker threads
# Set to 0 for automatic (number of CPU cores)
# Default: 0
workers = 8

# Request timeout in seconds (default: 30)
request_timeout = 30

# Max request body size in bytes (default: 10MB)
# 10485760 = 10 MB
max_body_size = 10485760

# Keep-alive timeout in seconds (default: 30)
keepalive_timeout = 30
```

### [plugins.*]

Plugin-specific configuration. Each plugin has its own section.

#### SeaORM Plugin

```toml
[plugins.seaorm]
# SQLite
database_url = "sqlite://data.db"

# PostgreSQL
# database_url = "postgresql://user:password@localhost:5432/mydb"

# MySQL
# database_url = "mysql://user:password@localhost:3306/mydb"

# Connection pool settings
# max_connections = 32
# min_connections = 5
# connection_timeout = 30
```

#### Auth Plugin

```toml
[plugins.auth]
# JWT secret (minimum 32 characters)
jwt_secret = "your-secret-key-min-32-characters-long"

# Token expiration in hours
jwt_expiration_hours = 24

# JWT algorithm: HS256, HS384, HS512
jwt_algorithm = "HS256"

# Optional: Issuer
issuer = "my-app"

# Optional: Audience
audience = "my-app-users"
```

#### Security Plugin

```toml
[plugins.security]
# X-Frame-Options: DENY, SAMEORIGIN, or ALLOW-FROM
frame_options = "DENY"

# X-Content-Type-Options: nosniff
content_type_nosniff = true

# X-XSS-Protection
xss_protection = true

# HSTS max-age in seconds
# 31536000 = 1 year
hsts_max_age = 31536000

# Content-Security-Policy
csp = "default-src 'self'"

# Referrer-Policy
referrer_policy = "no-referrer"

# Permissions-Policy
# permissions_policy = "geolocation=(), camera=(), microphone=()"
```

#### CORS Plugin

```toml
[plugins.cors]
# Allowed origins (comma-separated or "*")
allowed_origins = "*"

# Allowed methods
allowed_methods = "GET, POST, PUT, DELETE, PATCH"

# Allowed headers
allowed_headers = "Content-Type, Authorization"

# Exposed headers
exposed_headers = "X-Total-Count"

# Allow credentials
allow_credentials = false

# Max age in seconds
max_age = 3600
```

#### Compression Plugin

```toml
[plugins.compress]
# Minimum size to compress (bytes)
min_size = 1024

# Compression levels
gzip_level = 6
brotli_level = 11

# Algorithms to enable
algorithms = "gzip,brotli"
```

#### Vite Plugin

```toml
[plugins.vite]
# Vite dev server port
dev_port = 5173

# Auto-start Vite dev server
auto_start = true

# Build directory
build_dir = "dist"

# Public directory
public_dir = "public"
```

#### DataLoader Plugin

```toml
[plugins.dataloader]
# Maximum batch size
batch_size = 100

# Cache TTL in seconds
cache_ttl = 60

# Enable request-scoped caching
request_cache = true
```

#### Proxy Plugin

```toml
[plugins.proxy]
# Target URL for proxying
target = "http://localhost:3000"

# Path prefix to proxy
path_prefix = "/api"

# Timeout in seconds
timeout = 30

# WebSocket support
websocket = true
```

## Custom Plugin Configuration

You can add configuration for custom plugins:

```toml
[plugins.my_custom_plugin]
option1 = "value1"
option2 = 42
option3 = true
list_option = ["item1", "item2"]

[plugins.my_custom_plugin.nested]
nested_option = "nested_value"
```

Then load it in your plugin:

```rust
use firework::prelude::*;

#[plugin]
struct MyPlugin {
    config: MyConfig,
}

#[derive(Deserialize)]
struct MyConfig {
    option1: String,
    option2: i32,
}

impl MyPlugin {
    pub async fn from_config() -> Self {
        let config: MyConfig = firework::load_plugin_config_as("my_custom_plugin")
            .await
            .unwrap_or_default();
        
        Self { config }
    }
}
```

## Loading Configuration

### Automatic (Recommended)

```rust
#[tokio::main]
async fn main() {
    // Auto-load from Firework.toml
    let config = firework::init_config().await;
    
    // Access server config
    println!("Server: {}:{}", 
        config.server.address, 
        config.server.port
    );
    
    routes!().listen(&format!(
        "{}:{}",
        config.server.address,
        config.server.port
    )).await.unwrap();
}
```

### Manual Access

```rust
// Get loaded config
let config = firework::get_config();
println!("Listening on {}", config.server.address);

// Get specific plugin config
let seaorm_config: Option<SeaOrmConfig> = 
    firework::load_plugin_config("seaorm").await;
```

### From Custom Path

```rust
// Load from non-standard location
std::env::set_var("FIREWORK_CONFIG", "./custom/path/config.toml");
let config = firework::init_config().await;
```

## Environment Variables

Override configuration with environment variables:

```bash
# Server settings
FIREWORK_SERVER_ADDRESS=0.0.0.0
FIREWORK_SERVER_PORT=3000
FIREWORK_SERVER_WORKERS=16

# Plugin settings
FIREWORK_PLUGINS_SEAORM_DATABASE_URL=postgresql://user:pass@localhost/db
FIREWORK_PLUGINS_AUTH_JWT_SECRET=my-secret-key

# Custom plugin
FIREWORK_PLUGINS_CUSTOM_OPTION1=value1
```

In code:

```rust
// Environment variables override TOML
let db_url = std::env::var("FIREWORK_PLUGINS_SEAORM_DATABASE_URL")
    .or_else(|_| {
        // Fallback to TOML config
        Ok(firework::get_config()
            .plugins
            .get("seaorm")
            .map(|c| c["database_url"].as_str().unwrap_or("").to_string())
            .unwrap_or_default())
    });
```

## Development vs Production

Create environment-specific configs:

```bash
# Development
Firework.dev.toml

# Production
Firework.prod.toml

# Staging
Firework.staging.toml
```

Load based on environment:

```rust
#[tokio::main]
async fn main() {
    let env = std::env::var("ENVIRONMENT").unwrap_or("dev".to_string());
    std::env::set_var("FIREWORK_CONFIG", format!("./Firework.{}.toml", env));
    
    let config = firework::init_config().await;
    
    // Rest of application...
}
```

Or use `fwk` CLI:

```bash
# Development (default)
fwk dev

# Production
ENVIRONMENT=prod fwk build && fwk start

# Staging
ENVIRONMENT=staging fwk dev
```

## Example: Full Application Config

```toml
# Firework.toml

[server]
address = "0.0.0.0"
port = 8080
workers = 8
request_timeout = 60
max_body_size = 52428800  # 50 MB for file uploads
keepalive_timeout = 60

[plugins.seaorm]
database_url = "postgresql://app:password@db.example.com:5432/myapp"
max_connections = 32
min_connections = 5

[plugins.auth]
jwt_secret = "super-secret-key-min-32-characters-very-long-and-random"
jwt_expiration_hours = 24
jwt_algorithm = "HS256"
issuer = "myapp.com"
audience = "myapp-users"

[plugins.security]
frame_options = "DENY"
content_type_nosniff = true
xss_protection = true
hsts_max_age = 31536000
csp = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:"
referrer_policy = "strict-origin-when-cross-origin"
permissions_policy = "geolocation=(), camera=(), microphone=()"

[plugins.cors]
allowed_origins = "https://example.com, https://app.example.com"
allowed_methods = "GET, POST, PUT, DELETE, PATCH"
allowed_headers = "Content-Type, Authorization"
exposed_headers = "X-Total-Count, X-Page-Number"
allow_credentials = true
max_age = 3600

[plugins.compress]
min_size = 1024
gzip_level = 6
brotli_level = 11
algorithms = "gzip,brotli"

[plugins.vite]
dev_port = 5173
auto_start = true
build_dir = "dist"

[plugins.dataloader]
batch_size = 100
cache_ttl = 300
request_cache = true

# Custom plugin config
[plugins.email]
smtp_host = "smtp.example.com"
smtp_port = 587
smtp_user = "noreply@example.com"
smtp_password = "password"
from_address = "noreply@example.com"
from_name = "MyApp"

[plugins.email.templates]
welcome_email = "templates/welcome.html"
reset_password = "templates/reset.html"

[plugins.cache]
ttl = 3600
max_size = 10000
```

Then use it:

```rust
use firework::prelude::*;

#[tokio::main]
async fn main() {
    let config = firework::init_config().await;
    
    // All plugins load their config automatically from [plugins.*]
    let auth_plugin = firework_auth::AuthPlugin::from_config().await;
    let db_plugin = firework_seaorm::SeaOrmPlugin::from_config().await;
    let security_plugin = firework_security::SecurityHeadersPlugin::from_config().await;
    
    firework::register_plugin(std::sync::Arc::new(auth_plugin));
    firework::register_plugin(std::sync::Arc::new(db_plugin));
    firework::register_plugin(std::sync::Arc::new(security_plugin));
    
    // Start server with configured address and port
    let addr = config.server.address.clone();
    let port = config.server.port;
    
    routes!()
        .listen(&format!("{}:{}", addr, port))
        .await
        .unwrap();
}
```

## Configuration Errors

If configuration fails:

```rust
use firework::prelude::*;

#[tokio::main]
async fn main() {
    match firework::init_config().await {
        Ok(config) => {
            println!("Config loaded: listening on {}:{}", 
                config.server.address, 
                config.server.port
            );
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Make sure Firework.toml exists and is valid TOML");
            std::process::exit(1);
        }
    }
    
    // Continue...
}
```

## Tips

1. **Version Control**: Commit `Firework.toml` templates, use `Firework.toml.local` for secrets
2. **CI/CD**: Use environment variables for sensitive data (JWT secrets, DB URLs)
3. **Docker**: Mount config at runtime with volumes
4. **Reload**: Restart server to pick up config changes
5. **Type Safety**: Use `load_plugin_config_as::<T>()` for type-checked configs

## License

Same as Firework framework.
