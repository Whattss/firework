// Ejemplo Simple: API REST con Autenticación
// Este ejemplo muestra cómo crear una API completa con registro, login y rutas protegidas

use firework::prelude::*;
use firework_auth::{Auth, AuthPlugin, Claims, OptionalAuth, RequestAuthExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// MODELOS
// ============================================================================

#[derive(Debug, Deserialize)]
struct RegisterInput {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginInput {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    success: bool,
    token: String,
    user: UserInfo,
}

#[derive(Debug, Serialize)]
struct UserInfo {
    id: String,
    username: String,
    email: String,
}

// ============================================================================
// RUTAS PÚBLICAS
// ============================================================================

#[get("/")]
async fn index(_req: Request, _res: Response) -> Response {
    firework::html!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>API con Auth</title>
    <style>
        body { font-family: Arial; max-width: 800px; margin: 50px auto; }
        code { background: #f0f0f0; padding: 2px 6px; }
        pre { background: #f0f0f0; padding: 15px; overflow-x: auto; }
    </style>
</head>
<body>
    <h1>🔐 API REST con Autenticación</h1>
    
    <h2>Endpoints Disponibles:</h2>
    
    <h3>Autenticación</h3>
    <ul>
        <li><code>POST /auth/register</code> - Registrar nuevo usuario</li>
        <li><code>POST /auth/login</code> - Iniciar sesión</li>
        <li><code>GET /auth/me</code> - Información del usuario actual (requiere auth)</li>
    </ul>
    
    <h3>Público</h3>
    <ul>
        <li><code>GET /</code> - Esta página</li>
        <li><code>GET /health</code> - Estado del servidor</li>
    </ul>
    
    <h3>Protegido (requiere token)</h3>
    <ul>
        <li><code>GET /protected</code> - Ruta protegida de ejemplo</li>
    </ul>
    
    <h2>Ejemplo de Uso:</h2>
    
    <h3>1. Registrar Usuario</h3>
    <pre>curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "mi-password-seguro"
  }'</pre>
    
    <h3>2. Login</h3>
    <pre>curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "mi-password-seguro"
  }'</pre>
    
    <h3>3. Acceder a Ruta Protegida</h3>
    <pre>curl http://localhost:8080/protected \
  -H "Authorization: Bearer &lt;tu-token-aqui&gt;"</pre>
    
    <h3>4. Ver tu Información</h3>
    <pre>curl http://localhost:8080/auth/me \
  -H "Authorization: Bearer &lt;tu-token-aqui&gt;"</pre>
</body>
</html>
    "#
    )
}

#[get("/health")]
async fn health(_req: Request, _res: Response) -> Response {
    firework::json!(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// ============================================================================
// RUTAS DE AUTENTICACIÓN
// ============================================================================

#[scope("/auth")]
mod auth_routes {
    use super::*;

    /// Registrar nuevo usuario
    #[post("/register")]
    async fn register(Json(input): Json<RegisterInput>) -> Response {
        // Validar input
        if input.username.len() < 3 {
            return firework::json!(serde_json::json!({
                "success": false,
                "error": "Username debe tener al menos 3 caracteres"
            }));
        }

        if input.password.len() < 8 {
            return firework::json!(serde_json::json!({
                "success": false,
                "error": "Password debe tener al menos 8 caracteres"
            }));
        }

        if !input.email.contains('@') {
            return firework::json!(serde_json::json!({
                "success": false,
                "error": "Email inválido"
            }));
        }

        // Hashear password
        let password_hash = match AuthPlugin::hash_password(&input.password) {
            Ok(hash) => hash,
            Err(_) => {
                return firework::json!(serde_json::json!({
                    "success": false,
                    "error": "Error al procesar password"
                }));
            }
        };

        // En producción: guardar en base de datos
        println!("🔐 Usuario registrado:");
        println!("  Username: {}", input.username);
        println!("  Email: {}", input.email);
        println!("  Password hash: {}", &password_hash[..50]);

        // Crear ID de usuario (en producción viene de la DB)
        let user_id = format!("user_{}", uuid::Uuid::new_v4());

        // Crear token JWT
        let claims = Claims::new(&user_id)
            .expires_in_hours(24)
            .with_claim("username", serde_json::json!(input.username))
            .with_claim("email", serde_json::json!(input.email))
            .with_claim("role", serde_json::json!("user"));

        let registry = firework::plugin_registry();
        let registry = registry.read().await;
        let plugin = registry.get::<AuthPlugin>().unwrap();

        let token = match plugin.create_token(claims).await {
            Ok(t) => t,
            Err(_) => {
                return firework::json!(serde_json::json!({
                    "success": false,
                    "error": "Error al crear token"
                }));
            }
        };

        // Respuesta exitosa
        firework::json!(AuthResponse {
            success: true,
            token,
            user: UserInfo {
                id: user_id,
                username: input.username,
                email: input.email,
            },
        })
    }

    /// Iniciar sesión
    #[post("/login")]
    async fn login(Json(input): Json<LoginInput>) -> Response {
        // En producción: buscar usuario en DB
        // Para demo, aceptamos cualquier username con password "demo123"

        // Simular hash de DB
        let demo_hash = AuthPlugin::hash_password("demo123").unwrap();

        // Verificar password
        let is_valid = match AuthPlugin::verify_password(&input.password, &demo_hash) {
            Ok(valid) => valid,
            Err(_) => false,
        };

        if !is_valid {
            return firework::json!(serde_json::json!({
                "success": false,
                "error": "Credenciales inválidas"
            }));
        }

        // Usuario válido, crear token
        let user_id = format!("user_{}", input.username);

        let claims = Claims::new(&user_id)
            .expires_in_hours(24)
            .with_claim("username", serde_json::json!(input.username))
            .with_claim(
                "email",
                serde_json::json!(format!("{}@example.com", input.username)),
            )
            .with_claim("role", serde_json::json!("user"));

        let registry = firework::plugin_registry();
        let registry = registry.read().await;
        let plugin = registry.get::<AuthPlugin>().unwrap();

        let token = match plugin.create_token(claims).await {
            Ok(t) => t,
            Err(_) => {
                return firework::json!(serde_json::json!({
                    "success": false,
                    "error": "Error al crear token"
                }));
            }
        };

        println!("✅ Login exitoso: {}", input.username);

        firework::json!(AuthResponse {
            success: true,
            token,
            user: UserInfo {
                id: user_id,
                username: input.username.clone(),
                email: format!("{}@example.com", input.username),
            },
        })
    }

    /// Obtener información del usuario actual (requiere autenticación)
    #[get("/me")]
    async fn me(Auth(claims): Auth) -> Response {
        firework::json!(serde_json::json!({
            "success": true,
            "user": {
                "id": claims.sub,
                "username": claims.get_claim("username"),
                "email": claims.get_claim("email"),
                "role": claims.get_claim("role"),
            },
            "token_info": {
                "issued_at": claims.iat,
                "expires_at": claims.exp,
            }
        }))
    }

    /// Verificar si el token es válido
    #[get("/verify")]
    async fn verify(OptionalAuth(maybe_claims): OptionalAuth) -> Response {
        match maybe_claims {
            Some(claims) => {
                firework::json!(serde_json::json!({
                    "valid": true,
                    "user_id": claims.sub,
                    "expires_at": claims.exp,
                }))
            }
            None => {
                firework::json!(serde_json::json!({
                    "valid": false,
                    "error": "No token or invalid token"
                }))
            }
        }
    }

    /// Logout (cliente debe eliminar el token)
    #[post("/logout")]
    async fn logout(Auth(claims): Auth) -> Response {
        println!("👋 Logout: {}", claims.sub);

        // En producción con refresh tokens:
        // - Invalidar refresh token en DB
        // - Agregar JWT a blacklist (con Redis)

        firework::json!(serde_json::json!({
            "success": true,
            "message": "Logout exitoso. Elimina el token del cliente."
        }))
    }
}

// ============================================================================
// RUTAS PROTEGIDAS
// ============================================================================

#[get("/protected")]
async fn protected_route(Auth(claims): Auth) -> Response {
    firework::json!(serde_json::json!({
        "message": "¡Acceso autorizado!",
        "user_id": claims.sub,
        "username": claims.get_claim("username"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// Ejemplo con parámetros de ruta
#[get("/users/:id")]
async fn get_user(Path(id): Path<String>, Auth(claims): Auth) -> Response {
    // Verificar que el usuario solo pueda ver su propia información
    // o que sea admin

    let is_admin = claims
        .get_claim("role")
        .and_then(|v| v.as_str())
        .map(|r| r == "admin")
        .unwrap_or(false);

    if claims.sub != id && !is_admin {
        return firework::json!(serde_json::json!({
            "error": "No puedes ver información de otros usuarios",
            "status": 403
        }));
    }

    firework::json!(serde_json::json!({
        "id": id,
        "username": claims.get_claim("username"),
        "email": claims.get_claim("email"),
    }))
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Iniciando servidor con autenticación...\n");

    // Configurar plugin de autenticación
    let auth_config = firework_auth::AuthConfig {
        jwt_secret: "mi-super-secreto-cambiar-en-produccion".to_string(),
        jwt_expiration_hours: 24,
        jwt_algorithm: "HS256".to_string(),
        issuer: Some("mi-api".to_string()),
        audience: Some("usuarios".to_string()),
    };

    let auth_plugin = Arc::new(AuthPlugin::new(auth_config));
    firework::register_plugin(auth_plugin);

    println!("✅ Plugin de autenticación registrado");
    println!("\n📋 Rutas disponibles:");
    println!("  GET    /                - Página de inicio");
    println!("  GET    /health          - Health check");
    println!("  POST   /auth/register   - Registrar usuario");
    println!("  POST   /auth/login      - Login (usa password: demo123)");
    println!("  GET    /auth/me         - Info del usuario (requiere token)");
    println!("  GET    /auth/verify     - Verificar token");
    println!("  POST   /auth/logout     - Logout");
    println!("  GET    /protected       - Ruta protegida de ejemplo");
    println!("  GET    /users/:id       - Ver usuario (requiere token)");
    println!("\n🔑 Para testing rápido:");
    println!("  Username: cualquiera");
    println!("  Password: demo123");
    println!("\n🌐 Servidor escuchando en http://127.0.0.1:8080");
    println!("   Visita http://127.0.0.1:8080 para ver la documentación\n");

    routes!().listen("127.0.0.1:8080").await?;
    Ok(())
}
