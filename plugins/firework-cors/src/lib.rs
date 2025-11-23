//! # Firework CORS Plugin
//! 
//! Cross-Origin Resource Sharing (CORS) middleware for Firework framework.

use firework::{Plugin, PluginResult, PluginMetadata, Request, Response, Method, StatusCode, ResponseBody};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub max_age: Option<u32>,
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(), "POST".to_string(), "PUT".to_string(), 
                "DELETE".to_string(), "PATCH".to_string(), "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: vec![],
            max_age: Some(3600),
            allow_credentials: false,
        }
    }
}

#[derive(Clone)]
pub struct CorsPlugin {
    config: CorsConfig,
}

impl CorsPlugin {
    /// Create new CORS plugin with default config
    pub fn new() -> Self {
        Self { config: CorsConfig::default() }
    }
    
    /// Create CORS plugin from Firework.toml config
    /// 
    /// # Example Firework.toml
    /// 
    /// ```toml
    /// [plugins.cors]
    /// allowed_origins = ["https://myapp.com", "https://admin.myapp.com"]
    /// allowed_methods = ["GET", "POST", "PUT", "DELETE"]
    /// allowed_headers = ["Content-Type", "Authorization"]
    /// exposed_headers = ["X-Total-Count"]
    /// max_age = 3600
    /// allow_credentials = true
    /// ```
    pub async fn from_config() -> Self {
        let config: CorsConfig = firework::load_plugin_config_as("cors")
            .await
            .unwrap_or_default();
        
        Self { config }
    }
    
    /// Create with custom config
    pub fn with_config(config: CorsConfig) -> Self {
        Self { config }
    }
    
    pub fn permissive() -> Self {
        Self::new()
    }
    
    pub fn strict() -> Self {
        Self {
            config: CorsConfig {
                allowed_origins: vec![],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
                exposed_headers: vec![],
                max_age: Some(86400),
                allow_credentials: true,
            },
        }
    }
    
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.config.allowed_origins = vec![origin.into()];
        self
    }
    
    pub fn allow_origins(mut self, origins: Vec<impl Into<String>>) -> Self {
        self.config.allowed_origins = origins.into_iter().map(|o| o.into()).collect();
        self
    }
    
    pub fn allow_methods(mut self, methods: Vec<impl Into<String>>) -> Self {
        self.config.allowed_methods = methods.into_iter().map(|m| m.into()).collect();
        self
    }
    
    pub fn allow_headers(mut self, headers: Vec<impl Into<String>>) -> Self {
        self.config.allowed_headers = headers.into_iter().map(|h| h.into()).collect();
        self
    }
    
    pub fn expose_headers(mut self, headers: Vec<impl Into<String>>) -> Self {
        self.config.exposed_headers = headers.into_iter().map(|h| h.into()).collect();
        self
    }
    
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.config.allow_credentials = allow;
        self
    }
    
    pub fn max_age(mut self, seconds: u32) -> Self {
        self.config.max_age = Some(seconds);
        self
    }
    
    fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.config.allowed_origins.contains(&"*".to_string()) {
            return true;
        }
        self.config.allowed_origins.iter().any(|o| o == origin)
    }
    
    fn get_origin_header(&self, request_origin: Option<&str>) -> String {
        if self.config.allow_credentials {
            if let Some(origin) = request_origin {
                if self.is_origin_allowed(origin) {
                    return origin.to_string();
                }
            }
            return "".to_string();
        }
        
        if self.config.allowed_origins.contains(&"*".to_string()) {
            "*".to_string()
        } else if let Some(origin) = request_origin {
            if self.is_origin_allowed(origin) {
                origin.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }
    
    fn is_options(&self, method: &Method) -> bool {
        matches!(method, Method::OPTIONS)
    }
    
    fn handle_preflight(&self, req: &Request) -> Response {
        let origin = req.headers
            .get("origin")
            .and_then(|v| v.first())
            .map(|s| s.as_str());
        
        let mut headers = HashMap::new();
        
        let origin_header = self.get_origin_header(origin);
        if !origin_header.is_empty() {
            headers.insert("Access-Control-Allow-Origin".to_string(), origin_header);
        }
        
        headers.insert("Access-Control-Allow-Methods".to_string(), self.config.allowed_methods.join(", "));
        headers.insert("Access-Control-Allow-Headers".to_string(), self.config.allowed_headers.join(", "));
        
        if self.config.allow_credentials {
            headers.insert("Access-Control-Allow-Credentials".to_string(), "true".to_string());
        }
        
        if let Some(max_age) = self.config.max_age {
            headers.insert("Access-Control-Max-Age".to_string(), max_age.to_string());
        }
        
        Response {
            version: firework::Version::Http11,
            status: StatusCode::NoContent,
            headers,
            body: ResponseBody::Static(vec![]),
        }
    }
    
    fn add_cors_headers(&self, req: &Request, res: &mut Response) {
        let origin = req.headers
            .get("origin")
            .and_then(|v| v.first())
            .map(|s| s.as_str());
        
        let origin_header = self.get_origin_header(origin);
        
        if !origin_header.is_empty() {
            res.headers.insert("Access-Control-Allow-Origin".to_string(), origin_header);
        }
        
        if self.config.allow_credentials {
            res.headers.insert("Access-Control-Allow-Credentials".to_string(), "true".to_string());
        }
        
        if !self.config.exposed_headers.is_empty() {
            res.headers.insert("Access-Control-Expose-Headers".to_string(), self.config.exposed_headers.join(", "));
        }
        
        res.headers.insert("Vary".to_string(), "Origin".to_string());
    }
}

impl Default for CorsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Plugin for CorsPlugin {
    fn name(&self) -> &'static str {
        "CORS"
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "CORS",
            version: "1.0.0",
            author: "Firework Team",
            description: "Cross-Origin Resource Sharing (CORS) middleware",
        }
    }
    
    fn priority(&self) -> i32 {
        100
    }
    
    async fn on_init(&self) -> PluginResult<()> {
        println!("[CORS] Plugin initialized");
        
        if self.config.allowed_origins.contains(&"*".to_string()) {
            println!("[CORS] WARNING: Allowing ALL origins (*)");
            println!("[CORS] This is NOT recommended for production!");
        } else {
            println!("[CORS] Allowed origins: {:?}", self.config.allowed_origins);
        }
        
        println!("[CORS] Allowed methods: {:?}", self.config.allowed_methods);
        println!("[CORS] Allow credentials: {}", self.config.allow_credentials);
        
        Ok(())
    }
    
    async fn on_request(&self, req: &mut Request, _res: &mut Response) -> PluginResult<Option<Response>> {
        if self.is_options(&req.method) {
            let has_origin = req.headers.contains_key("origin");
            let has_request_method = req.headers.contains_key("access-control-request-method");
            
            if has_origin && has_request_method {
                return Ok(Some(self.handle_preflight(req)));
            }
        }
        
        Ok(None)
    }
    
    async fn on_response(&self, req: &Request, res: &mut Response) -> PluginResult<()> {
        self.add_cors_headers(req, res);
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn cors() -> CorsPlugin {
    CorsPlugin::permissive()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permissive_config() {
        let cors = CorsPlugin::permissive();
        assert!(cors.config.allowed_origins.contains(&"*".to_string()));
        assert!(!cors.config.allow_credentials);
    }
    
    #[test]
    fn test_strict_config() {
        let cors = CorsPlugin::strict();
        assert!(cors.config.allowed_origins.is_empty());
        assert!(cors.config.allow_credentials);
    }
    
    #[test]
    fn test_origin_allowed() {
        let cors = CorsPlugin::new()
            .allow_origin("https://example.com");
        
        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(!cors.is_origin_allowed("https://evil.com"));
    }
    
    #[test]
    fn test_wildcard_origin() {
        let cors = CorsPlugin::permissive();
        assert!(cors.is_origin_allowed("https://anything.com"));
    }
    
    #[test]
    fn test_builder_pattern() {
        let cors = CorsPlugin::new()
            .allow_origin("https://app.com")
            .allow_methods(vec!["GET", "POST"])
            .allow_headers(vec!["Content-Type"])
            .allow_credentials(true)
            .max_age(7200);
        
        assert_eq!(cors.config.allowed_origins, vec!["https://app.com"]);
        assert_eq!(cors.config.allowed_methods, vec!["GET", "POST"]);
        assert_eq!(cors.config.allowed_headers, vec!["Content-Type"]);
        assert_eq!(cors.config.max_age, Some(7200));
        assert!(cors.config.allow_credentials);
    }
}
