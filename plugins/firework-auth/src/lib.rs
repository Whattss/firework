use firework::{Plugin, PluginResult, PluginError, PluginMetadata, Request, Response, Flow};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};

/// Authentication configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub jwt_algorithm: String,
    pub issuer: Option<String>,
    pub audience: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "change-this-secret-in-production".to_string(),
            jwt_expiration_hours: 24,
            jwt_algorithm: "HS256".to_string(),
            issuer: None,
            audience: None,
        }
    }
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // Subject (user ID)
    pub exp: usize,            // Expiration time
    pub iat: usize,            // Issued at
    pub iss: Option<String>,   // Issuer
    pub aud: Option<String>,   // Audience
    #[serde(flatten)]
    pub extra: serde_json::Value,  // Extra custom claims
}

impl Claims {
    /// Create new claims with user ID
    pub fn new(user_id: impl Into<String>) -> Self {
        let now = chrono::Utc::now().timestamp() as usize;
        Self {
            sub: user_id.into(),
            exp: now + 86400, // 24 hours default
            iat: now,
            iss: None,
            aud: None,
            extra: serde_json::json!({}),
        }
    }

    /// Set expiration time (hours from now)
    pub fn expires_in_hours(mut self, hours: i64) -> Self {
        let exp_time = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(hours))
            .unwrap()
            .timestamp() as usize;
        self.exp = exp_time;
        self
    }

    /// Set issuer
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.iss = Some(issuer.into());
        self
    }

    /// Set audience
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.aud = Some(audience.into());
        self
    }

    /// Add custom claim
    pub fn with_claim(mut self, key: &str, value: serde_json::Value) -> Self {
        if let Some(obj) = self.extra.as_object_mut() {
            obj.insert(key.to_string(), value);
        }
        self
    }

    /// Get custom claim
    pub fn get_claim(&self, key: &str) -> Option<&serde_json::Value> {
        self.extra.get(key)
    }
}

/// Authentication plugin for Firework
#[derive(Clone)]
pub struct AuthPlugin {
    config: Arc<RwLock<AuthConfig>>,
}

impl AuthPlugin {
    /// Create new auth plugin with config
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Create from Firework configuration
    pub async fn from_config() -> Self {
        let config: AuthConfig = firework::load_plugin_config_as("auth")
            .await
            .unwrap_or_default();
        Self::new(config)
    }

    /// Get current config
    pub async fn config(&self) -> AuthConfig {
        self.config.read().await.clone()
    }

    /// Generate JWT token
    pub async fn create_token(&self, claims: Claims) -> Result<String, AuthError> {
        let config = self.config.read().await;
        let secret = config.jwt_secret.as_bytes();
        
        let algorithm = match config.jwt_algorithm.as_str() {
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            _ => Algorithm::HS256,
        };
        
        let header = Header::new(algorithm);
        
        encode(&header, &claims, &EncodingKey::from_secret(secret))
            .map_err(|e| AuthError::TokenCreation(e.to_string()))
    }

    /// Verify and decode JWT token
    pub async fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let config = self.config.read().await;
        let secret = config.jwt_secret.as_bytes();
        
        let mut validation = Validation::default();
        validation.algorithms = vec![match config.jwt_algorithm.as_str() {
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            _ => Algorithm::HS256,
        }];
        
        if let Some(ref iss) = config.issuer {
            validation.set_issuer(&[iss]);
        }
        
        if let Some(ref aud) = config.audience {
            validation.set_audience(&[aud]);
        }
        
        decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
            .map(|data| data.claims)
            .map_err(|e| AuthError::TokenVerification(e.to_string()))
    }

    /// Hash password using Argon2
    pub fn hash_password(password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AuthError::PasswordHashing(e.to_string()))
    }

    /// Verify password against hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AuthError::PasswordVerification(e.to_string()))?;
        
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

impl Default for AuthPlugin {
    fn default() -> Self {
        Self::new(AuthConfig::default())
    }
}

// Plugin implementation
#[async_trait::async_trait]
impl Plugin for AuthPlugin {
    fn name(&self) -> &'static str {
        "Auth"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Auth",
            version: "1.0.0",
            author: "Firework Team",
            description: "JWT-based authentication plugin with Argon2 password hashing",
        }
    }

    fn priority(&self) -> i32 {
        50 // Run early in the middleware chain
    }

    async fn on_init(&self) -> PluginResult<()> {
        let config = self.config.read().await;
        println!("[Auth] Plugin initialized v1.0.0");
        println!("[Auth] JWT Algorithm: {}", config.jwt_algorithm);
        println!("[Auth] Token expiration: {} hours", config.jwt_expiration_hours);
        
        if config.jwt_secret == "change-this-secret-in-production" {
            eprintln!("[Auth] WARNING: Using default JWT secret! Change this in production!");
        }
        
        // Validate JWT secret
        if config.jwt_secret.len() < 32 {
            return Err(PluginError(
                "JWT secret must be at least 32 characters long".to_string()
            ));
        }
        
        // Validate algorithm
        match config.jwt_algorithm.as_str() {
            "HS256" | "HS384" | "HS512" => Ok(()),
            _ => Err(PluginError(
                format!("Unsupported JWT algorithm: {}", config.jwt_algorithm)
            )),
        }
    }

    async fn on_start(&self) -> PluginResult<()> {
        println!("[Auth] Ready to authenticate requests");
        Ok(())
    }

    async fn on_shutdown(&self) -> PluginResult<()> {
        println!("[Auth] Shutting down authentication plugin");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Auth extractor for handlers
#[derive(Debug, Clone)]
pub struct Auth(pub Claims);

#[async_trait::async_trait]
impl firework::FromRequest for Auth {
    async fn from_request(req: &mut Request, _res: &mut Response) -> firework::Result<Self> {
        // Try to get from context first (set by middleware)
        if let Some(claims) = req.get_context::<Claims>() {
            return Ok(Auth(claims));
        }

        // Extract from Authorization header
        let token = extract_token_from_header(req)
            .ok_or_else(|| firework::Error::Unauthorized("No authorization token provided".into()))?;

        // Get plugin and verify token (pure async - no blocking!)
        let registry = firework::plugin_registry().read().await;
        let plugin = registry.get::<AuthPlugin>()
            .ok_or_else(|| firework::Error::Internal("Auth plugin not registered".into()))?;
        
        let claims = plugin.verify_token(&token).await
            .map_err(|_| firework::Error::Unauthorized("Invalid or expired token".into()))?;

        Ok(Auth(claims))
    }
}

/// Optional auth extractor (doesn't fail if no token)
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<Claims>);

#[async_trait::async_trait]
impl firework::FromRequest for OptionalAuth {
    async fn from_request(req: &mut Request, _res: &mut Response) -> firework::Result<Self> {
        // Try context first
        if let Some(claims) = req.get_context::<Claims>() {
            return Ok(OptionalAuth(Some(claims)));
        }

        // Try to extract token
        let token = match extract_token_from_header(req) {
            Some(t) => t,
            None => return Ok(OptionalAuth(None)),
        };

        // Verify token (pure async)
        let registry = firework::plugin_registry().read().await;
        if let Some(plugin) = registry.get::<AuthPlugin>() {
            if let Ok(claims) = plugin.verify_token(&token).await {
                return Ok(OptionalAuth(Some(claims)));
            }
        }

        Ok(OptionalAuth(None))
    }
}

/// Extract token from Authorization header
fn extract_token_from_header(req: &Request) -> Option<String> {
    req.headers
        .get("authorization")
        .or_else(|| req.headers.get("Authorization"))
        .and_then(|v| v.first())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Request extension trait for auth
pub trait RequestAuthExt {
    fn claims(&self) -> Option<Claims>;
    fn user_id(&self) -> Option<String>;
}

impl RequestAuthExt for Request {
    fn claims(&self) -> Option<Claims> {
        self.get_context::<Claims>()
    }

    fn user_id(&self) -> Option<String> {
        self.get_context::<Claims>().map(|c| c.sub.clone())
    }
}

/// Auth errors
#[derive(Debug)]
pub enum AuthError {
    TokenCreation(String),
    TokenVerification(String),
    PasswordHashing(String),
    PasswordVerification(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::TokenCreation(msg) => write!(f, "Token creation error: {}", msg),
            AuthError::TokenVerification(msg) => write!(f, "Token verification error: {}", msg),
            AuthError::PasswordHashing(msg) => write!(f, "Password hashing error: {}", msg),
            AuthError::PasswordVerification(msg) => write!(f, "Password verification error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

/// Helper functions and middleware
pub mod helpers {
    use super::*;
    use firework::Response;

    /// Middleware to require authentication
    pub fn require_auth(mut req: Request, res: Response) -> Flow {
        let token = match extract_token_from_header(&req) {
            Some(t) => t,
            None => {
                return Flow::Stop(
                    firework::json!(serde_json::json!({
                        "error": "No authorization token provided",
                        "status": 401
                    }))
                );
            }
        };

        // Verify token and set context
        let claims = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let registry = firework::plugin_registry();
                let registry = registry.read().await;
                
                if let Some(plugin) = registry.get::<AuthPlugin>() {
                    plugin.verify_token(&token).await.ok()
                } else {
                    None
                }
            })
        });

        match claims {
            Some(c) => {
                req.set_context(c);
                Flow::Next(req, res)
            }
            None => Flow::Stop(
                firework::json!(serde_json::json!({
                    "error": "Invalid or expired token",
                    "status": 401
                }))
            ),
        }
    }

    /// Middleware for optional authentication (doesn't fail if no token)
    pub fn optional_auth(mut req: Request, res: Response) -> Flow {
        if let Some(token) = extract_token_from_header(&req) {
            let claims = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let registry = firework::plugin_registry();
                    let registry = registry.read().await;
                    
                    if let Some(plugin) = registry.get::<AuthPlugin>() {
                        plugin.verify_token(&token).await.ok()
                    } else {
                        None
                    }
                })
            });

            if let Some(c) = claims {
                req.set_context(c);
            }
        }
        
        Flow::Next(req, res)
    }

    /// Convert AuthError to Response
    pub fn auth_error_to_response(err: AuthError) -> Response {
        match err {
            AuthError::TokenVerification(_) => {
                firework::json!(serde_json::json!({
                    "error": "Invalid or expired token",
                    "status": 401
                }))
            }
            AuthError::TokenCreation(_) => {
                firework::json!(serde_json::json!({
                    "error": "Failed to create token",
                    "status": 500
                }))
            }
            AuthError::PasswordHashing(_) | AuthError::PasswordVerification(_) => {
                firework::json!(serde_json::json!({
                    "error": "Authentication failed",
                    "status": 401
                }))
            }
        }
    }
}

/// Macros
pub mod macros {
    /// Get auth plugin from registry
    #[macro_export]
    macro_rules! auth_plugin {
        () => {{
            let registry = $crate::firework::plugin_registry();
            let registry = registry.read().await;
            registry.get::<$crate::AuthPlugin>()
        }};
    }

    /// Create JWT token
    #[macro_export]
    macro_rules! create_token {
        ($user_id:expr) => {{
            let plugin = $crate::auth_plugin!();
            match plugin {
                Some(p) => p.create_token($crate::Claims::new($user_id)).await,
                None => Err($crate::AuthError::TokenCreation("Auth plugin not registered".into())),
            }
        }};
        ($claims:expr) => {{
            let plugin = $crate::auth_plugin!();
            match plugin {
                Some(p) => p.create_token($claims).await,
                None => Err($crate::AuthError::TokenCreation("Auth plugin not registered".into())),
            }
        }};
    }

    /// Verify JWT token
    #[macro_export]
    macro_rules! verify_token {
        ($token:expr) => {{
            let plugin = $crate::auth_plugin!();
            match plugin {
                Some(p) => p.verify_token($token).await,
                None => Err($crate::AuthError::TokenVerification("Auth plugin not registered".into())),
            }
        }};
    }
}

// Re-export for convenience
pub use helpers::{require_auth, optional_auth, auth_error_to_response};
