# Firework Security Plugin

HTTP security headers middleware for Firework. Protects your application against common web vulnerabilities like clickjacking, XSS, MIME sniffing, and more.

## Features

- **X-Frame-Options** - Prevents clickjacking attacks by controlling iframe embedding
- **X-Content-Type-Options** - Prevents MIME type sniffing
- **X-XSS-Protection** - Enables browser XSS filtering
- **Strict-Transport-Security (HSTS)** - Forces HTTPS connections
- **Content-Security-Policy (CSP)** - Controls resource loading and execution
- **Referrer-Policy** - Controls referrer information in requests
- **Permissions-Policy** - Controls browser features and APIs
- Three presets: Default, Strict, Relaxed
- Fully customizable per-header configuration

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
firework-security = { git = "https://github.com/Whattss/firework" }
```

## Quick Start

### Default Configuration (Recommended)

```rust
use firework::prelude::*;
use firework_security::SecurityHeadersPlugin;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Register with default secure settings
    firework::register_plugin(Arc::new(SecurityHeadersPlugin::default()));
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

This adds the following headers to all responses:

```http
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000
Content-Security-Policy: default-src 'self'
Referrer-Policy: no-referrer
```

## Presets

### Default - Balanced Security

Good for most applications. Reasonable restrictions with good compatibility.

```rust
let plugin = Arc::new(SecurityHeadersPlugin::default());
firework::register_plugin(plugin);
```

Headers:
- X-Frame-Options: DENY
- HSTS: 1 year (31536000 seconds)
- CSP: `default-src 'self'`
- Referrer: no-referrer

### Strict - Maximum Security

For high-security applications. Very restrictive CSP and permissions.

```rust
let plugin = Arc::new(SecurityHeadersPlugin::strict());
firework::register_plugin(plugin);
```

Headers:
- X-Frame-Options: DENY
- HSTS: 2 years (63072000 seconds)
- CSP: `default-src 'none'; script-src 'self'; connect-src 'self'; img-src 'self'; style-src 'self'`
- Referrer: no-referrer
- Permissions: `geolocation=(), microphone=(), camera=()`

### Relaxed - Development Friendly

For development or legacy applications. Looser restrictions.

```rust
let plugin = Arc::new(SecurityHeadersPlugin::relaxed());
firework::register_plugin(plugin);
```

Headers:
- X-Frame-Options: SAMEORIGIN (allows same-origin iframes)
- HSTS: disabled
- CSP: `default-src 'self' 'unsafe-inline' 'unsafe-eval'` (allows inline scripts)
- Referrer: no-referrer-when-downgrade

## Security Headers Explained

### X-Frame-Options

Prevents your site from being embedded in iframes (clickjacking protection).

**Options:**
- `DENY` - Never allow framing (most secure)
- `SAMEORIGIN` - Allow framing only from same origin
- `ALLOW-FROM https://trusted.com` - Allow specific origin (deprecated, use CSP instead)

```rust
// Completely prevent framing
let plugin = SecurityHeadersPlugin::default()
    .frame_options("DENY");

// Allow same-origin framing
let plugin = SecurityHeadersPlugin::default()
    .frame_options("SAMEORIGIN");

// Disable (not recommended)
let plugin = SecurityHeadersPlugin::default()
    .no_frame_options();
```

### X-Content-Type-Options

Prevents browsers from MIME-sniffing responses away from the declared content-type.

```rust
// Enable (recommended)
let plugin = SecurityHeadersPlugin::default()
    .content_type_nosniff(true);

// Disable
let plugin = SecurityHeadersPlugin::default()
    .content_type_nosniff(false);
```

Always set to `nosniff` to prevent security issues where browsers might interpret JSON as HTML.

### X-XSS-Protection

Enables the browser's built-in XSS filter (legacy, but still useful for older browsers).

```rust
// Enable (recommended for older browsers)
let plugin = SecurityHeadersPlugin::default()
    .xss_protection(true);

// Disable (if you have strong CSP)
let plugin = SecurityHeadersPlugin::default()
    .xss_protection(false);
```

Modern browsers rely on CSP instead, but this provides defense-in-depth.

### Strict-Transport-Security (HSTS)

Forces browsers to always use HTTPS for your domain.

```rust
// 1 year (recommended minimum)
let plugin = SecurityHeadersPlugin::default()
    .hsts(31536000);

// 2 years (strict security)
let plugin = SecurityHeadersPlugin::default()
    .hsts(63072000);

// Disable for development
let plugin = SecurityHeadersPlugin::default()
    .no_hsts();
```

**Important:** Only enable HSTS if you have a valid SSL certificate and HTTPS is fully configured.

### Content-Security-Policy (CSP)

The most powerful security header. Controls what resources can be loaded and executed.

```rust
// Strict: Only allow resources from your domain
let plugin = SecurityHeadersPlugin::default()
    .csp("default-src 'self'");

// Allow specific CDNs
let plugin = SecurityHeadersPlugin::default()
    .csp("default-src 'self'; script-src 'self' https://cdn.jsdelivr.net; style-src 'self' https://fonts.googleapis.com");

// Allow inline scripts (less secure, for legacy apps)
let plugin = SecurityHeadersPlugin::default()
    .csp("default-src 'self' 'unsafe-inline' 'unsafe-eval'");

// Disable CSP
let plugin = SecurityHeadersPlugin::default()
    .no_csp();
```

**CSP Directives:**
- `default-src` - Fallback for all resource types
- `script-src` - JavaScript sources
- `style-src` - CSS sources
- `img-src` - Image sources
- `font-src` - Font sources
- `connect-src` - AJAX, WebSocket, fetch() sources
- `frame-src` - iframe sources
- `media-src` - Audio/video sources

**Special Values:**
- `'self'` - Same origin as the document
- `'none'` - Block everything
- `'unsafe-inline'` - Allow inline scripts/styles (not recommended)
- `'unsafe-eval'` - Allow eval() (not recommended)
- `https:` - Allow any HTTPS resource
- `data:` - Allow data: URIs

### Referrer-Policy

Controls how much referrer information is sent with requests.

```rust
// No referrer (most private)
let plugin = SecurityHeadersPlugin::default()
    .referrer_policy("no-referrer");

// Referrer only on same origin
let plugin = SecurityHeadersPlugin::default()
    .referrer_policy("same-origin");

// Full URL only on HTTPS
let plugin = SecurityHeadersPlugin::default()
    .referrer_policy("strict-origin-when-cross-origin");
```

**Options:**
- `no-referrer` - Never send referrer (most private)
- `no-referrer-when-downgrade` - Don't send on HTTPS→HTTP
- `origin` - Send only origin (not full URL)
- `origin-when-cross-origin` - Full URL for same-origin, origin for cross-origin
- `same-origin` - Only for same-origin requests
- `strict-origin` - Origin only, no downgrade
- `strict-origin-when-cross-origin` - Recommended default
- `unsafe-url` - Always send full URL (not recommended)

### Permissions-Policy

Controls which browser features and APIs can be used.

```rust
// Block geolocation, camera, microphone
let plugin = SecurityHeadersPlugin::default()
    .permissions_policy("geolocation=(), microphone=(), camera=()");

// Allow geolocation for self only
let plugin = SecurityHeadersPlugin::default()
    .permissions_policy("geolocation=(self)");

// Complex policy
let plugin = SecurityHeadersPlugin::default()
    .permissions_policy("geolocation=(self), microphone=(), camera=(), payment=(self 'https://trusted.com')");
```

**Common Features:**
- `geolocation` - Geolocation API
- `camera` - Camera access
- `microphone` - Microphone access
- `payment` - Payment Request API
- `usb` - WebUSB API
- `fullscreen` - Fullscreen API
- `accelerometer` - Accelerometer
- `gyroscope` - Gyroscope

## Custom Configuration

### Builder Pattern

Chain methods to customize your security policy:

```rust
use firework_security::SecurityHeadersPlugin;

let plugin = SecurityHeadersPlugin::default()
    .frame_options("SAMEORIGIN")
    .hsts(63072000)  // 2 years
    .csp("default-src 'self'; img-src 'self' https://images.example.com")
    .referrer_policy("strict-origin-when-cross-origin")
    .permissions_policy("geolocation=(), camera=(), microphone=()");

firework::register_plugin(Arc::new(plugin));
```

### Configuration Object

Use a configuration struct for more complex setups:

```rust
use firework_security::{SecurityHeadersPlugin, SecurityConfig};

let config = SecurityConfig {
    frame_options: Some("DENY".to_string()),
    content_type_nosniff: true,
    xss_protection: true,
    hsts_max_age: Some(31536000),
    csp: Some("default-src 'self'; script-src 'self' 'unsafe-inline'".to_string()),
    referrer_policy: Some("no-referrer".to_string()),
    permissions_policy: Some("geolocation=()".to_string()),
};

let plugin = SecurityHeadersPlugin::with_config(config);
firework::register_plugin(Arc::new(plugin));
```

### Load from Config File

```rust
// In firework.config.toml:
[plugins.security]
frame_options = "DENY"
content_type_nosniff = true
xss_protection = true
hsts_max_age = 31536000
csp = "default-src 'self'"
referrer_policy = "no-referrer"
permissions_policy = "geolocation=()"

// In code:
let plugin = Arc::new(SecurityHeadersPlugin::from_config().await);
firework::register_plugin(plugin);
```

## Real-World Examples

### Standard Web Application

```rust
let plugin = SecurityHeadersPlugin::default()
    .frame_options("DENY")
    .hsts(31536000)
    .csp("default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'")
    .referrer_policy("strict-origin-when-cross-origin");

firework::register_plugin(Arc::new(plugin));
```

### API Server (No Browser Features)

```rust
let plugin = SecurityHeadersPlugin::strict()
    .csp("default-src 'none'")  // API doesn't load resources
    .permissions_policy("geolocation=(), camera=(), microphone=(), payment=()");

firework::register_plugin(Arc::new(plugin));
```

### App with CDN Resources

```rust
let plugin = SecurityHeadersPlugin::default()
    .csp(concat!(
        "default-src 'self'; ",
        "script-src 'self' https://cdn.jsdelivr.net https://unpkg.com; ",
        "style-src 'self' https://fonts.googleapis.com 'unsafe-inline'; ",
        "font-src 'self' https://fonts.gstatic.com; ",
        "img-src 'self' data: https://images.example.com; ",
        "connect-src 'self' https://api.example.com"
    ));

firework::register_plugin(Arc::new(plugin));
```

### Embeddable Widget

```rust
let plugin = SecurityHeadersPlugin::default()
    .frame_options("SAMEORIGIN")  // Allow embedding in your other sites
    .csp("default-src 'self'; frame-ancestors 'self' https://trusted-partner.com");

firework::register_plugin(Arc::new(plugin));
```

### Development Environment

```rust
#[cfg(debug_assertions)]
let plugin = SecurityHeadersPlugin::relaxed()
    .no_hsts()  // Don't force HTTPS in dev
    .csp("default-src 'self' 'unsafe-inline' 'unsafe-eval'");  // Allow hot reload

#[cfg(not(debug_assertions))]
let plugin = SecurityHeadersPlugin::strict();

firework::register_plugin(Arc::new(plugin));
```

## Testing Security Headers

Use curl to verify your security headers:

```bash
# Check all security headers
curl -I http://localhost:8080

# Expected output:
HTTP/1.1 200 OK
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000
Content-Security-Policy: default-src 'self'
Referrer-Policy: no-referrer
```

Automated testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use firework::test::TestClient;
    
    #[tokio::test]
    async fn test_security_headers() {
        let client = TestClient::new(routes!());
        let res = client.get("/").send().await;
        
        assert_eq!(res.header("X-Frame-Options"), Some("DENY"));
        assert_eq!(res.header("X-Content-Type-Options"), Some("nosniff"));
        assert_eq!(res.header("X-XSS-Protection"), Some("1; mode=block"));
        assert!(res.header("Strict-Transport-Security").is_some());
        assert!(res.header("Content-Security-Policy").is_some());
    }
}
```

## Best Practices

### 1. Start with Defaults, Then Customize

```rust
// Start here
let plugin = SecurityHeadersPlugin::default();

// Then adjust as needed
let plugin = plugin
    .csp("default-src 'self'; script-src 'self' https://trusted-cdn.com");
```

### 2. Use CSP Report-Only Mode First

Test your CSP without blocking resources:

```rust
// Instead of Content-Security-Policy, use Content-Security-Policy-Report-Only
// This logs violations without blocking
let plugin = SecurityHeadersPlugin::default()
    .csp("default-src 'self'; report-uri /csp-report");

// Create an endpoint to receive reports
#[post("/csp-report")]
async fn csp_report(Json(report): Json<serde_json::Value>) -> Response {
    eprintln!("CSP Violation: {:?}", report);
    Response::new(StatusCode::NoContent, vec![])
}
```

### 3. Gradually Tighten Security

Don't go straight to `strict()`. Iterate:

```rust
// Week 1: Relaxed with monitoring
let plugin = SecurityHeadersPlugin::relaxed();

// Week 2: Default after fixing issues
let plugin = SecurityHeadersPlugin::default();

// Week 3: Custom stricter policy
let plugin = SecurityHeadersPlugin::default()
    .csp("default-src 'self'; script-src 'self'");

// Production: Strict
let plugin = SecurityHeadersPlugin::strict();
```

### 4. Environment-Specific Configs

```rust
use std::env;

let plugin = match env::var("ENVIRONMENT").as_deref() {
    Ok("development") => SecurityHeadersPlugin::relaxed().no_hsts(),
    Ok("staging") => SecurityHeadersPlugin::default(),
    Ok("production") => SecurityHeadersPlugin::strict(),
    _ => SecurityHeadersPlugin::default(),
};

firework::register_plugin(Arc::new(plugin));
```

### 5. Monitor CSP Violations

```rust
let plugin = SecurityHeadersPlugin::default()
    .csp("default-src 'self'; report-uri /api/csp-violations");

#[post("/api/csp-violations")]
async fn handle_csp_violation(
    Json(report): Json<serde_json::Value>
) -> Response {
    // Log to monitoring service
    log::warn!("CSP violation: {:?}", report);
    
    // Could send to Sentry, DataDog, etc.
    // sentry::capture_message(&format!("CSP violation: {:?}", report));
    
    Response::new(StatusCode::NoContent, vec![])
}
```

## Common Issues

### CSP Blocks Inline Scripts

**Problem:** Your inline `<script>` tags don't work.

**Solutions:**

1. Move scripts to external files:
```html
<!-- Before -->
<script>console.log('hello')</script>

<!-- After -->
<script src="/js/app.js"></script>
```

2. Use nonces (recommended):
```rust
// Generate nonce per request
let nonce = generate_nonce();
let plugin = plugin.csp(&format!("script-src 'self' 'nonce-{}'", nonce));

// In HTML
<script nonce="{nonce}">console.log('hello')</script>
```

3. Allow unsafe-inline (not recommended):
```rust
let plugin = plugin.csp("default-src 'self'; script-src 'self' 'unsafe-inline'");
```

### HSTS Forces HTTPS in Development

**Problem:** Can't test on `http://localhost` because HSTS forces HTTPS.

**Solution:** Disable HSTS in development:

```rust
#[cfg(debug_assertions)]
let plugin = SecurityHeadersPlugin::default().no_hsts();

#[cfg(not(debug_assertions))]
let plugin = SecurityHeadersPlugin::default().hsts(31536000);
```

### Can't Embed YouTube Videos

**Problem:** CSP blocks iframe embeds.

**Solution:** Allow trusted frame sources:

```rust
let plugin = plugin.csp(
    "default-src 'self'; frame-src 'self' https://www.youtube.com https://www.youtube-nocookie.com"
);
```

### Images from CDN Don't Load

**Problem:** CSP blocks images from external domains.

**Solution:** Whitelist your CDN:

```rust
let plugin = plugin.csp(
    "default-src 'self'; img-src 'self' https://cdn.example.com https://images.example.com data:"
);
```

## Security Checklist

- [ ] Use HTTPS in production (required for HSTS)
- [ ] Enable HSTS with at least 1 year (`max-age=31536000`)
- [ ] Use strict CSP (`default-src 'self'` or stricter)
- [ ] Set `X-Frame-Options: DENY` unless you need iframe embedding
- [ ] Enable `X-Content-Type-Options: nosniff`
- [ ] Set `Referrer-Policy: no-referrer` or `strict-origin-when-cross-origin`
- [ ] Block unnecessary browser features with `Permissions-Policy`
- [ ] Test with CSP Report-Only mode before enforcing
- [ ] Monitor CSP violations in production
- [ ] Use environment-specific security configs
- [ ] Never use `'unsafe-eval'` in production
- [ ] Minimize use of `'unsafe-inline'` (use nonces instead)
- [ ] Regularly update security policies as your app evolves

## Additional Resources

- [OWASP Secure Headers Project](https://owasp.org/www-project-secure-headers/)
- [MDN: Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [CSP Evaluator Tool](https://csp-evaluator.withgoogle.com/)
- [securityheaders.com](https://securityheaders.com/) - Test your headers
- [Mozilla Observatory](https://observatory.mozilla.org/) - Security scanner

## License

Same as Firework framework.
