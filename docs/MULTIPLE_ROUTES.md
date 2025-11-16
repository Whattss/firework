# Multiple Routes per Handler

**Register multiple routes** that point to the same handler function.

## Usage

Simply stack multiple route macros on the same function:

```rust
#[get("/health")]
#[get("/api/health")]
#[get("/status")]
async fn health_check() -> Json<Value> {
    json!({"status": "ok"})
}
```

All three routes (`/health`, `/api/health`, `/status`) will call the same `health_check` function!

## Use Cases

### 1. **API Aliases**

Provide multiple entry points for the same functionality:

```rust
#[get("/login")]
#[get("/signin")]
#[get("/auth/login")]
async fn login_page() -> Html {
    Html("<h1>Login</h1>")
}
```

### 2. **API Versioning**

Support multiple API versions:

```rust
#[get("/api/v1/users")]
#[get("/api/v2/users")]
async fn list_users() -> Json<Vec<User>> {
    // Same implementation for both versions
    Json(get_users().await)
}
```

### 3. **Legacy Support**

Maintain old routes while migrating:

```rust
#[get("/tweets")]           // New clean route
#[get("/api/tweets")]       // Old API route
#[get("/v1/feed/tweets")]   // Very old route
async fn get_tweets() -> Json<Vec<Tweet>> {
    Json(fetch_tweets().await)
}
```

### 4. **Different Methods, Same Handler**

Handle multiple HTTP methods:

```rust
#[get("/test")]
#[post("/test")]
async fn test_endpoint() -> &'static str {
    "Works for both GET and POST"
}
```

### 5. **Internationalization**

Different language paths:

```rust
#[get("/en/about")]
#[get("/es/acerca")]
#[get("/fr/apropos")]
async fn about_page() -> Html {
    Html("<h1>About Us</h1>")
}
```

## How It Works

Each route macro generates:
1. A unique wrapper function (one per route)
2. A unique static registration (one per route)
3. **Only one** copy of your handler function

The wrappers all call the same underlying handler, so there's **zero code duplication**.

## Performance

**Zero overhead!**

- Each route has its own matcher (for fast routing)
- All routes call the same handler function
- No runtime checks or conditionals
- Compiles to the same code as manual route duplication

## Advanced Examples

### Health Endpoints with Multiple Formats

```rust
#[get("/health")]
#[get("/health.json")]
#[get("/api/health")]
async fn health_check(req: &Request) -> Response {
    let data = json!({"status": "ok", "uptime": get_uptime()});
    
    // Check path to determine format
    if req.uri().path().ends_with(".json") || req.uri().path().starts_with("/api") {
        Json(data).into_response()
    } else {
        Html(format!("<h1>Status: OK</h1>")).into_response()
    }
}
```

### Redirect Old Routes

```rust
#[get("/old-blog")]
#[get("/blog-old")]
#[get("/legacy/blog")]
async fn old_blog_redirect() -> Redirect {
    Redirect::permanent("/blog")
}
```

### Resource with Multiple ID Formats

```rust
#[get("/users/:id")]
#[get("/user/:id")]
#[get("/profile/:id")]
async fn get_user_profile(
    DbEntity(user): DbEntity<users::Model>
) -> Json<users::Model> {
    Json(user)
}
```

## Limitations

1. **Path parameters must match**: You can't use different parameter names across routes

```rust
// ❌ This won't work - different param names
#[get("/users/:id")]
#[get("/profiles/:user_id")]  // Different param name!
async fn get_user(Path(id): Path<i32>) -> Json<User>
```

2. **Must be same HTTP method or use different macros**:

```rust
// ✅ This works
#[get("/test")]
#[post("/test")]
async fn handler() -> &'static str { "ok" }

// ❌ This doesn't compile
#[get("/test")]
#[get("/test")]  // Duplicate!
async fn handler() -> &'static str { "ok" }
```

## FAQ

### Can I mix methods?

Yes! You can stack different HTTP method macros:

```rust
#[get("/api/resource")]
#[post("/api/resource")]
#[put("/api/resource")]
#[delete("/api/resource")]
async fn handle_resource() -> Response {
    // Handle based on req.method()
}
```

### Does `fwk routes` show all routes?

Yes! The CLI will list each route separately:

```bash
$ fwk routes

  GET     /health                  health_check
  GET     /api/health              health_check
  GET     /status                  health_check
```

### Is this different from wildcards?

Yes! This registers distinct routes, not a wildcard:

```rust
// Multiple explicit routes
#[get("/health")]
#[get("/api/health")]
async fn health() -> Json<Value>

// NOT the same as a wildcard (which we don't support yet)
#[get("/**/health")]  // This doesn't work
async fn health() -> Json<Value>
```

## Comparison with Other Frameworks

### Express.js (JavaScript)
```javascript
app.get(['/health', '/api/health', '/status'], healthCheck);
```

### FastAPI (Python)
```python
@app.get("/health")
@app.get("/api/health")
async def health_check():
    return {"status": "ok"}
```

### Rails (Ruby)
```ruby
get '/health', to: 'health#check'
get '/api/health', to: 'health#check'
```

### Firework (Rust)
```rust
#[get("/health")]
#[get("/api/health")]
async fn health_check() -> Json<Value> {
    json!({"status": "ok"})
}
```

**Firework advantage**: Type-safe, compile-time checked, zero-cost!

## Best Practices

1. **Use for aliases, not business logic**
   - Good: Same functionality, different paths
   - Bad: Different behavior based on path

2. **Keep it simple**
   - Don't stack 10+ routes on one handler
   - Consider using a scope instead

3. **Document why you have multiple routes**
   ```rust
   // Legacy support: old clients use /api/v1/users
   #[get("/api/v1/users")]
   #[get("/api/v2/users")]
   async fn list_users() -> Json<Vec<User>>
   ```

4. **Use meaningful route names**
   - Paths should be self-documenting
   - `/health` and `/api/health` are clear
   - `/h` and `/a/h` are confusing

## Migration Guide

If you have duplicate handlers:

**Before**:
```rust
#[get("/health")]
async fn health_check() -> Json<Value> {
    json!({"status": "ok"})
}

#[get("/api/health")]
async fn api_health_check() -> Json<Value> {
    json!({"status": "ok"})  // Duplicate code!
}
```

**After**:
```rust
#[get("/health")]
#[get("/api/health")]
async fn health_check() -> Json<Value> {
    json!({"status": "ok"})  // DRY!
}
```

---

**Part of Firework Framework** 🔥 - Making web development faster and more enjoyable!
