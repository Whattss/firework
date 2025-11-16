# 🔐 Auth Plugin

JWT authentication plugin for Firework.

---

## Installation

```toml
[dependencies]
firework-auth = { path = "../firework/plugins/firework-auth" }
```

**Firework.toml:**
```toml
[plugins.auth]
jwt_secret = "your-super-secret-key-256-bits-minimum"
jwt_expiration_hours = 24
jwt_algorithm = "HS256"
```

---

## Setup

```rust
use firework_auth::AuthPlugin;

#[tokio::main]
async fn main() {
    // Register plugin
    let auth = AuthPlugin::from_config().await;
    register_plugin_async(Arc::new(auth)).await.unwrap();
    
    routes!().listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Creating Tokens

```rust
use firework_auth::{AuthPlugin, Claims};

#[post("/login")]
async fn login(
    Json(creds): Json<LoginRequest>,
    Extract(auth): Extract<AuthPlugin>,
) -> Result<Json<serde_json::Value>, Error> {
    // Verify credentials...
    
    // Create claims
    let claims = Claims::new("user_123")
        .expires_in_hours(24)
        .with_claim("role", serde_json::json!("admin"));
    
    // Generate token
    let token = auth.create_token(claims).await
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({
        "token": token
    })))
}
```

---

## Protecting Routes

### Method 1: Extractor

```rust
use firework_auth::Auth;

#[get("/profile")]
async fn profile(Auth(claims): Auth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": claims.sub,
        "custom_claim": claims.get_claim("role")
    }))
}
```

### Method 2: Middleware

```rust
use firework_auth::require_auth;

#[scope("/api", middleware = [require_auth])]
mod api {
    #[get("/protected")]
    async fn protected(req: Request) -> String {
        let claims = req.get_context::<Claims>().unwrap();
        format!("Hello, user {}!", claims.sub)
    }
}
```

---

## Password Hashing

```rust
use firework_auth::AuthPlugin;

// Hash password
let hash = AuthPlugin::hash_password("secret123")
    .map_err(|e| Error::Internal(e.to_string()))?;

// Save hash to database...

// Later, verify password
let is_valid = AuthPlugin::verify_password("secret123", &hash)
    .map_err(|e| Error::Internal(e.to_string()))?;

if is_valid {
    // Login success
}
```

---

## API Reference

### Claims

```rust
Claims::new(user_id)
    .expires_in_hours(24)
    .with_issuer("my-app")
    .with_audience("web")
    .with_claim("role", json!("admin"))
```

### AuthPlugin Methods

- `create_token(&self, claims: Claims) -> Result<String>`
- `verify_token(&self, token: &str) -> Result<Claims>`
- `hash_password(password: &str) -> Result<String>`
- `verify_password(password: &str, hash: &str) -> Result<bool>`
