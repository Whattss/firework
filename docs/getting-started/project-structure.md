# 🗂️ Project Structure

Understanding Firework project organization for scalable applications.

---

## Basic Project Layout

```
my-app/
├── Cargo.toml              # Project manifest
├── Firework.toml           # Framework configuration (optional)
├── src/
│   ├── main.rs             # Application entry point
│   ├── routes/             # Route handlers (recommended)
│   │   ├── mod.rs
│   │   ├── users.rs
│   │   └── posts.rs
│   ├── models/             # Data models
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── middleware/         # Custom middleware
│   │   ├── mod.rs
│   │   └── auth.rs
│   └── plugins/            # Custom plugins
│       └── mod.rs
├── static/                 # Static files (CSS, JS, images)
│   ├── css/
│   ├── js/
│   └── images/
├── templates/              # HTML templates (if using)
└── tests/                  # Integration tests
    └── api_tests.rs
```

---

## Cargo.toml Structure

**Minimal setup:**

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
firework = { git = "https://github.com/your-org/firework" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Production setup:**

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core framework
firework = { git = "https://github.com/your-org/firework" }
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database (optional)
firework-seaorm = { path = "../firework/plugins/firework-seaorm" }
sea-orm = { version = "0.12", features = ["sqlx-sqlite", "runtime-tokio-native-tls"] }

# Authentication (optional)
firework-auth = { path = "../firework/plugins/firework-auth" }

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde", "v4"] }

[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## Firework.toml Configuration

**Optional configuration file** at project root:

```toml
# Server Configuration
[server]
address = "127.0.0.1"
port = 8080
workers = 8  # Number of CPU cores

# Plugin Configurations
[plugins.seaorm]
database_url = "sqlite://data.db"

[plugins.auth]
jwt_secret = "your-super-secret-key-change-in-production"
jwt_expiration_hours = 24
jwt_algorithm = "HS256"

[plugins.cache]
ttl = 300
max_size = 1000
enabled = true
```

---

## Entry Point: main.rs

**Small app (single file):**

```rust
use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    routes!().listen("127.0.0.1:8080").await.unwrap();
}
```

**Medium app (modular):**

```rust
use firework::prelude::*;

mod routes;
mod models;
mod middleware;

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("🔥 Server running on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

**Large app (with plugins):**

```rust
use firework::prelude::*;

mod routes;
mod models;
mod middleware;
mod plugins;

#[tokio::main]
async fn main() {
    // Register plugins
    register_plugin_async(Arc::new(plugins::MyPlugin::new())).await.unwrap();
    
    // Build server
    let server = routes!();
    
    // Load config
    let config = get_config().await;
    let addr = config.bind_address();
    
    println!("🔥 Server running on http://{}", addr);
    server.listen(&addr).await.unwrap();
}
```

---

## Modular Routes Structure

### routes/mod.rs

```rust
pub mod users;
pub mod posts;
pub mod auth;

// Re-export handlers
pub use users::*;
pub use posts::*;
pub use auth::*;
```

### routes/users.rs

```rust
use firework::prelude::*;
use crate::models::User;

#[get("/api/users")]
async fn list_users() -> Json<Vec<User>> {
    // Implementation
    Json(vec![])
}

#[get("/api/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, Error> {
    // Implementation
    Err(Error::NotFound("User not found".into()))
}

#[post("/api/users")]
async fn create_user(Json(user): Json<User>) -> Json<User> {
    // Implementation
    Json(user)
}

#[put("/api/users/:id")]
async fn update_user(
    Path(id): Path<u32>,
    Json(user): Json<User>,
) -> Result<Json<User>, Error> {
    // Implementation
    Ok(Json(user))
}

#[delete("/api/users/:id")]
async fn delete_user(Path(id): Path<u32>) -> Response {
    // Implementation
    Response::new(StatusCode::NoContent, b"")
}
```

---

## Models Structure

### models/mod.rs

```rust
pub mod user;
pub mod post;

pub use user::User;
pub use post::Post;
```

### models/user.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: 0,
            username,
            email,
            password_hash: String::new(),
            created_at: chrono::Utc::now(),
        }
    }
}
```

---

## Middleware Structure

### middleware/mod.rs

```rust
pub mod auth;
pub mod logging;
pub mod cors;

pub use auth::*;
pub use logging::*;
pub use cors::*;
```

### middleware/auth.rs

```rust
use firework::prelude::*;

#[middleware]
async fn require_auth(req: &mut Request, res: &mut Response) -> Flow {
    // Check for authentication
    match req.header("Authorization") {
        Some(token) => {
            // Validate token
            Flow::Continue
        }
        None => {
            *res = Response::new(
                StatusCode::Unauthorized,
                b"{\"error\":\"Authentication required\"}"
            );
            Flow::Stop(res.clone())
        }
    }
}

#[middleware]
async fn optional_auth(req: &mut Request, _res: &mut Response) -> Flow {
    // Try to authenticate but don't fail
    if let Some(token) = req.header("Authorization") {
        // Set user context if valid
    }
    Flow::Continue
}
```

---

## Using Scopes for Organization

```rust
use firework::prelude::*;

// Group related routes with scopes
#[scope("/api/v1")]
mod api_v1 {
    use super::*;
    
    #[get("/users")]
    async fn list_users() -> &'static str {
        "List users"
    }
    
    #[get("/posts")]
    async fn list_posts() -> &'static str {
        "List posts"
    }
}

#[scope("/admin", middleware = [require_admin])]
mod admin {
    use super::*;
    
    #[get("/dashboard")]
    async fn dashboard() -> &'static str {
        "Admin dashboard"
    }
}

#[middleware]
async fn require_admin(req: &mut Request, res: &mut Response) -> Flow {
    // Check admin permissions
    Flow::Continue
}
```

---

## Testing Structure

### tests/api_tests.rs

```rust
use firework::prelude::*;

#[tokio::test]
async fn test_user_creation() {
    // Use test client
    let client = TestClient::new();
    
    let response = client
        .post("/api/users")
        .json(&serde_json::json!({
            "username": "testuser",
            "email": "test@example.com"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
}
```

---

## Static Files Organization

```
static/
├── css/
│   ├── main.css
│   └── components/
│       ├── button.css
│       └── navbar.css
├── js/
│   ├── app.js
│   └── utils/
│       └── api.js
├── images/
│   ├── logo.png
│   └── icons/
└── fonts/
```

**Serve static files:**

```rust
#[get("/static/*")]
async fn static_handler(req: Request) -> Response {
    serve_static("./static", &req.uri.path).await
}
```

---

## Database Migrations (with SeaORM)

```
migrations/
├── 001_create_users.sql
├── 002_create_posts.sql
└── 003_add_indexes.sql
```

---

## Environment-Specific Config

### .env (development)

```env
DATABASE_URL=sqlite://dev.db
JWT_SECRET=dev-secret-key
RUST_LOG=debug
```

### .env.production

```env
DATABASE_URL=postgresql://user:pass@localhost/prod
JWT_SECRET=super-secret-production-key
RUST_LOG=info
```

---

## Recommended Project Structure (Large App)

```
my-app/
├── Cargo.toml
├── Firework.toml
├── .env
├── .gitignore
├── README.md
│
├── src/
│   ├── main.rs                 # Entry point
│   │
│   ├── config/                 # Configuration
│   │   ├── mod.rs
│   │   └── settings.rs
│   │
│   ├── routes/                 # Route handlers
│   │   ├── mod.rs
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── users.rs
│   │   │   ├── posts.rs
│   │   │   └── auth.rs
│   │   └── web/
│   │       ├── mod.rs
│   │       └── pages.rs
│   │
│   ├── models/                 # Data models
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── post.rs
│   │   └── dto/
│   │       ├── mod.rs
│   │       └── user_dto.rs
│   │
│   ├── services/               # Business logic
│   │   ├── mod.rs
│   │   ├── user_service.rs
│   │   └── post_service.rs
│   │
│   ├── middleware/             # Custom middleware
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── logging.rs
│   │   └── rate_limit.rs
│   │
│   ├── plugins/                # Custom plugins
│   │   ├── mod.rs
│   │   └── analytics.rs
│   │
│   └── utils/                  # Utilities
│       ├── mod.rs
│       ├── jwt.rs
│       └── validation.rs
│
├── static/                     # Static assets
│   ├── css/
│   ├── js/
│   └── images/
│
├── templates/                  # HTML templates
│   ├── base.html
│   └── pages/
│
├── migrations/                 # Database migrations
│   └── 001_init.sql
│
└── tests/                      # Integration tests
    ├── api/
    │   ├── user_tests.rs
    │   └── post_tests.rs
    └── common/
        └── mod.rs
```

---

## Best Practices

1. **Separate concerns** - Routes, models, services in different modules
2. **Use scopes** - Group related endpoints
3. **Middleware organization** - One file per middleware
4. **Testing** - Mirror src/ structure in tests/
5. **Configuration** - Use Firework.toml for app config
6. **Static files** - Keep separate from source code

---

## Next Steps

- [Routing Guide](../core/routing.md) - Learn advanced routing
- [Handlers Guide](../core/handlers.md) - Handler patterns
- [Testing Guide](../advanced/testing.md) - Write tests
