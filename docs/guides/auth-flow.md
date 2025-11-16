# 🔐 Authentication & Authorization

Implement authentication with the Auth plugin.

---

## Setup

```toml
[dependencies]
firework-auth = { path = "../firework/plugins/firework-auth" }
jsonwebtoken = "9"
argon2 = "0.5"
```

**Firework.toml:**
```toml
[plugins.auth]
jwt_secret = "your-256-bit-secret-key-change-in-production"
jwt_expiration_hours = 24
jwt_algorithm = "HS256"
```

---

## Register Plugin

```rust
use firework_auth::AuthPlugin;

#[tokio::main]
async fn main() {
    // Register auth plugin
    let auth_plugin = AuthPlugin::from_config().await;
    register_plugin_async(Arc::new(auth_plugin)).await.unwrap();
    
    routes!().listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Registration Handler

```rust
use firework_auth::{AuthPlugin, Claims};

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[post("/register")]
async fn register(
    Json(data): Json<RegisterRequest>,
    Extract(auth): Extract<AuthPlugin>,
) -> Result<Json<serde_json::Value>, Error> {
    // Hash password
    let password_hash = AuthPlugin::hash_password(&data.password)
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    // Save user to database...
    let user_id = save_user(&data.username, &password_hash).await?;
    
    // Generate JWT token
    let claims = Claims::new(user_id.to_string());
    let token = auth.create_token(claims).await
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({
        "token": token,
        "user_id": user_id
    })))
}
```

---

## Login Handler

```rust
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[post("/login")]
async fn login(
    Json(data): Json<LoginRequest>,
    Extract(auth): Extract<AuthPlugin>,
) -> Result<Json<serde_json::Value>, Error> {
    // Find user by username
    let user = find_user_by_username(&data.username).await
        .ok_or_else(|| Error::Unauthorized("Invalid credentials".into()))?;
    
    // Verify password
    let is_valid = AuthPlugin::verify_password(&data.password, &user.password_hash)
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    if !is_valid {
        return Err(Error::Unauthorized("Invalid credentials".into()));
    }
    
    // Generate token
    let claims = Claims::new(user.id.to_string());
    let token = auth.create_token(claims).await
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({
        "token": token,
        "user_id": user.id
    })))
}
```

---

## Protected Routes

```rust
use firework_auth::Auth;

#[get("/profile")]
async fn profile(Auth(claims): Auth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": claims.sub,
        "authenticated": true
    }))
}

// Or use middleware
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
