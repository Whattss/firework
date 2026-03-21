# Firework Auth Plugin

JWT-based authentication and password hashing plugin for Firework. Provides token generation, verification, and Argon2 password hashing out of the box.

## Features

- JWT token generation and verification
- Argon2id password hashing (memory-hard, resistant to GPU attacks)
- Multiple JWT algorithms (HS256, HS384, HS512)
- Custom claims support
- Request extractors for authenticated routes
- Optional authentication (for routes that work with/without auth)
- Zero-copy token validation with request context caching

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
firework-auth = { git = "https://github.com/Whattss/firework" }
```

## Quick Start

### 1. Register the Plugin

```rust
use firework::prelude::*;
use firework_auth::{AuthPlugin, AuthConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Configure authentication
    let auth_config = AuthConfig {
        jwt_secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key-min-32-chars".to_string()),
        jwt_expiration_hours: 24,
        jwt_algorithm: "HS256".to_string(),
        issuer: Some("my-app".to_string()),
        audience: Some("my-app-users".to_string()),
    };
    
    // Register plugin
    let auth_plugin = Arc::new(AuthPlugin::new(auth_config));
    firework::register_plugin(auth_plugin);
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

### 2. Protected Routes

Use the `Auth` extractor to require authentication:

```rust
use firework::prelude::*;
use firework_auth::Auth;

#[get("/protected")]
async fn protected_route(Auth(claims): Auth) -> String {
    format!("Hello, user {}!", claims.sub)
}

#[get("/profile")]
async fn get_profile(Auth(claims): Auth) -> Response {
    json!({
        "user_id": claims.sub,
        "issued_at": claims.iat,
        "expires_at": claims.exp
    })
}
```

If the request doesn't have a valid token, returns 401 Unauthorized automatically.

## Core Concepts

### JWT Claims

The `Claims` struct represents the JWT payload:

```rust
pub struct Claims {
    pub sub: String,           // Subject (user ID)
    pub exp: usize,            // Expiration time (Unix timestamp)
    pub iat: usize,            // Issued at (Unix timestamp)
    pub iss: Option<String>,   // Issuer
    pub aud: Option<String>,   // Audience
    pub extra: serde_json::Value,  // Custom claims
}
```

### Creating Tokens

```rust
use firework_auth::{AuthPlugin, Claims};

#[post("/login")]
async fn login(Json(credentials): Json<LoginRequest>) -> Response {
    // Verify credentials (check password, etc.)
    // ...
    
    // Get auth plugin
    let registry = firework::plugin_registry().read().await;
    let auth = registry.get::<AuthPlugin>().unwrap();
    
    // Create claims
    let claims = Claims::new(user.id.to_string())
        .expires_in_hours(24)
        .with_issuer("my-app")
        .with_audience("my-app-users")
        .with_claim("role", json!("admin"))
        .with_claim("email", json!(user.email));
    
    // Generate token
    let token = auth.create_token(claims).await.unwrap();
    
    json!({
        "token": token,
        "expires_in": 86400  // 24 hours in seconds
    })
}
```

### Verifying Tokens

Tokens are automatically verified by the `Auth` extractor, but you can manually verify them:

```rust
use firework_auth::AuthPlugin;

async fn verify_manual(token: &str) -> Result<Claims, AuthError> {
    let registry = firework::plugin_registry().read().await;
    let auth = registry.get::<AuthPlugin>().unwrap();
    
    auth.verify_token(token).await
}
```

### Password Hashing

Use Argon2id for secure password hashing:

```rust
use firework_auth::AuthPlugin;

#[post("/register")]
async fn register(Json(data): Json<RegisterRequest>) -> Response {
    // Hash password
    let password_hash = AuthPlugin::hash_password(&data.password)
        .expect("Failed to hash password");
    
    // Store password_hash in database
    // ...
    
    json!({
        "message": "User registered successfully"
    })
}

#[post("/login")]
async fn login(Json(data): Json<LoginRequest>) -> Response {
    // Fetch user from database
    // let user = ...;
    
    // Verify password
    let is_valid = AuthPlugin::verify_password(&data.password, &user.password_hash)
        .expect("Failed to verify password");
    
    if !is_valid {
        return Response::new(StatusCode::Unauthorized, vec![])
            .json(json!({"error": "Invalid credentials"}));
    }
    
    // Generate token
    // ...
}
```

## Advanced Usage

### Custom Claims

Add custom data to your JWT tokens:

```rust
let claims = Claims::new(user.id.to_string())
    .with_claim("role", json!("admin"))
    .with_claim("permissions", json!(["read", "write", "delete"]))
    .with_claim("organization_id", json!(org.id))
    .with_claim("subscription_tier", json!("premium"));

// Access custom claims
#[get("/admin")]
async fn admin_only(Auth(claims): Auth) -> Response {
    let role = claims.get_claim("role")
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    
    if role != "admin" {
        return Response::new(StatusCode::Forbidden, vec![])
            .json(json!({"error": "Admin access required"}));
    }
    
    json!({"message": "Welcome, admin!"})
}
```

### Optional Authentication

Use `OptionalAuth` for routes that work with or without authentication:

```rust
use firework_auth::OptionalAuth;

#[get("/posts")]
async fn list_posts(OptionalAuth(claims): OptionalAuth) -> Response {
    let posts = if let Some(claims) = claims {
        // Authenticated user - show private posts too
        fetch_all_posts(&claims.sub).await
    } else {
        // Anonymous user - show only public posts
        fetch_public_posts().await
    };
    
    json!(posts)
}

#[get("/profile")]
async fn get_profile(OptionalAuth(claims): OptionalAuth) -> Response {
    match claims {
        Some(claims) => json!({
            "authenticated": true,
            "user_id": claims.sub,
        }),
        None => json!({
            "authenticated": false,
            "message": "Please log in to see your profile"
        })
    }
}
```

### Request Extension Trait

Access claims directly from the request:

```rust
use firework_auth::RequestAuthExt;

async fn my_handler(req: Request) -> Response {
    if let Some(user_id) = req.user_id() {
        // User is authenticated
        json!({"user_id": user_id})
    } else {
        // User is not authenticated
        json!({"error": "Not authenticated"})
    }
}

// Or get full claims
async fn another_handler(req: Request) -> Response {
    if let Some(claims) = req.claims() {
        json!({
            "user_id": claims.sub,
            "expires_at": claims.exp
        })
    } else {
        json!({"error": "Not authenticated"})
    }
}
```

### Auth Middleware

Create middleware to protect entire route groups:

```rust
use firework::prelude::*;
use firework_auth::{AuthPlugin, Claims};

async fn auth_middleware(req: &mut Request, res: &mut Response) -> Flow {
    // Extract token from header
    let token = req.headers
        .get("authorization")
        .and_then(|v| v.first())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());
    
    let token = match token {
        Some(t) => t,
        None => {
            *res = Response::new(StatusCode::Unauthorized, vec![])
                .json(json!({"error": "No authorization token"}));
            return Flow::Stop;
        }
    };
    
    // Verify token
    let registry = firework::plugin_registry().read().await;
    let auth = registry.get::<AuthPlugin>().unwrap();
    
    match auth.verify_token(&token).await {
        Ok(claims) => {
            // Store claims in request context
            req.set_context(claims);
            Flow::Continue
        }
        Err(_) => {
            *res = Response::new(StatusCode::Unauthorized, vec![])
                .json(json!({"error": "Invalid or expired token"}));
            Flow::Stop
        }
    }
}

// Apply to route group
#[scope("/api", middleware = [auth_middleware])]
mod api {
    #[get("/protected")]
    async fn protected() -> String {
        "This route requires authentication".to_string()
    }
    
    #[get("/admin")]
    async fn admin() -> String {
        "Admin-only route".to_string()
    }
}
```

### Token Refresh

Implement token refresh pattern:

```rust
#[post("/refresh")]
async fn refresh_token(Auth(old_claims): Auth) -> Response {
    let registry = firework::plugin_registry().read().await;
    let auth = registry.get::<AuthPlugin>().unwrap();
    
    // Create new token with same user ID but new expiration
    let new_claims = Claims::new(old_claims.sub.clone())
        .expires_in_hours(24)
        .with_issuer(old_claims.iss.clone().unwrap_or_default())
        .with_audience(old_claims.aud.clone().unwrap_or_default());
    
    // Copy custom claims from old token
    let mut new_claims = new_claims;
    if let Some(obj) = old_claims.extra.as_object() {
        for (key, value) in obj {
            new_claims = new_claims.with_claim(key, value.clone());
        }
    }
    
    let token = auth.create_token(new_claims).await.unwrap();
    
    json!({
        "token": token,
        "expires_in": 86400
    })
}
```

## Configuration

### Environment Variables

Load configuration from environment:

```rust
let auth_config = AuthConfig {
    jwt_secret: std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set"),
    jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap(),
    jwt_algorithm: std::env::var("JWT_ALGORITHM")
        .unwrap_or_else(|_| "HS256".to_string()),
    issuer: std::env::var("JWT_ISSUER").ok(),
    audience: std::env::var("JWT_AUDIENCE").ok(),
};
```

### Config File

Or load from Firework config file:

```rust
// In firework.config.toml:
[plugins.auth]
jwt_secret = "your-secret-key-min-32-chars"
jwt_expiration_hours = 24
jwt_algorithm = "HS256"
issuer = "my-app"
audience = "my-app-users"

// In code:
let auth_plugin = Arc::new(AuthPlugin::from_config().await);
```

### Supported Algorithms

- `HS256` - HMAC with SHA-256 (default, recommended)
- `HS384` - HMAC with SHA-384
- `HS512` - HMAC with SHA-512

## Security Best Practices

### JWT Secret

1. **Minimum length**: 32 characters (enforced at startup)
2. **Randomness**: Use cryptographically secure random generator
3. **Storage**: Store in environment variables, never in code
4. **Rotation**: Change secrets periodically and invalidate old tokens

```bash
# Generate secure secret (Linux/Mac)
openssl rand -base64 48

# Generate secure secret (any platform with Python)
python3 -c "import secrets; print(secrets.token_urlsafe(48))"
```

### Password Hashing

Argon2id is the winner of the Password Hashing Competition and is resistant to:
- **GPU attacks**: Memory-hard algorithm
- **Side-channel attacks**: Constant-time verification
- **Dictionary attacks**: High computational cost

Default parameters (secure for most applications):
- Memory: 19 MiB
- Iterations: 2
- Parallelism: 1
- Output length: 32 bytes

### Token Expiration

Shorter expiration times are more secure but require more frequent refreshes:

```rust
// Short-lived access tokens (recommended for high-security apps)
let claims = Claims::new(user_id).expires_in_hours(1);

// Long-lived tokens (better UX, lower security)
let claims = Claims::new(user_id).expires_in_hours(168); // 1 week

// Custom expiration
let claims = Claims::new(user_id).expires_in_hours(24 * 30); // 30 days
```

### HTTPS Only

Always use HTTPS in production to prevent token interception:

```rust
// In your middleware
if !req.is_https() && is_production() {
    return Flow::Stop;
}
```

## Error Handling

### Auth Errors

```rust
use firework_auth::AuthError;

match auth.create_token(claims).await {
    Ok(token) => { /* success */ }
    Err(AuthError::TokenCreation(msg)) => {
        eprintln!("Failed to create token: {}", msg);
    }
    Err(e) => {
        eprintln!("Auth error: {}", e);
    }
}
```

### Extractor Errors

The `Auth` extractor returns standard Firework errors:

- `401 Unauthorized` - Missing or invalid token
- `500 Internal Server Error` - Plugin not registered or other system error

```rust
// Custom error handling with OptionalAuth
#[get("/api/data")]
async fn get_data(OptionalAuth(claims): OptionalAuth) -> Response {
    let claims = match claims {
        Some(c) => c,
        None => {
            return Response::new(StatusCode::Unauthorized, vec![])
                .json(json!({
                    "error": "AUTHENTICATION_REQUIRED",
                    "message": "This endpoint requires authentication",
                    "code": 401
                }));
        }
    };
    
    // Process authenticated request
    json!({"data": "..."})
}
```

## Performance

### Zero-Copy Validation

Claims are stored in request context after first extraction, avoiding repeated token verification:

```rust
#[get("/multi-extract")]
async fn multi_extract(
    Auth(claims1): Auth,  // Verifies token
    Auth(claims2): Auth,  // Reuses cached claims (zero-cost)
) -> Response {
    // Both extractors see the same claims
    assert_eq!(claims1.sub, claims2.sub);
    json!(claims1)
}
```

### Async Token Operations

All token operations are fully async, never blocking the executor:

```rust
// Token creation and verification are non-blocking
let token = auth.create_token(claims).await?;  // Async
let claims = auth.verify_token(&token).await?;  // Async
```

### Password Hashing

Password hashing is intentionally slow (CPU-hard) to prevent brute-force attacks. Use async tasks to avoid blocking:

```rust
use tokio::task;

#[post("/register")]
async fn register(Json(data): Json<RegisterRequest>) -> Response {
    // Hash password in blocking thread pool
    let password_hash = task::spawn_blocking(move || {
        AuthPlugin::hash_password(&data.password)
    })
    .await
    .unwrap()
    .unwrap();
    
    // Save to database
    // ...
}
```

## Examples

### Complete Auth Flow

```rust
use firework::prelude::*;
use firework_auth::{AuthPlugin, AuthConfig, Claims, Auth};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user_id: String,
    expires_in: u64,
}

#[post("/auth/register")]
async fn register(Json(data): Json<RegisterRequest>) -> Response {
    // Hash password
    let password_hash = tokio::task::spawn_blocking(move || {
        AuthPlugin::hash_password(&data.password)
    })
    .await
    .unwrap()
    .expect("Failed to hash password");
    
    // Create user in database
    // let user_id = create_user(&data.username, &data.email, &password_hash).await;
    let user_id = "123"; // placeholder
    
    // Generate token
    let registry = firework::plugin_registry().read().await;
    let auth = registry.get::<AuthPlugin>().unwrap();
    
    let claims = Claims::new(user_id)
        .expires_in_hours(24)
        .with_claim("email", json!(data.email));
    
    let token = auth.create_token(claims).await.unwrap();
    
    Response::new(StatusCode::Created, vec![]).json(AuthResponse {
        token,
        user_id: user_id.to_string(),
        expires_in: 86400,
    })
}

#[post("/auth/login")]
async fn login(Json(data): Json<LoginRequest>) -> Response {
    // Fetch user from database
    // let user = find_user_by_email(&data.email).await;
    let user = User {
        id: "123".to_string(),
        email: data.email.clone(),
        password_hash: "...".to_string(),
    };
    
    // Verify password
    let password = data.password.clone();
    let password_hash = user.password_hash.clone();
    
    let is_valid = tokio::task::spawn_blocking(move || {
        AuthPlugin::verify_password(&password, &password_hash)
    })
    .await
    .unwrap()
    .unwrap();
    
    if !is_valid {
        return Response::new(StatusCode::Unauthorized, vec![])
            .json(json!({"error": "Invalid credentials"}));
    }
    
    // Generate token
    let registry = firework::plugin_registry().read().await;
    let auth = registry.get::<AuthPlugin>().unwrap();
    
    let claims = Claims::new(user.id.clone())
        .expires_in_hours(24)
        .with_claim("email", json!(user.email));
    
    let token = auth.create_token(claims).await.unwrap();
    
    json!(AuthResponse {
        token,
        user_id: user.id,
        expires_in: 86400,
    })
}

#[get("/auth/me")]
async fn get_current_user(Auth(claims): Auth) -> Response {
    json!({
        "user_id": claims.sub,
        "email": claims.get_claim("email"),
        "issued_at": claims.iat,
        "expires_at": claims.exp,
    })
}

#[post("/auth/logout")]
async fn logout(Auth(_claims): Auth) -> Response {
    // JWT tokens are stateless, so logout is handled client-side
    // The client should delete the token from storage
    
    json!({
        "message": "Logged out successfully",
        "note": "Please delete the token from your client"
    })
}

// User struct placeholder
struct User {
    id: String,
    email: String,
    password_hash: String,
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_password_hashing() {
        let password = "super_secret_password123";
        
        // Hash password
        let hash = AuthPlugin::hash_password(password).unwrap();
        
        // Verify correct password
        assert!(AuthPlugin::verify_password(password, &hash).unwrap());
        
        // Verify wrong password
        assert!(!AuthPlugin::verify_password("wrong_password", &hash).unwrap());
    }
    
    #[tokio::test]
    async fn test_token_creation_and_verification() {
        let config = AuthConfig {
            jwt_secret: "test-secret-key-min-32-characters-long".to_string(),
            ..Default::default()
        };
        let auth = AuthPlugin::new(config);
        
        // Create token
        let claims = Claims::new("user123")
            .expires_in_hours(24)
            .with_claim("role", json!("admin"));
        
        let token = auth.create_token(claims.clone()).await.unwrap();
        
        // Verify token
        let verified = auth.verify_token(&token).await.unwrap();
        assert_eq!(verified.sub, "user123");
        assert_eq!(verified.get_claim("role").unwrap(), &json!("admin"));
    }
}
```

## Troubleshooting

### "JWT secret must be at least 32 characters long"

Your JWT secret is too short. Generate a secure secret:

```bash
openssl rand -base64 48
```

### "Invalid or expired token"

Common causes:
1. Token has expired (check `exp` claim)
2. Wrong JWT secret used for verification
3. Token was modified or corrupted
4. Issuer/audience mismatch

### "Auth plugin not registered"

Register the plugin before starting the server:

```rust
let auth_plugin = Arc::new(AuthPlugin::new(config));
firework::register_plugin(auth_plugin);
```

### "Failed to hash password"

Ensure you have enough system resources. Argon2 requires ~19 MiB of memory per hash operation.

## License

Same as Firework framework.
