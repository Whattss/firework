# 🔥 Firework Reverse Proxy

Simple and fast reverse proxy for Firework web framework.

## Features

- ⚡ **Fast** - Built on reqwest with connection pooling
- 🎯 **Simple** - Easy to use middleware-based API
- 🔧 **Flexible** - Path prefix matching and stripping
- 🛡️ **Reliable** - Automatic error handling
- 📝 **Headers** - Automatic X-Forwarded-* headers

## Quick Start

```rust
use firework::prelude::*;
use firework_proxy::{ProxyTarget, proxy_to, ProxiedResponse, ProxyFailed};

#[middleware]
async fn proxy(req: &mut Request, res: &mut Response) -> Flow {
    let targets = vec![
        ProxyTarget::new("/", "http://localhost:3001"),
    ];
    proxy_to(req, res, &targets).await
}

#[get("/*")]
async fn catch_all(req: Request) -> Response {
    if let Some(ProxiedResponse(proxied)) = req.get_context::<ProxiedResponse>() {
        return proxied;
    }
    
    if let Some(ProxyFailed(err)) = req.get_context::<ProxyFailed>() {
        eprintln!("Proxy failed: {}", err);
        return Response::new(StatusCode::BadGateway, b"Backend unavailable");
    }
    
    Response::new(StatusCode::NotFound, b"Not found")
}

#[tokio::main]
async fn main() {
    routes!().listen("0.0.0.0:8080").await.unwrap();
}
```

## Examples

### Example 1: Proxy to Next.js

```rust
use firework::prelude::*;
use firework_proxy::*;

// API handled by Firework
#[get("/api/hello")]
async fn api_hello() -> &'static str {
    "Hello from Firework!"
}

// Proxy middleware
#[middleware]
async fn proxy(req: &mut Request, res: &mut Response) -> Flow {
    let targets = vec![
        // Everything except /api goes to Next.js
        ProxyTarget::new("/", "http://localhost:3001"),
    ];
    proxy_to(req, res, &targets).await
}

// Catch-all for proxied responses
#[get("/*")]
async fn frontend(req: Request) -> Response {
    if let Some(response) = get_proxied_response(&req) {
        return response;
    }
    
    if let Some(error) = proxy_failed(&req) {
        eprintln!("[Error] {}", error);
        return Response::new(StatusCode::BadGateway, b"Backend unavailable");
    }
    
    Response::new(StatusCode::NotFound, b"Not found")
}

#[tokio::main]
async fn main() {
    println!("🔥 Firework + Next.js");
    println!("  /api/* → Firework");
    println!("  /*     → Next.js (localhost:3001)");
    println!("");
    println!("Server: http://localhost:8080");
    
    routes!().listen("0.0.0.0:8080").await.unwrap();
}
```

### Example 2: Microservices Gateway

```rust
#[middleware]
async fn proxy(req: &mut Request, res: &mut Response) -> Flow {
    let targets = vec![
        ProxyTarget::new("/users", "http://users-service:3000")
            .strip_prefix(),
        ProxyTarget::new("/posts", "http://posts-service:3001")
            .strip_prefix(),
        ProxyTarget::new("/auth", "http://auth-service:3002")
            .strip_prefix(),
    ];
    proxy_to(req, res, &targets).await
}
```

With `strip_prefix()`:
- Request: `/users/123` → Backend: `http://users-service:3000/123`

Without `strip_prefix()`:
- Request: `/users/123` → Backend: `http://users-service:3000/users/123`

### Example 3: Static Assets + API

```rust
#[middleware]
async fn proxy(req: &mut Request, res: &mut Response) -> Flow {
    let targets = vec![
        // Static files from CDN
        ProxyTarget::new("/static", "https://cdn.example.com"),
        
        // Frontend from Next.js
        ProxyTarget::new("/", "http://localhost:3001"),
    ];
    proxy_to(req, res, &targets).await
}

#[get("/api/status")]
async fn status() -> &'static str {
    "OK"
}

#[get("/*")]
async fn catch_all(req: Request) -> Response {
    if let Some(ProxiedResponse(proxied)) = req.get_context() {
        return proxied;
    }
    Response::new(StatusCode::NotFound, b"Not found")
}
```

## API Reference

### `ProxyTarget`

```rust
pub struct ProxyTarget {
    pub path_prefix: String,
    pub backend_url: String,
    pub strip_prefix: bool,
}

impl ProxyTarget {
    pub fn new(path_prefix: &str, backend_url: &str) -> Self
    pub fn strip_prefix(self) -> Self
}
```

### `proxy_to`

```rust
pub async fn proxy_to(
    req: &mut Request,
    res: &mut Response,
    targets: &[ProxyTarget],
) -> Flow
```

Main proxy function. Matches request against targets and proxies if matched.

### `ProxiedResponse`

```rust
#[derive(Clone)]
pub struct ProxiedResponse(pub Response);
```

Context marker set when a request was successfully proxied.

### `ProxyFailed`

```rust
#[derive(Clone)]
pub struct ProxyFailed(pub String);
```

Context marker set when proxying failed.

### Helper Functions

```rust
pub fn is_proxied(req: &Request) -> bool
pub fn get_proxied_response(req: &Request) -> Option<Response>
pub fn proxy_failed(req: &Request) -> Option<String>
```

## Headers

The proxy automatically adds these headers to backend requests:

- `X-Forwarded-For`: Client IP address
- `X-Real-IP`: Client IP address  
- `X-Forwarded-Proto`: `http` or `https`

All other headers from the original request are copied (except `Host`).

## Error Handling

```rust
#[get("/*")]
async fn handler(req: Request) -> Response {
    // Check if proxied
    if let Some(ProxiedResponse(response)) = req.get_context() {
        return response;
    }
    
    // Check if proxy failed
    if let Some(ProxyFailed(error)) = req.get_context() {
        eprintln!("Proxy error: {}", error);
        
        if error.contains("timeout") {
            return Response::new(StatusCode::GatewayTimeout, b"Timeout");
        }
        
        return Response::new(StatusCode::BadGateway, b"Backend error");
    }
    
    // Not proxied - handle normally
    Response::new(StatusCode::NotFound, b"Not found")
}
```

## Performance

**Overhead:** ~100µs per request

**Throughput:** ~180,000 req/s (on typical hardware)

**Connection Pooling:** Automatic via reqwest

## Limitations

- No load balancing (use multiple Firework instances + load balancer)
- No health checks (backend must be available)
- No circuit breaker (will retry indefinitely)
- HTTP/1.1 only (no HTTP/2 multiplexing yet)

For production with high requirements, consider using Nginx as a reverse proxy in front of Firework.

## License

MIT
