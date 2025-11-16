# 🎯 Handlers

Complete guide to Firework handler functions and signatures.

---

## Handler Basics

Handlers are async functions that process requests and return responses.

### Basic Handler

```rust
#[get("/")]
async fn index() -> &'static str {
    "Hello, World!"
}
```

---

## Handler Signatures

Firework supports **multiple handler signatures** for flexibility.

### 1. Simple Return Value

Return types that implement `IntoResponse`:

```rust
// String
#[get("/text")]
async fn text() -> &'static str {
    "Plain text response"
}

// Owned String
#[get("/dynamic")]
async fn dynamic() -> String {
    format!("Generated at {:?}", std::time::SystemTime::now())
}

// JSON
#[get("/json")]
async fn json() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Hello",
        "status": "ok"
    }))
}

// Response directly
#[get("/custom")]
async fn custom() -> Response {
    Response::new(StatusCode::Ok, b"Custom response")
}
```

### 2. With Request

Access the full request:

```rust
#[get("/info")]
async fn info(req: Request) -> String {
    format!("Method: {:?}, Path: {}", req.method, req.uri.path)
}
```

### 3. With Request and Response

Modify the response:

```rust
#[get("/headers")]
async fn headers(req: Request, mut res: Response) -> Response {
    res.headers.insert("X-Custom".into(), "Value".into());
    res.status = StatusCode::Ok;
    res.set_body(b"With custom headers".to_vec());
    res
}
```

### 4. With Extractors

Type-safe parameter extraction:

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u32,
    username: String,
    email: String,
}

// Path parameter
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(User {
        id,
        username: "john".into(),
        email: "john@example.com".into(),
    })
}

// JSON body
#[post("/users")]
async fn create_user(Json(data): Json<CreateUser>) -> Json<User> {
    Json(User {
        id: 1,
        username: data.username,
        email: data.email,
    })
}

// Query parameters
#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

#[get("/users")]
async fn list_users(Query(params): Query<Pagination>) -> Json<Vec<User>> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    
    // Fetch users...
    Json(vec![])
}
```

### 5. Multiple Extractors

Combine multiple extractors:

```rust
#[post("/users/:id/posts")]
async fn create_post(
    Path(user_id): Path<u32>,
    Json(post): Json<CreatePost>,
    req: Request,
) -> Json<Post> {
    // user_id from path
    // post from JSON body
    // req for additional context
    
    Json(Post::default())
}
```

---

## Return Types

### Built-in Return Types

Firework supports these return types out of the box:

```rust
// 1. &'static str
#[get("/")]
async fn str_response() -> &'static str {
    "Static string"
}

// 2. String
#[get("/")]
async fn string_response() -> String {
    String::from("Owned string")
}

// 3. Json<T>
#[get("/")]
async fn json_response() -> Json<MyStruct> {
    Json(MyStruct { /* ... */ })
}

// 4. Response
#[get("/")]
async fn response_response() -> Response {
    Response::new(StatusCode::Ok, b"Raw response")
}

// 5. Result<T, Error>
#[get("/")]
async fn result_response() -> Result<Json<Data>, Error> {
    Ok(Json(Data::default()))
}
```

### Result for Error Handling

Use `Result` to handle errors gracefully:

```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, Error> {
    if id == 0 {
        return Err(Error::BadRequest("Invalid ID".into()));
    }
    
    // Fetch from database...
    let user = fetch_user(id).await
        .ok_or_else(|| Error::NotFound("User not found".into()))?;
    
    Ok(Json(user))
}
```

**Error is automatically converted to JSON response:**
```json
{
  "error": "User not found",
  "status": 404
}
```

---

## Custom Return Types

Implement `IntoResponse` for custom types:

```rust
use firework::prelude::*;

struct HtmlResponse(String);

impl IntoResponse for HtmlResponse {
    fn into_response(self) -> Response {
        Response::new(StatusCode::Ok, self.0.into_bytes())
            .with_header("Content-Type", "text/html; charset=utf-8")
    }
}

#[get("/page")]
async fn page() -> HtmlResponse {
    HtmlResponse(String::from("<h1>Hello</h1>"))
}
```

### XML Response Example

```rust
struct XmlResponse(String);

impl IntoResponse for XmlResponse {
    fn into_response(self) -> Response {
        Response::new(StatusCode::Ok, self.0.into_bytes())
            .with_header("Content-Type", "application/xml")
    }
}

#[get("/data.xml")]
async fn xml_data() -> XmlResponse {
    XmlResponse(String::from(r#"<?xml version="1.0"?><data>Hello</data>"#))
}
```

---

## Handler Patterns

### CRUD Operations

```rust
// Create
#[post("/items")]
async fn create(Json(item): Json<Item>) -> Result<Json<Item>, Error> {
    // Validate
    if item.name.is_empty() {
        return Err(Error::BadRequest("Name required".into()));
    }
    
    // Save to DB
    let saved = save_item(item).await?;
    
    Ok(Json(saved))
}

// Read (one)
#[get("/items/:id")]
async fn show(Path(id): Path<u32>) -> Result<Json<Item>, Error> {
    let item = find_item(id).await
        .ok_or_else(|| Error::NotFound("Item not found".into()))?;
    
    Ok(Json(item))
}

// Read (all)
#[get("/items")]
async fn index(Query(params): Query<Pagination>) -> Json<Vec<Item>> {
    let items = list_items(params).await;
    Json(items)
}

// Update
#[put("/items/:id")]
async fn update(
    Path(id): Path<u32>,
    Json(item): Json<Item>,
) -> Result<Json<Item>, Error> {
    let updated = update_item(id, item).await?;
    Ok(Json(updated))
}

// Delete
#[delete("/items/:id")]
async fn destroy(Path(id): Path<u32>) -> Result<Response, Error> {
    delete_item(id).await?;
    Ok(Response::new(StatusCode::NoContent, b""))
}
```

### File Upload Handler

```rust
#[post("/upload")]
async fn upload(req: Request) -> Result<Json<UploadResponse>, Error> {
    // Get raw body
    let body = req.body;
    
    if body.len() > 10_000_000 {
        return Err(Error::PayloadTooLarge("File too large".into()));
    }
    
    // Save file
    let filename = save_file(&body).await?;
    
    Ok(Json(UploadResponse {
        filename,
        size: body.len(),
    }))
}
```

### Streaming Response

```rust
use tokio::fs::File;

#[get("/download/:file")]
async fn download(Path(file): Path<String>) -> Result<Response, Error> {
    let path = format!("./files/{}", file);
    
    // Security: prevent directory traversal
    if file.contains("..") {
        return Err(Error::Forbidden("Invalid path".into()));
    }
    
    let file = File::open(path).await
        .map_err(|_| Error::NotFound("File not found".into()))?;
    
    let mut response = Response::stream(StatusCode::Ok, file);
    response.headers.insert(
        "Content-Disposition".into(),
        format!("attachment; filename=\"{}\"", file)
    );
    
    Ok(response)
}
```

### Redirect Handler

```rust
#[get("/old-url")]
async fn redirect() -> Response {
    Response::new(StatusCode::Found, b"")
        .with_header("Location", "/new-url")
}
```

### Health Check

```rust
#[derive(Serialize)]
struct HealthStatus {
    status: &'static str,
    version: &'static str,
    uptime: u64,
}

#[get("/health")]
async fn health() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        uptime: get_uptime(),
    })
}
```

---

## Async Operations

### Database Queries

```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, Error> {
    // Async database call
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|_| Error::NotFound("User not found".into()))?;
    
    Ok(Json(user))
}
```

### External API Calls

```rust
#[get("/weather/:city")]
async fn weather(Path(city): Path<String>) -> Result<Json<WeatherData>, Error> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("https://api.weather.com/v1/{}", city))
        .send()
        .await
        .map_err(|_| Error::ServiceUnavailable("Weather API down".into()))?;
    
    let data = response.json::<WeatherData>().await
        .map_err(|_| Error::Internal("Failed to parse response".into()))?;
    
    Ok(Json(data))
}
```

### Concurrent Operations

```rust
use tokio::try_join;

#[get("/dashboard/:user_id")]
async fn dashboard(Path(user_id): Path<u32>) -> Result<Json<Dashboard>, Error> {
    // Fetch multiple resources concurrently
    let (user, posts, stats) = try_join!(
        fetch_user(user_id),
        fetch_user_posts(user_id),
        fetch_user_stats(user_id),
    ).map_err(|_| Error::Internal("Failed to fetch data".into()))?;
    
    Ok(Json(Dashboard { user, posts, stats }))
}
```

---

## Handler Helpers

### Response Builders

```rust
// JSON helper
fn json_ok<T: Serialize>(data: T) -> Response {
    Response::new(StatusCode::Ok, serde_json::to_vec(&data).unwrap())
        .with_header("Content-Type", "application/json")
}

// Error helper
fn error_response(status: StatusCode, message: &str) -> Response {
    let body = serde_json::json!({
        "error": message,
        "status": status.code(),
    });
    Response::new(status, serde_json::to_vec(&body).unwrap())
        .with_header("Content-Type", "application/json")
}

#[get("/helper-example")]
async fn example() -> Response {
    json_ok(vec!["item1", "item2"])
}
```

---

## Best Practices

1. **Use extractors** - Type-safe and ergonomic
2. **Return `Result<T, Error>`** - Automatic error handling
3. **Keep handlers thin** - Business logic in services
4. **Use async properly** - Don't block the runtime
5. **Validate input early** - Return errors immediately
6. **Document public APIs** - Use doc comments

---

## Anti-Patterns to Avoid

❌ **Blocking operations:**
```rust
// DON'T
#[get("/")]
async fn bad() -> String {
    std::thread::sleep(Duration::from_secs(1)); // Blocks the runtime!
    String::from("done")
}

// DO
#[get("/")]
async fn good() -> String {
    tokio::time::sleep(Duration::from_secs(1)).await;
    String::from("done")
}
```

❌ **Unwrap in handlers:**
```rust
// DON'T
#[get("/users/:id")]
async fn bad(Path(id): Path<u32>) -> Json<User> {
    let user = find_user(id).await.unwrap(); // Panics on error!
    Json(user)
}

// DO
#[get("/users/:id")]
async fn good(Path(id): Path<u32>) -> Result<Json<User>, Error> {
    let user = find_user(id).await
        .ok_or_else(|| Error::NotFound("User not found".into()))?;
    Ok(Json(user))
}
```

---

## Next Steps

- [Extractors](./extractors.md) - Deep dive into extractors
- [Middleware](./middleware.md) - Request processing
- [Error Handling](./errors.md) - Comprehensive error handling
