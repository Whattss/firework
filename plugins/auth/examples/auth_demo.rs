use firework::prelude::*;
use firework_auth::{AuthPlugin, Auth, OptionalAuth, Claims, RequestAuthExt};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

// Models
#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    email: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    user: UserInfo,
}

#[derive(Debug, Serialize)]
struct UserInfo {
    id: String,
    username: String,
}

// Routes
#[get("/")]
async fn index(_req: Request, _res: Response) -> Response {
    firework::html!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Firework Auth Example</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }
        .endpoint { background: #f0f0f0; padding: 10px; margin: 10px 0; border-radius: 5px; }
        code { background: #e0e0e0; padding: 2px 6px; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>🔐 Firework Auth Plugin Demo</h1>
    
    <h2>Available Endpoints:</h2>
    
    <div class="endpoint">
        <h3>POST /auth/register</h3>
        <p>Register a new user</p>
        <code>{"username": "alice", "password": "secret123", "email": "alice@example.com"}</code>
    </div>
    
    <div class="endpoint">
        <h3>POST /auth/login</h3>
        <p>Login and get JWT token</p>
        <code>{"username": "alice", "password": "secret123"}</code>
    </div>
    
    <div class="endpoint">
        <h3>GET /auth/me</h3>
        <p>Get current user info (requires auth)</p>
        <code>Authorization: Bearer &lt;token&gt;</code>
    </div>
    
    <div class="endpoint">
        <h3>GET /public</h3>
        <p>Public endpoint (no auth required)</p>
    </div>
    
    <div class="endpoint">
        <h3>GET /protected</h3>
        <p>Protected endpoint (requires auth)</p>
        <code>Authorization: Bearer &lt;token&gt;</code>
    </div>
    
    <div class="endpoint">
        <h3>GET /optional</h3>
        <p>Optional auth (shows different content based on auth)</p>
    </div>
    
    <h2>Try it:</h2>
    <pre>
# Register
curl -X POST http://localhost:8080/auth/register \\
  -H "Content-Type: application/json" \\
  -d '{"username":"alice","password":"secret123","email":"alice@example.com"}'

# Login
curl -X POST http://localhost:8080/auth/login \\
  -H "Content-Type: application/json" \\
  -d '{"username":"alice","password":"secret123"}'

# Access protected route
curl http://localhost:8080/protected \\
  -H "Authorization: Bearer &lt;your-token&gt;"

# Get user info
curl http://localhost:8080/auth/me \\
  -H "Authorization: Bearer &lt;your-token&gt;"
    </pre>
</body>
</html>
    "#)
}

#[get("/public")]
async fn public_route(_req: Request, _res: Response) -> Response {
    firework::json!(serde_json::json!({
        "message": "This is a public endpoint",
        "auth_required": false
    }))
}

#[get("/protected")]
async fn protected_route(Auth(claims): Auth) -> Response {
    firework::json!(serde_json::json!({
        "message": "This is a protected endpoint",
        "user_id": claims.sub,
        "issued_at": claims.iat,
        "expires_at": claims.exp
    }))
}

#[get("/optional")]
async fn optional_route(OptionalAuth(maybe_claims): OptionalAuth) -> Response {
    match maybe_claims {
        Some(claims) => {
            firework::json!(serde_json::json!({
                "message": "Hello authenticated user!",
                "user_id": claims.sub,
                "authenticated": true
            }))
        }
        None => {
            firework::json!(serde_json::json!({
                "message": "Hello anonymous user!",
                "authenticated": false,
                "hint": "Try adding Authorization header"
            }))
        }
    }
}

// Auth scope
#[scope("/auth")]
mod auth_routes {
    use super::*;

    #[post("/register")]
    async fn register(Json(input): Json<RegisterRequest>) -> Response {
        // Hash password
        let password_hash = match AuthPlugin::hash_password(&input.password) {
            Ok(hash) => hash,
            Err(e) => {
                return firework::json!(serde_json::json!({
                    "error": format!("Failed to hash password: {}", e),
                    "status": 500
                }));
            }
        };

        // In a real app, save to database
        println!("User registered: {}", input.username);
        println!("Password hash: {}", password_hash);

        // Create token
        let user_id = "123"; // Would come from database
        let claims = Claims::new(user_id)
            .expires_in_hours(24)
            .with_claim("username", serde_json::json!(input.username))
            .with_claim("email", serde_json::json!(input.email));

        let registry = firework::plugin_registry();
        let registry = registry.read().await;
        let plugin = registry.get::<AuthPlugin>().unwrap();

        match plugin.create_token(claims).await {
            Ok(token) => {
                firework::json!(LoginResponse {
                    token,
                    user: UserInfo {
                        id: user_id.to_string(),
                        username: input.username,
                    }
                })
            }
            Err(e) => {
                firework::json!(serde_json::json!({
                    "error": format!("Token creation failed: {}", e),
                    "status": 500
                }))
            }
        }
    }

    #[post("/login")]
    async fn login(Json(input): Json<LoginRequest>) -> Response {
        // In a real app, fetch user from database
        // For demo, we'll accept any username with password "secret123"
        
        // Simulate password verification
        // In production: let hash = get_from_db(&input.username)
        let demo_hash = AuthPlugin::hash_password("secret123").unwrap();
        
        let is_valid = match AuthPlugin::verify_password(&input.password, &demo_hash) {
            Ok(valid) => valid,
            Err(_) => false,
        };

        if !is_valid {
            return firework::json!(serde_json::json!({
                "error": "Invalid credentials",
                "status": 401
            }));
        }

        // Create token
        let user_id = "123"; // Would come from database
        let claims = Claims::new(user_id)
            .expires_in_hours(24)
            .with_claim("username", serde_json::json!(input.username))
            .with_claim("role", serde_json::json!("user"));

        let registry = firework::plugin_registry();
        let registry = registry.read().await;
        let plugin = registry.get::<AuthPlugin>().unwrap();

        match plugin.create_token(claims).await {
            Ok(token) => {
                firework::json!(LoginResponse {
                    token,
                    user: UserInfo {
                        id: user_id.to_string(),
                        username: input.username,
                    }
                })
            }
            Err(e) => {
                firework::json!(serde_json::json!({
                    "error": format!("Login failed: {}", e),
                    "status": 500
                }))
            }
        }
    }

    #[get("/me")]
    async fn me(Auth(claims): Auth) -> Response {
        firework::json!(serde_json::json!({
            "id": claims.sub,
            "username": claims.get_claim("username"),
            "email": claims.get_claim("email"),
            "role": claims.get_claim("role"),
            "issued_at": claims.iat,
            "expires_at": claims.exp,
        }))
    }

    #[get("/verify")]
    async fn verify(req: Request) -> Response {
        match req.user_id() {
            Some(user_id) => {
                firework::json!(serde_json::json!({
                    "valid": true,
                    "user_id": user_id
                }))
            }
            None => {
                firework::json!(serde_json::json!({
                    "valid": false,
                    "error": "No valid token"
                }))
            }
        }
    }
}

// Admin routes with middleware
#[scope("/admin", middleware = [admin_middleware])]
mod admin_routes {
    use super::*;

    #[get("/dashboard")]
    async fn dashboard(Auth(claims): Auth) -> Response {
        firework::json!(serde_json::json!({
            "message": "Admin Dashboard",
            "admin_id": claims.sub,
            "role": claims.get_claim("role")
        }))
    }

    #[get("/users")]
    async fn list_users(_req: Request) -> Response {
        firework::json!(serde_json::json!({
            "users": [
                {"id": "1", "username": "alice", "role": "admin"},
                {"id": "2", "username": "bob", "role": "user"},
            ]
        }))
    }
}

// Admin middleware - requires auth and admin role
#[middleware]
fn admin_middleware(mut req: Request, res: Response) -> Flow {
    // First, require auth
    let token = match req.headers
        .get("authorization")
        .and_then(|v| v.first())
        .and_then(|s| s.strip_prefix("Bearer "))
    {
        Some(t) => t,
        None => {
            return Flow::Stop(firework::json!(serde_json::json!({
                "error": "Authentication required",
                "status": 401
            })));
        }
    };

    // Verify token
    let claims = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let registry = firework::plugin_registry();
            let registry = registry.read().await;
            
            if let Some(plugin) = registry.get::<AuthPlugin>() {
                plugin.verify_token(token).await.ok()
            } else {
                None
            }
        })
    });

    let claims = match claims {
        Some(c) => c,
        None => {
            return Flow::Stop(firework::json!(serde_json::json!({
                "error": "Invalid token",
                "status": 401
            })));
        }
    };

    // Check admin role
    if claims.get_claim("role") != Some(&serde_json::json!("admin")) {
        return Flow::Stop(firework::json!(serde_json::json!({
            "error": "Admin access required",
            "status": 403
        })));
    }

    req.set_context(claims);
    Flow::Next(req, res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Firework Auth Plugin Example\n");

    // Create auth plugin with custom config
    let auth_config = firework_auth::AuthConfig {
        jwt_secret: "my-super-secret-key".to_string(),
        jwt_expiration_hours: 24,
        jwt_algorithm: "HS256".to_string(),
        issuer: Some("firework-app".to_string()),
        audience: Some("firework-users".to_string()),
    };

    let auth_plugin = Arc::new(AuthPlugin::new(auth_config));
    firework::register_plugin(auth_plugin);

    println!("Routes:");
    println!("  GET  /                    - Home page");
    println!("  GET  /public              - Public endpoint");
    println!("  GET  /protected           - Protected endpoint (requires auth)");
    println!("  GET  /optional            - Optional auth endpoint");
    println!("  POST /auth/register       - Register new user");
    println!("  POST /auth/login          - Login");
    println!("  GET  /auth/me             - Get current user");
    println!("  GET  /auth/verify         - Verify token");
    println!("  GET  /admin/dashboard     - Admin dashboard (requires admin role)");
    println!("  GET  /admin/users         - List users (requires admin role)");
    println!();
    println!("Server running on http://127.0.0.1:8080");
    println!();

    routes!().listen("127.0.0.1:8080").await?;
    Ok(())
}
