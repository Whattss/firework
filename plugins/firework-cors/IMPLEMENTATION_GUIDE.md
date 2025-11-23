# 🔥 Firework - Guía de Implementación de Features Críticos

## Resumen

Este documento contiene las especificaciones detalladas para implementar los 6 features críticos que le faltan a Firework.

**Tiempo estimado total:** 36 horas  
**Prioridad:** De mayor a menor criticidad

---

## 1. CORS Middleware (6 horas) ⭐⭐⭐⭐⭐

### Ubicación
`plugins/firework-cors/`

### Dependencias
```toml
[dependencies]
firework = { path = "../../" }
serde = { version = "1.0", features = ["derive"] }
```

### API Propuesta
```rust
use firework_cors::CorsPlugin;

// Simple (desarrollo)
let cors = CorsPlugin::permissive();

// Configurado (producción)
let cors = CorsPlugin::new()
    .allow_origin("https://myapp.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allow_headers(vec!["Content-Type", "Authorization"])
    .allow_credentials(true)
    .max_age(3600);

// Multiple origins
let cors = CorsPlugin::new()
    .allow_origins(vec!["https://app1.com", "https://app2.com"]);
```

### Implementación
```rust
pub struct CorsPlugin {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
    exposed_headers: Vec<String>,
    max_age: Option<u32>,
    allow_credentials: bool,
}

impl CorsPlugin {
    pub fn permissive() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS".to_string()],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: vec![],
            max_age: Some(3600),
            allow_credentials: false,
        }
    }
    
    fn handle_preflight(&self, req: &Request, res: &mut Response) -> Flow {
        // Handle OPTIONS request
        res.set_header("Access-Control-Allow-Origin", &self.allowed_origins.join(","));
        res.set_header("Access-Control-Allow-Methods", &self.allowed_methods.join(","));
        res.set_header("Access-Control-Allow-Headers", &self.allowed_headers.join(","));
        
        if self.allow_credentials {
            res.set_header("Access-Control-Allow-Credentials", "true");
        }
        
        if let Some(max_age) = self.max_age {
            res.set_header("Access-Control-Max-Age", &max_age.to_string());
        }
        
        res.status = 204; // No Content
        Flow::Stop(res.clone())
    }
}

#[async_trait::async_trait]
impl Plugin for CorsPlugin {
    fn on_request(&self, req: &mut Request, res: &mut Response) -> Flow {
        // Handle preflight OPTIONS request
        if req.method == Method::OPTIONS {
            return self.handle_preflight(req, res);
        }
        
        // Set CORS headers for actual request
        res.set_header("Access-Control-Allow-Origin", &self.allowed_origins.join(","));
        
        if self.allow_credentials {
            res.set_header("Access-Control-Allow-Credentials", "true");
        }
        
        if !self.exposed_headers.is_empty() {
            res.set_header("Access-Control-Expose-Headers", &self.exposed_headers.join(","));
        }
        
        Flow::Continue
    }
}
```

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use firework::TestClient;
    
    #[tokio::test]
    async fn test_cors_preflight() {
        let cors = CorsPlugin::permissive();
        // Test OPTIONS request
    }
    
    #[tokio::test]
    async fn test_cors_actual_request() {
        // Test GET/POST with CORS headers
    }
}
```

---

## 2. Cookie Support (4 horas) ⭐⭐⭐⭐

### Ubicación
`src/cookie.rs` (core feature, no plugin)

### Dependencias
```toml
cookie = "0.18"
time = "0.3"
```

### API Propuesta
```rust
use firework::Cookie;

// Crear cookie
let cookie = Cookie::new("session_id", "abc123")
    .http_only(true)
    .secure(true)
    .same_site(SameSite::Strict)
    .max_age(Duration::hours(24))
    .path("/")
    .domain(".myapp.com");

// En handler
#[get("/set-cookie")]
fn handler(mut res: Response) -> Response {
    res.set_cookie(cookie);
    res
}

// Leer cookies
#[get("/get-cookie")]
fn handler(req: Request) -> Response {
    if let Some(session) = req.cookie("session_id") {
        // Use session
    }
    Response::ok()
}
```

### Implementación
```rust
use time::{Duration, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct Cookie {
    name: String,
    value: String,
    expires: Option<OffsetDateTime>,
    max_age: Option<Duration>,
    domain: Option<String>,
    path: Option<String>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

#[derive(Debug, Clone, Copy)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            expires: None,
            max_age: None,
            domain: None,
            path: Some("/".to_string()),
            secure: false,
            http_only: false,
            same_site: None,
        }
    }
    
    pub fn to_header_value(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];
        
        if let Some(expires) = self.expires {
            parts.push(format!("Expires={}", expires.format(&Rfc2822).unwrap()));
        }
        
        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age.whole_seconds()));
        }
        
        if let Some(domain) = &self.domain {
            parts.push(format!("Domain={}", domain));
        }
        
        if let Some(path) = &self.path {
            parts.push(format!("Path={}", path));
        }
        
        if self.secure {
            parts.push("Secure".to_string());
        }
        
        if self.http_only {
            parts.push("HttpOnly".to_string());
        }
        
        if let Some(same_site) = self.same_site {
            parts.push(format!("SameSite={}", match same_site {
                SameSite::Strict => "Strict",
                SameSite::Lax => "Lax",
                SameSite::None => "None",
            }));
        }
        
        parts.join("; ")
    }
    
    pub fn parse(s: &str) -> Option<Self> {
        // Parse "Set-Cookie" header value
        // Implementation needed
    }
}

// Add to Request
impl Request {
    pub fn cookie(&self, name: &str) -> Option<&str> {
        self.headers
            .get("cookie")
            .and_then(|cookies| {
                cookies.iter()
                    .flat_map(|s| s.split(';'))
                    .find_map(|pair| {
                        let (k, v) = pair.trim().split_once('=')?;
                        (k == name).then_some(v)
                    })
            })
    }
    
    pub fn cookies(&self) -> Vec<(String, String)> {
        // Parse all cookies
    }
}

// Add to Response
impl Response {
    pub fn set_cookie(&mut self, cookie: Cookie) {
        self.headers
            .entry("Set-Cookie".to_string())
            .or_insert_with(Vec::new)
            .push(cookie.to_header_value());
    }
    
    pub fn delete_cookie(&mut self, name: &str) {
        let cookie = Cookie::new(name, "")
            .max_age(Duration::seconds(0));
        self.set_cookie(cookie);
    }
}
```

---

## 3. Compression Middleware (6 horas) ⭐⭐⭐⭐⭐

### Ubicación
`plugins/firework-compress/`

### Dependencias
```toml
flate2 = "1.0"
brotli = "3.4"
async-compression = { version = "0.4", features = ["tokio", "gzip", "brotli"] }
```

### API Propuesta
```rust
use firework_compress::CompressionPlugin;

// Auto (detecta Accept-Encoding)
let compress = CompressionPlugin::auto();

// Configurado
let compress = CompressionPlugin::new()
    .gzip()
    .brotli()
    .min_size(1024)  // Solo comprimir > 1KB
    .quality(6);     // Nivel de compresión
```

### Implementación
```rust
pub struct CompressionPlugin {
    algorithms: Vec<Algorithm>,
    min_size: usize,
    quality: u32,
}

#[derive(Debug, Clone)]
enum Algorithm {
    Gzip,
    Brotli,
    Deflate,
}

impl CompressionPlugin {
    pub fn auto() -> Self {
        Self {
            algorithms: vec![Algorithm::Brotli, Algorithm::Gzip],
            min_size: 1024,
            quality: 6,
        }
    }
    
    fn choose_algorithm(&self, accept_encoding: &str) -> Option<Algorithm> {
        for algo in &self.algorithms {
            match algo {
                Algorithm::Brotli if accept_encoding.contains("br") => return Some(Algorithm::Brotli),
                Algorithm::Gzip if accept_encoding.contains("gzip") => return Some(Algorithm::Gzip),
                Algorithm::Deflate if accept_encoding.contains("deflate") => return Some(Algorithm::Deflate),
                _ => continue,
            }
        }
        None
    }
    
    fn compress_gzip(&self, data: &[u8]) -> Vec<u8> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.quality));
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }
    
    fn compress_brotli(&self, data: &[u8]) -> Vec<u8> {
        use brotli::enc::BrotliEncoderParams;
        
        let params = BrotliEncoderParams {
            quality: self.quality as i32,
            ..Default::default()
        };
        
        let mut output = Vec::new();
        brotli::BrotliCompress(&mut &data[..], &mut output, &params).unwrap();
        output
    }
}

#[async_trait::async_trait]
impl Plugin for CompressionPlugin {
    async fn on_response(&self, req: &Request, res: &mut Response) -> Flow {
        // Skip if already compressed
        if res.headers.contains_key("content-encoding") {
            return Flow::Continue;
        }
        
        // Skip small responses
        if res.body.len() < self.min_size {
            return Flow::Continue;
        }
        
        // Check Accept-Encoding
        let accept_encoding = req.headers
            .get("accept-encoding")
            .and_then(|v| v.first())
            .map(|s| s.as_str())
            .unwrap_or("");
        
        let Some(algorithm) = self.choose_algorithm(accept_encoding) else {
            return Flow::Continue;
        };
        
        // Compress
        let compressed = match algorithm {
            Algorithm::Gzip => {
                res.set_header("Content-Encoding", "gzip");
                self.compress_gzip(&res.body)
            }
            Algorithm::Brotli => {
                res.set_header("Content-Encoding", "br");
                self.compress_brotli(&res.body)
            }
            Algorithm::Deflate => {
                // Implementation
                vec![]
            }
        };
        
        res.body = compressed;
        res.set_header("Content-Length", &res.body.len().to_string());
        res.set_header("Vary", "Accept-Encoding");
        
        Flow::Continue
    }
}
```

---

## 4. Security Headers (2 horas) ⭐⭐⭐⭐

### Ubicación
`src/middleware/security.rs` (built-in)

### API Propuesta
```rust
use firework::middleware::security_headers;

server.use_middleware(security_headers());

// Configurado
server.use_middleware(
    security_headers()
        .frame_options("DENY")
        .content_type_nosniff(true)
        .xss_protection(true)
        .hsts(31536000)  // 1 year
        .csp("default-src 'self'")
);
```

### Implementación
```rust
pub struct SecurityHeaders {
    frame_options: Option<String>,
    content_type_nosniff: bool,
    xss_protection: bool,
    hsts_max_age: Option<u32>,
    csp: Option<String>,
}

impl SecurityHeaders {
    pub fn strict() -> Self {
        Self {
            frame_options: Some("DENY".to_string()),
            content_type_nosniff: true,
            xss_protection: true,
            hsts_max_age: Some(31536000),
            csp: Some("default-src 'self'".to_string()),
        }
    }
}

impl Plugin for SecurityHeaders {
    fn on_response(&self, _req: &Request, res: &mut Response) -> Flow {
        if let Some(ref fo) = self.frame_options {
            res.set_header("X-Frame-Options", fo);
        }
        
        if self.content_type_nosniff {
            res.set_header("X-Content-Type-Options", "nosniff");
        }
        
        if self.xss_protection {
            res.set_header("X-XSS-Protection", "1; mode=block");
        }
        
        if let Some(max_age) = self.hsts_max_age {
            res.set_header("Strict-Transport-Security", &format!("max-age={}", max_age));
        }
        
        if let Some(ref csp) = self.csp {
            res.set_header("Content-Security-Policy", csp);
        }
        
        Flow::Continue
    }
}
```

---

## 5. Input Validation (8 horas) ⭐⭐⭐⭐⭐

### Ubicación
`src/validation.rs` + integration con extractors

### Dependencias
```toml
validator = { version = "0.18", features = ["derive"] }
```

### API Propuesta
```rust
use firework::Validated;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8, max = 128))]
    password: String,
    
    #[validate(range(min = 18, max = 120))]
    age: u8,
}

#[post("/users")]
async fn create_user(Validated(user): Validated<Json<CreateUser>>) -> Response {
    // user ya está validado
    json!({"created": true})
}
```

### Implementación
```rust
use validator::{Validate, ValidationErrors};

pub struct Validated<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for Validated<Json<T>>
where
    T: DeserializeOwned + Validate + Send,
{
    async fn from_request(req: &mut Request, res: &mut Response) -> Result<Self> {
        // Extract JSON first
        let Json(data) = Json::<T>::from_request(req, res).await?;
        
        // Validate
        data.validate()
            .map_err(|e: ValidationErrors| {
                Error::BadRequest(format!("Validation failed: {}", e))
            })?;
        
        Ok(Validated(Json(data)))
    }
}

// Custom validators
pub mod validators {
    use validator::ValidationError;
    
    pub fn validate_username(username: &str) -> Result<(), ValidationError> {
        if username.len() < 3 {
            return Err(ValidationError::new("username_too_short"));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::new("username_invalid_chars"));
        }
        Ok(())
    }
}
```

---

## 6. File Uploads (10 horas) ⭐⭐⭐⭐

### Ubicación
`src/multipart.rs`

### Dependencias
```toml
multer = "3.0"
tempfile = "3.8"
```

### API Propuesta
```rust
use firework::Multipart;

#[post("/upload")]
async fn upload(mut multipart: Multipart) -> Response {
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("file");
        let filename = field.file_name().map(String::from);
        let data = field.bytes().await?;
        
        // Save file
        tokio::fs::write(format!("uploads/{}", filename.unwrap()), data).await?;
    }
    
    json!({"uploaded": true})
}
```

### Implementación
```rust
use multer::Multipart as MulterMultipart;

pub struct Multipart {
    inner: MulterMultipart<'static>,
}

pub struct Field {
    inner: multer::Field<'static>,
}

impl Field {
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }
    
    pub fn file_name(&self) -> Option<&str> {
        self.inner.file_name()
    }
    
    pub fn content_type(&self) -> Option<&str> {
        self.inner.content_type().map(|m| m.as_ref())
    }
    
    pub async fn bytes(self) -> Result<Vec<u8>> {
        self.inner.bytes().await
            .map(|b| b.to_vec())
            .map_err(|e| Error::BadRequest(format!("Failed to read field: {}", e)))
    }
    
    pub async fn text(self) -> Result<String> {
        self.inner.text().await
            .map_err(|e| Error::BadRequest(format!("Failed to read field: {}", e)))
    }
}

impl Multipart {
    pub async fn next_field(&mut self) -> Result<Option<Field>> {
        self.inner.next_field().await
            .map(|opt| opt.map(|inner| Field { inner }))
            .map_err(|e| Error::BadRequest(format!("Multipart error: {}", e)))
    }
}

#[async_trait::async_trait]
impl FromRequest for Multipart {
    async fn from_request(req: &mut Request, _res: &mut Response) -> Result<Self> {
        let content_type = req.headers
            .get("content-type")
            .and_then(|v| v.first())
            .ok_or_else(|| Error::BadRequest("Missing Content-Type".into()))?;
        
        if !content_type.starts_with("multipart/form-data") {
            return Err(Error::BadRequest("Not multipart/form-data".into()));
        }
        
        let boundary = content_type
            .split("boundary=")
            .nth(1)
            .ok_or_else(|| Error::BadRequest("Missing boundary".into()))?;
        
        let body = std::mem::take(&mut req.body);
        let multipart = MulterMultipart::new(body, boundary);
        
        Ok(Multipart { inner: multipart })
    }
}
```

---

## Integración en el Framework

### Actualizar src/lib.rs
```rust
// Re-export nuevos features
pub use cookie::{Cookie, SameSite};
pub use validation::Validated;
pub use multipart::{Multipart, Field};

// Middleware built-in
pub mod middleware {
    pub use crate::security::security_headers;
}
```

### Actualizar Cargo.toml
```toml
[dependencies]
# Existing...
cookie = "0.18"
validator = { version = "0.18", features = ["derive"] }
multer = "3.0"
tempfile = "3.8"
flate2 = "1.0"
brotli = "3.4"
time = "0.3"
```

---

## Testing

Cada feature debe tener tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use firework::TestClient;
    
    #[tokio::test]
    async fn test_cors() {
        // Test CORS headers
    }
    
    #[tokio::test]
    async fn test_cookies() {
        // Test cookie parsing/setting
    }
    
    #[tokio::test]
    async fn test_compression() {
        // Test gzip/brotli
    }
    
    #[tokio::test]
    async fn test_validation() {
        // Test validation errors
    }
    
    #[tokio::test]
    async fn test_file_upload() {
        // Test multipart upload
    }
}
```

---

## Documentación

Cada plugin necesita:
1. README.md con ejemplos
2. Documentación inline (rustdoc)
3. Ejemplo completo en examples/

---

## Orden de Implementación Recomendado

1. **CORS** (día 1) - Más urgente
2. **Cookie** (día 1-2) - Relativamente simple
3. **Security Headers** (día 2) - Muy simple
4. **Compression** (día 3) - Moderado
5. **Validation** (día 4-5) - Complejo
6. **File Uploads** (día 6-7) - Más complejo

**Total:** ~1 semana de trabajo full-time

