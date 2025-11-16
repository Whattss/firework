# 🛣️ Routing

Complete guide to Firework's powerful routing system.

---

## Basic Routing

### Simple Routes

```rust
use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Home page"
}

#[get("/about")]
async fn about() -> &'static str {
    "About us"
}

#[get("/contact")]
async fn contact() -> &'static str {
    "Contact page"
}
```

### HTTP Methods

Firework supports all standard HTTP methods:

```rust
#[get("/resource")]
async fn get_resource() -> &'static str { "GET" }

#[post("/resource")]
async fn create_resource() -> &'static str { "POST" }

#[put("/resource")]
async fn update_resource() -> &'static str { "PUT" }

#[patch("/resource")]
async fn patch_resource() -> &'static str { "PATCH" }

#[delete("/resource")]
async fn delete_resource() -> &'static str { "DELETE" }
```

---

## Path Parameters

### Single Parameter

```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> String {
    format!("User ID: {}", id)
}

#[get("/posts/:slug")]
async fn get_post(Path(slug): Path<String>) -> String {
    format!("Post slug: {}", slug)
}
```

**Test:**
```bash
curl http://localhost:8080/users/42
# Output: User ID: 42

curl http://localhost:8080/posts/hello-world
# Output: Post slug: hello-world
```

### Multiple Parameters

```rust
#[get("/users/:user_id/posts/:post_id")]
async fn get_user_post(req: Request) -> String {
    let user_id = req.param("user_id").unwrap();
    let post_id = req.param("post_id").unwrap();
    
    format!("User: {}, Post: {}", user_id, post_id)
}
```

**Or with type-safe extraction:**

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct UserPostParams {
    user_id: u32,
    post_id: u32,
}

// Note: This requires custom extractor implementation
// For now, use req.param() for multiple params
```

---

## Query Parameters

### Basic Query Params

```rust
#[get("/search")]
async fn search(req: Request) -> String {
    let query = req.query("q").unwrap_or(&String::from(""));
    let page = req.query("page").unwrap_or(&String::from("1"));
    
    format!("Search: {}, Page: {}", query, page)
}
```

**Test:**
```bash
curl "http://localhost:8080/search?q=rust&page=2"
# Output: Search: rust, Page: 2
```

### Type-Safe Query Params

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    page: Option<u32>,
    limit: Option<u32>,
}

#[get("/search")]
async fn search(Query(params): Query<SearchQuery>) -> String {
    format!(
        "Search: {}, Page: {}, Limit: {}",
        params.q,
        params.page.unwrap_or(1),
        params.limit.unwrap_or(10)
    )
}
```

---

## Wildcard Routes

### Catch-All Routes

```rust
#[get("/files/*")]
async fn serve_files(req: Request) -> Response {
    let path = &req.uri.path;
    serve_static("./static", path).await
}

#[get("/*")]
async fn fallback() -> Response {
    Response::new(StatusCode::NotFound, b"404 - Page not found")
}
```

**Important:** Wildcard routes should be defined **last** as they match everything!

---

## Route Scopes

### Basic Scope

Group routes under a common prefix:

```rust
#[scope("/api")]
mod api {
    use super::*;
    
    #[get("/users")]
    async fn users() -> &'static str {
        "API Users"  // Matches: /api/users
    }
    
    #[get("/posts")]
    async fn posts() -> &'static str {
        "API Posts"  // Matches: /api/posts
    }
}
```

### Nested Scopes

```rust
#[scope("/api")]
mod api {
    use super::*;
    
    #[scope("/v1")]
    mod v1 {
        use super::*;
        
        #[get("/users")]
        async fn users() -> &'static str {
            "API V1 Users"  // Matches: /api/v1/users
        }
    }
    
    #[scope("/v2")]
    mod v2 {
        use super::*;
        
        #[get("/users")]
        async fn users() -> &'static str {
            "API V2 Users"  // Matches: /api/v2/users
        }
    }
}
```

### Scope with Middleware

```rust
#[middleware]
async fn require_api_key(req: &mut Request, res: &mut Response) -> Flow {
    match req.header("X-API-Key") {
        Some(_) => Flow::Continue,
        None => {
            *res = Response::new(StatusCode::Unauthorized, b"API key required");
            Flow::Stop(res.clone())
        }
    }
}

#[scope("/api", middleware = [require_api_key])]
mod api {
    use super::*;
    
    #[get("/secret")]
    async fn secret() -> &'static str {
        "Secret data"
    }
}
```

---

## Route Priority

Routes are matched in this order:

1. **Exact matches** - `/users/profile`
2. **Parameterized routes** - `/users/:id`
3. **Wildcard routes** - `/users/*`

```rust
#[get("/users/profile")]
async fn profile() -> &'static str {
    "User profile"  // Matches first
}

#[get("/users/:id")]
async fn user_by_id(Path(id): Path<u32>) -> String {
    format!("User {}", id)  // Matches second
}

#[get("/users/*")]
async fn users_wildcard() -> &'static str {
    "Users wildcard"  // Matches last
}
```

**Test:**
```bash
curl http://localhost:8080/users/profile
# Output: User profile

curl http://localhost:8080/users/123
# Output: User 123

curl http://localhost:8080/users/something/else
# Output: Users wildcard
```

---

## Advanced Patterns

### REST Resource Routing

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: u32,
    title: String,
    content: String,
}

// List all
#[get("/posts")]
async fn index() -> Json<Vec<Post>> {
    Json(vec![])
}

// Get one
#[get("/posts/:id")]
async fn show(Path(id): Path<u32>) -> Result<Json<Post>, Error> {
    // Fetch from DB...
    Err(Error::NotFound("Post not found".into()))
}

// Create
#[post("/posts")]
async fn create(Json(post): Json<Post>) -> Json<Post> {
    // Save to DB...
    Json(post)
}

// Update (full replace)
#[put("/posts/:id")]
async fn update(
    Path(id): Path<u32>,
    Json(post): Json<Post>,
) -> Result<Json<Post>, Error> {
    // Update in DB...
    Ok(Json(post))
}

// Partial update
#[patch("/posts/:id")]
async fn patch(
    Path(id): Path<u32>,
    Json(updates): Json<serde_json::Value>,
) -> Result<Json<Post>, Error> {
    // Apply partial updates...
    Err(Error::NotImplemented)
}

// Delete
#[delete("/posts/:id")]
async fn destroy(Path(id): Path<u32>) -> Response {
    // Delete from DB...
    Response::new(StatusCode::NoContent, b"")
}
```

### Nested Resources

```rust
// User's posts
#[get("/users/:user_id/posts")]
async fn user_posts(Path(user_id): Path<u32>) -> Json<Vec<Post>> {
    // Fetch user's posts...
    Json(vec![])
}

#[get("/users/:user_id/posts/:post_id")]
async fn user_post(req: Request) -> Result<Json<Post>, Error> {
    let user_id: u32 = req.param_as("user_id").unwrap();
    let post_id: u32 = req.param_as("post_id").unwrap();
    
    // Fetch specific post...
    Err(Error::NotFound("Post not found".into()))
}

// Create post for user
#[post("/users/:user_id/posts")]
async fn create_user_post(
    Path(user_id): Path<u32>,
    Json(post): Json<Post>,
) -> Json<Post> {
    // Create post for user...
    Json(post)
}
```

### Route Groups by Version

```rust
#[scope("/api/v1")]
mod v1 {
    use super::*;
    
    #[get("/users")]
    async fn users() -> Json<Vec<String>> {
        Json(vec!["user1".into(), "user2".into()])
    }
}

#[scope("/api/v2")]
mod v2 {
    use super::*;
    
    #[derive(Serialize)]
    struct UserV2 {
        id: u32,
        username: String,
        email: String,
    }
    
    #[get("/users")]
    async fn users() -> Json<Vec<UserV2>> {
        Json(vec![])
    }
}
```

---

## Manual Router Configuration

For more control, build routes programmatically:

```rust
use firework::prelude::*;

async fn handler1(req: Request, res: Response) -> Response {
    res
}

async fn handler2(req: Request, res: Response) -> Response {
    res
}

#[tokio::main]
async fn main() {
    let server = Server::new()
        .get("/", handler1)
        .post("/api/data", handler2)
        .scope("/admin", |scope| {
            scope
                .get("/dashboard", handler1)
                .get("/users", handler2)
        });
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Route Parameters Validation

```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, Error> {
    if id == 0 {
        return Err(Error::BadRequest("ID must be greater than 0".into()));
    }
    
    if id > 1000000 {
        return Err(Error::BadRequest("ID out of range".into()));
    }
    
    // Fetch user...
    Ok(Json(User::default()))
}
```

---

## Route Documentation

Use doc comments for auto-documentation:

```rust
/// Get all users
/// 
/// Returns a list of all users in the system.
/// 
/// # Query Parameters
/// - `page` (optional): Page number (default: 1)
/// - `limit` (optional): Items per page (default: 10)
/// 
/// # Example
/// ```
/// GET /api/users?page=2&limit=20
/// ```
#[get("/api/users")]
async fn list_users(Query(params): Query<Pagination>) -> Json<Vec<User>> {
    Json(vec![])
}
```

---

## Performance Tips

1. **Use exact routes when possible** - Faster than parameters
2. **Put common routes first** - Radix tree optimization
3. **Avoid deep nesting** - Keep URL structure flat
4. **Use scopes for organization** - No performance penalty
5. **Wildcard routes last** - They match everything

---

## Common Patterns

### Health Check Endpoint

```rust
#[get("/health")]
async fn health() -> Response {
    Response::new(StatusCode::Ok, b"OK")
}
```

### API Versioning

```rust
#[scope("/api/v1")]
mod v1 {
    // V1 endpoints
}

#[scope("/api/v2")]
mod v2 {
    // V2 endpoints
}
```

### Admin Routes

```rust
#[scope("/admin", middleware = [require_admin])]
mod admin {
    #[get("/dashboard")]
    async fn dashboard() -> &'static str { "Admin" }
}
```

---

## Debugging Routes

Print all registered routes:

```rust
#[tokio::main]
async fn main() {
    let server = routes!();
    
    // In development, you can inspect routes
    println!("Registered routes:");
    for route in ROUTES {
        println!("  {} {}", route.method, route.path);
    }
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Next Steps

- [Handlers](./handlers.md) - Handler function signatures
- [Middleware](./middleware.md) - Request/response processing
- [Extractors](./extractors.md) - Type-safe parameter extraction
