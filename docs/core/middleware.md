# ⚙️ Middleware

Complete guide to Firework's middleware system for request/response processing.

---

## What is Middleware?

Middleware functions process requests **before** they reach handlers and responses **after** handlers execute.

```
Request → Middleware 1 → Middleware 2 → Handler → Middleware 2 → Middleware 1 → Response
```

---

## Basic Middleware

### Simple Middleware

```rust
use firework::prelude::*;

#[middleware]
async fn logger(req: &mut Request, res: &mut Response) -> Flow {
    println!("→ {} {}", format!("{:?}", req.method), req.uri.path);
    Flow::Continue
}
```

**The `#[middleware]` macro auto-registers the middleware globally!**

### Sync vs Async Middleware

```rust
// Sync middleware (for simple operations)
#[middleware]
fn add_header(req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("X-Powered-By".into(), "Firework".into());
    Flow::Continue
}

// Async middleware (for DB, API calls, etc.)
#[middleware]
async fn auth_check(req: &mut Request, res: &mut Response) -> Flow {
    // Can use .await
    let is_valid = validate_token(req).await;
    
    if is_valid {
        Flow::Continue
    } else {
        Flow::Stop(Response::new(
            StatusCode::Unauthorized,
            b"Unauthorized"
        ))
    }
}
```

---

## Flow Control

Middleware returns `Flow` to control execution:

### Flow::Continue

Continue to the next middleware or handler:

```rust
#[middleware]
async fn my_middleware(req: &mut Request, res: &mut Response) -> Flow {
    // Do something...
    Flow::Continue  // Keep going
}
```

### Flow::Stop

Stop processing and return a response immediately:

```rust
#[middleware]
async fn rate_limit(req: &mut Request, res: &mut Response) -> Flow {
    if is_rate_limited(req).await {
        return Flow::Stop(Response::new(
            StatusCode::TooManyRequests,
            b"Rate limit exceeded"
        ));
    }
    
    Flow::Continue
}
```

---

## Common Middleware Patterns

### 1. Logging

```rust
#[middleware]
async fn request_logger(req: &mut Request, res: &mut Response) -> Flow {
    let start = std::time::Instant::now();
    let method = format!("{:?}", req.method);
    let path = req.uri.path.clone();
    
    println!("→ {} {}", method, path);
    
    // After handler executes, this continues...
    Flow::Continue
}

// Advanced logging with timing
#[middleware]
async fn detailed_logger(req: &mut Request, res: &mut Response) -> Flow {
    use std::time::Instant;
    
    let start = Instant::now();
    let method = format!("{:?}", req.method);
    let path = req.uri.path.clone();
    
    // Store start time in context
    req.set_context(start);
    
    println!("→ {} {}", method, path);
    
    Flow::Continue
    
    // Note: To log response time, you'd need post-middleware
}
```

### 2. CORS

```rust
#[middleware]
async fn cors(req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("Access-Control-Allow-Origin".into(), "*".into());
    res.headers.insert(
        "Access-Control-Allow-Methods".into(),
        "GET, POST, PUT, DELETE, OPTIONS".into()
    );
    res.headers.insert(
        "Access-Control-Allow-Headers".into(),
        "Content-Type, Authorization".into()
    );
    
    // Handle preflight
    if matches!(req.method, Method::OPTIONS) {
        return Flow::Stop(Response::new(StatusCode::NoContent, b""));
    }
    
    Flow::Continue
}
```

### 3. Authentication

```rust
#[middleware]
async fn require_auth(req: &mut Request, res: &mut Response) -> Flow {
    // Extract token from header
    let token = match req.header("Authorization") {
        Some(auth) => {
            if let Some(token) = auth.strip_prefix("Bearer ") {
                token
            } else {
                return Flow::Stop(error_response(
                    StatusCode::Unauthorized,
                    "Invalid authorization format"
                ));
            }
        }
        None => {
            return Flow::Stop(error_response(
                StatusCode::Unauthorized,
                "No authorization token"
            ));
        }
    };
    
    // Verify token
    match verify_jwt(token).await {
        Ok(claims) => {
            // Store user info in request context
            req.set_context(claims);
            Flow::Continue
        }
        Err(_) => {
            Flow::Stop(error_response(
                StatusCode::Unauthorized,
                "Invalid token"
            ))
        }
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    let body = serde_json::json!({
        "error": message,
        "status": status.code()
    });
    Response::new(status, serde_json::to_vec(&body).unwrap())
        .with_header("Content-Type", "application/json")
}
```

### 4. Request ID Tracking

```rust
use uuid::Uuid;

#[middleware]
async fn request_id(req: &mut Request, res: &mut Response) -> Flow {
    // Generate or extract request ID
    let req_id = req.header("X-Request-ID")
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Store in context
    req.set_context(RequestId(req_id.clone()));
    
    // Add to response
    res.headers.insert("X-Request-ID".into(), req_id);
    
    Flow::Continue
}

#[derive(Clone)]
struct RequestId(String);
```

### 5. Security Headers

```rust
#[middleware]
async fn security_headers(req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("X-Content-Type-Options".into(), "nosniff".into());
    res.headers.insert("X-Frame-Options".into(), "DENY".into());
    res.headers.insert("X-XSS-Protection".into(), "1; mode=block".into());
    res.headers.insert(
        "Strict-Transport-Security".into(),
        "max-age=31536000; includeSubDomains".into()
    );
    
    Flow::Continue
}
```

### 6. Rate Limiting

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

lazy_static::lazy_static! {
    static ref RATE_LIMITER: Arc<RwLock<HashMap<String, (u32, Instant)>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[middleware]
async fn rate_limit(req: &mut Request, res: &mut Response) -> Flow {
    const MAX_REQUESTS: u32 = 100;
    const WINDOW: Duration = Duration::from_secs(60);
    
    // Get client IP
    let client_ip = req.remote_addr
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let mut limiter = RATE_LIMITER.write().await;
    
    let now = Instant::now();
    let (count, window_start) = limiter
        .entry(client_ip.clone())
        .or_insert((0, now));
    
    // Reset if window expired
    if now.duration_since(*window_start) > WINDOW {
        *count = 0;
        *window_start = now;
    }
    
    *count += 1;
    
    if *count > MAX_REQUESTS {
        return Flow::Stop(Response::new(
            StatusCode::TooManyRequests,
            b"Rate limit exceeded"
        ));
    }
    
    Flow::Continue
}
```

### 7. Request Validation

```rust
#[middleware]
async fn validate_content_type(req: &mut Request, res: &mut Response) -> Flow {
    // Only validate POST/PUT/PATCH
    if !matches!(req.method, Method::POST | Method::PUT | Method::PATCH) {
        return Flow::Continue;
    }
    
    let content_type = req.header("Content-Type").unwrap_or("");
    
    if !content_type.contains("application/json") {
        return Flow::Stop(Response::new(
            StatusCode::BadRequest,
            b"{\"error\":\"Content-Type must be application/json\"}"
        ).with_header("Content-Type", "application/json"));
    }
    
    Flow::Continue
}
```

---

## Middleware Phases

### Pre-Middleware (Default)

Executes **before** the handler:

```rust
#[middleware]
async fn pre_middleware(req: &mut Request, res: &mut Response) -> Flow {
    println!("Before handler");
    Flow::Continue
}
```

### Post-Middleware

Executes **after** the handler:

```rust
#[middleware(post)]
async fn post_middleware(req: &mut Request, res: &mut Response) -> Flow {
    println!("After handler");
    
    // Can modify response
    res.headers.insert("X-Processing-Time".into(), "100ms".into());
    
    Flow::Continue
}
```

---

## Scope-Level Middleware

Apply middleware to specific routes:

```rust
#[middleware]
async fn require_api_key(req: &mut Request, res: &mut Response) -> Flow {
    match req.header("X-API-Key") {
        Some(key) if key == "secret" => Flow::Continue,
        _ => Flow::Stop(Response::new(
            StatusCode::Unauthorized,
            b"Invalid API key"
        ))
    }
}

#[scope("/api", middleware = [require_api_key])]
mod api {
    use super::*;
    
    #[get("/data")]
    async fn data() -> &'static str {
        "Secret data"
    }
}
```

### Multiple Middleware on Scope

```rust
#[middleware]
async fn mw1(req: &mut Request, res: &mut Response) -> Flow {
    Flow::Continue
}

#[middleware]
async fn mw2(req: &mut Request, res: &mut Response) -> Flow {
    Flow::Continue
}

#[scope("/admin", middleware = [mw1, mw2])]
mod admin {
    // Both middleware apply to all routes here
}
```

---

## Request Context

Share data between middleware and handlers:

```rust
#[derive(Clone)]
struct User {
    id: u32,
    username: String,
}

#[middleware]
async fn auth(req: &mut Request, res: &mut Response) -> Flow {
    // Authenticate user
    let user = User {
        id: 1,
        username: "john".into(),
    };
    
    // Store in context
    req.set_context(user);
    
    Flow::Continue
}

#[get("/profile")]
async fn profile(req: Request) -> String {
    // Retrieve from context
    let user = req.get_context::<User>().unwrap();
    format!("Username: {}", user.username)
}
```

---

## Conditional Middleware

Apply middleware based on conditions:

```rust
#[middleware]
async fn conditional_logging(req: &mut Request, res: &mut Response) -> Flow {
    // Only log API routes
    if req.uri.path.starts_with("/api") {
        println!("API call: {}", req.uri.path);
    }
    
    Flow::Continue
}

#[middleware]
async fn dev_only(req: &mut Request, res: &mut Response) -> Flow {
    // Only in development
    #[cfg(debug_assertions)]
    {
        println!("Debug mode enabled");
    }
    
    Flow::Continue
}
```

---

## Error Handling in Middleware

```rust
#[middleware]
async fn error_handler(req: &mut Request, res: &mut Response) -> Flow {
    // Try to process request
    match process_request(req).await {
        Ok(_) => Flow::Continue,
        Err(e) => {
            eprintln!("Middleware error: {}", e);
            
            Flow::Stop(Response::new(
                StatusCode::InternalServerError,
                b"Internal server error"
            ))
        }
    }
}

async fn process_request(req: &Request) -> Result<(), Box<dyn std::error::Error>> {
    // Processing logic that might fail
    Ok(())
}
```

---

## Performance Optimization

### Efficient Middleware

```rust
// ❌ BAD - Cloning unnecessarily
#[middleware]
async fn bad_middleware(req: &mut Request, res: &mut Response) -> Flow {
    let path = req.uri.path.clone();  // Unnecessary clone
    println!("{}", path);
    Flow::Continue
}

// ✅ GOOD - Using references
#[middleware]
async fn good_middleware(req: &mut Request, res: &mut Response) -> Flow {
    println!("{}", req.uri.path);  // No clone
    Flow::Continue
}
```

### Lazy Evaluation

```rust
#[middleware]
async fn lazy_check(req: &mut Request, res: &mut Response) -> Flow {
    // Early return if not needed
    if !req.uri.path.starts_with("/api") {
        return Flow::Continue;
    }
    
    // Expensive operation only for /api routes
    expensive_validation(req).await;
    
    Flow::Continue
}
```

---

## Testing Middleware

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cors_middleware() {
        let mut req = Request::new(
            Method::GET,
            Uri::new("/test", None),
            Version::Http11,
            HashMap::new(),
            vec![],
            None,
        );
        let mut res = Response::default();
        
        let result = cors(&mut req, &mut res).await;
        
        assert!(matches!(result, Flow::Continue));
        assert_eq!(
            res.headers.get("Access-Control-Allow-Origin"),
            Some(&"*".to_string())
        );
    }
}
```

---

## Best Practices

1. **Keep middleware fast** - They run on every request
2. **Use early returns** - Stop processing ASAP
3. **Avoid heavy computations** - Offload to handlers
4. **Use context wisely** - Don't over-store data
5. **Order matters** - Auth before rate limiting
6. **Be defensive** - Handle missing headers gracefully

---

## Common Middleware Stack

Recommended order:

```rust
// 1. Request ID (first)
#[middleware]
async fn request_id(req: &mut Request, res: &mut Response) -> Flow { /* ... */ }

// 2. Logging
#[middleware]
async fn logger(req: &mut Request, res: &mut Response) -> Flow { /* ... */ }

// 3. Security headers
#[middleware]
async fn security_headers(req: &mut Request, res: &mut Response) -> Flow { /* ... */ }

// 4. CORS
#[middleware]
async fn cors(req: &mut Request, res: &mut Response) -> Flow { /* ... */ }

// 5. Rate limiting
#[middleware]
async fn rate_limit(req: &mut Request, res: &mut Response) -> Flow { /* ... */ }

// 6. Authentication
#[middleware]
async fn auth(req: &mut Request, res: &mut Response) -> Flow { /* ... */ }

// 7. Authorization
#[middleware]
async fn authorize(req: &mut Request, res: &mut Response) -> Flow { /* ... */ }
```

---

## Next Steps

- [Request & Response](./request-response.md) - Deep dive
- [Extractors](./extractors.md) - Type-safe data extraction
- [Error Handling](./errors.md) - Comprehensive error handling
