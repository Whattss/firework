# 🔒 CORS & Security

Security best practices for Firework applications.

---

## CORS Middleware

```rust
#[middleware]
async fn cors(_req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("Access-Control-Allow-Origin".into(), "*".into());
    res.headers.insert(
        "Access-Control-Allow-Methods".into(),
        "GET, POST, PUT, DELETE, OPTIONS".into()
    );
    res.headers.insert(
        "Access-Control-Allow-Headers".into(),
        "Content-Type, Authorization".into()
    );
    res.headers.insert(
        "Access-Control-Max-Age".into(),
        "86400".into()
    );
    
    Flow::Continue
}
```

---

## Security Headers

```rust
#[middleware]
async fn security_headers(_req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("X-Content-Type-Options".into(), "nosniff".into());
    res.headers.insert("X-Frame-Options".into(), "DENY".into());
    res.headers.insert("X-XSS-Protection".into(), "1; mode=block".into());
    res.headers.insert(
        "Strict-Transport-Security".into(),
        "max-age=31536000".into()
    );
    res.headers.insert(
        "Content-Security-Policy".into(),
        "default-src 'self'".into()
    );
    
    Flow::Continue
}
```

---

## Rate Limiting

See [Middleware Guide](../core/middleware.md#6-rate-limiting)

---

## Input Validation

```rust
#[post("/users")]
async fn create_user(Json(user): Json<CreateUser>) -> Result<Json<User>, Error> {
    // Validate username
    if user.username.len() < 3 {
        return Err(Error::BadRequest("Username too short".into()));
    }
    
    // Validate email
    if !user.email.contains('@') {
        return Err(Error::BadRequest("Invalid email".into()));
    }
    
    // Sanitize inputs
    let username = sanitize_string(&user.username);
    
    Ok(Json(User { /* ... */ }))
}

fn sanitize_string(s: &str) -> String {
    s.trim().to_string()
}
```
