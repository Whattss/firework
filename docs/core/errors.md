# ⚠️ Error Handling

Comprehensive error handling in Firework.

---

## Error Types

Firework provides 15+ built-in error types:

```rust
pub enum Error {
    BadRequest(String),           // 400
    Unauthorized(String),          // 401
    Forbidden(String),             // 403
    NotFound(String),              // 404
    MethodNotAllowed(String),      // 405
    NotAcceptable(String),         // 406
    RequestTimeout(String),        // 408
    Conflict(String),              // 409
    Gone(String),                  // 410
    PayloadTooLarge(String),       // 413
    UriTooLong(String),            // 414
    UnprocessableEntity(String),   // 422
    TooManyRequests(String),       // 429
    InternalServerError(String),   // 500
    ServiceUnavailable(String),    // 503
    GatewayTimeout(String),        // 504
    Custom(String),                // Custom error
    CustomWithCode(u16, String),   // Custom with status code
}
```

---

## Using Errors

### Basic Error Handling

```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, Error> {
    if id == 0 {
        return Err(Error::BadRequest("ID cannot be zero".into()));
    }
    
    let user = fetch_user(id).await
        .ok_or_else(|| Error::NotFound("User not found".into()))?;
    
    Ok(Json(user))
}
```

**Automatic JSON response:**
```json
{
  "error": "User not found",
  "status": 404
}
```

### All Error Types Example

```rust
// 400 - Bad input
Err(Error::BadRequest("Invalid email format".into()))

// 401 - Not authenticated
Err(Error::Unauthorized("Login required".into()))

// 403 - Not allowed
Err(Error::Forbidden("Admin access required".into()))

// 404 - Not found
Err(Error::NotFound("Resource not found".into()))

// 409 - Conflict
Err(Error::Conflict("Username already exists".into()))

// 413 - Too large
Err(Error::PayloadTooLarge("File exceeds 10MB limit".into()))

// 422 - Validation failed
Err(Error::UnprocessableEntity("Validation errors".into()))

// 429 - Rate limited
Err(Error::TooManyRequests("Try again in 60 seconds".into()))

// 500 - Server error
Err(Error::InternalServerError("Database connection failed".into()))

// 503 - Service down
Err(Error::ServiceUnavailable("Maintenance mode".into()))
```

---

## Custom Errors

```rust
// Custom message
Err(Error::Custom("Something went wrong".into()))

// Custom status code
Err(Error::CustomWithCode(451, "Unavailable for legal reasons".into()))
```

---

## Converting External Errors

```rust
// From Result<T, E>
let data = some_operation()
    .map_err(|e| Error::Internal(e.to_string()))?;

// From Option
let user = find_user(id)
    .ok_or_else(|| Error::NotFound("User not found".into()))?;
```

---

## Error Responses

Errors automatically convert to JSON responses:

```rust
impl Error {
    pub fn into_response(self) -> Response {
        let (status, message) = match self {
            Error::NotFound(msg) => (StatusCode::NotFound, msg),
            Error::Unauthorized(msg) => (StatusCode::Unauthorized, msg),
            // ... all variants
        };
        
        let body = json!({
            "error": message,
            "status": status.code()
        });
        
        Response::new(status, serde_json::to_vec(&body).unwrap())
            .with_header("Content-Type", "application/json")
    }
}
```

---

## Best Practices

1. **Use specific error types** - `NotFound` over `Custom`
2. **Include context** - "User 123 not found" over "Not found"
3. **Return Result<T, Error>** - Automatic handling
4. **Don't expose internals** - No stack traces to users
5. **Log server errors** - Use eprintln! or logging crate

---

## Error Middleware

```rust
#[middleware(post)]
async fn error_logger(req: &mut Request, res: &mut Response) -> Flow {
    if res.status.code() >= 500 {
        eprintln!("Server error on {}: {:?}", req.uri.path, res.status);
    }
    Flow::Continue
}
```
