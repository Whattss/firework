# 🔍 Extractors

Type-safe request data extraction in Firework.

---

## What are Extractors?

Extractors are types that implement `FromRequest` trait to extract data from requests in a type-safe manner.

```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> String {
    //                ^^^^^^^^^^^^^^^^^^^
    //                Extractor extracts :id as u32
    format!("User ID: {}", id)
}
```

---

## Built-in Extractors

### 1. Path - Route Parameters

Extract parameters from the URL path:

```rust
// Single parameter
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> String {
    format!("User {}", id)
}

// String parameter
#[get("/posts/:slug")]
async fn get_post(Path(slug): Path<String>) -> String {
    format!("Post: {}", slug)
}

// Custom type (implements FromStr)
use uuid::Uuid;

#[get("/items/:uuid")]
async fn get_item(Path(uuid): Path<Uuid>) -> String {
    format!("Item: {}", uuid)
}
```

**Multiple parameters** (use Request directly):
```rust
#[get("/users/:user_id/posts/:post_id")]
async fn user_post(req: Request) -> String {
    let user_id: u32 = req.param_as("user_id").unwrap();
    let post_id: u32 = req.param_as("post_id").unwrap();
    format!("User: {}, Post: {}", user_id, post_id)
}
```

### 2. Json - JSON Body

Deserialize JSON request body:

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
    age: u32,
}

#[post("/users")]
async fn create_user(Json(user): Json<CreateUser>) -> String {
    format!("Creating user: {}", user.username)
}
```

**Return JSON:**
```rust
#[derive(Serialize)]
struct User {
    id: u32,
    username: String,
}

#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(User {
        id,
        username: "john".into(),
    })
}
```

### 3. Query - Query Parameters

Extract query string parameters:

```rust
#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
}

#[get("/items")]
async fn list_items(Query(params): Query<Pagination>) -> String {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    
    format!("Page {}, Limit {}", page, limit)
}
```

**Test:**
```bash
curl "http://localhost:8080/items?page=2&limit=20&sort=name"
```

### 4. Body - Raw Body

Get raw request body as string:

```rust
#[post("/data")]
async fn handle_data(Body(content): Body) -> String {
    format!("Received {} bytes", content.len())
}
```

### 5. Request - Full Request

Access the complete request:

```rust
#[get("/info")]
async fn request_info(req: Request) -> String {
    format!(
        "Method: {:?}, Path: {}, Headers: {}",
        req.method,
        req.uri.path,
        req.headers.len()
    )
}
```

---

## Combining Extractors

Use multiple extractors in one handler:

```rust
#[post("/users/:id/posts")]
async fn create_post(
    Path(user_id): Path<u32>,
    Json(post): Json<CreatePost>,
    req: Request,
) -> Json<Post> {
    // user_id from path
    // post from JSON body
    // req for auth token, etc.
    
    let token = req.header("Authorization");
    
    Json(Post {
        id: 1,
        user_id,
        title: post.title,
    })
}
```

---

## Custom Extractors

Create your own extractors:

```rust
use firework::prelude::*;

// Custom extractor for authenticated user
struct AuthUser {
    id: u32,
    username: String,
}

#[async_trait::async_trait]
impl FromRequest for AuthUser {
    async fn from_request(req: &mut Request, _res: &mut Response) -> Result<Self, Error> {
        // Get from context (set by middleware)
        req.get_context::<AuthUser>()
            .ok_or_else(|| Error::Unauthorized("Not authenticated".into()))
    }
}

// Use in handler
#[get("/profile")]
async fn profile(user: AuthUser) -> String {
    format!("Logged in as: {}", user.username)
}
```

### Admin Extractor Example

```rust
struct AdminUser(AuthUser);

#[async_trait::async_trait]
impl FromRequest for AdminUser {
    async fn from_request(req: &mut Request, res: &mut Response) -> Result<Self, Error> {
        // First get authenticated user
        let user = AuthUser::from_request(req, res).await?;
        
        // Check if admin
        if is_admin(user.id).await {
            Ok(AdminUser(user))
        } else {
            Err(Error::Forbidden("Admin access required".into()))
        }
    }
}

#[get("/admin/dashboard")]
async fn admin_dashboard(admin: AdminUser) -> String {
    format!("Welcome, admin: {}", admin.0.username)
}
```

---

## Plugin Extractors

Extract plugin instances:

```rust
use firework_auth::{AuthPlugin, Claims};

#[get("/create-token")]
async fn create_token(Extract(auth): Extract<AuthPlugin>) -> Result<String, Error> {
    let token = auth.create_token(Claims::new("user123")).await?;
    Ok(token)
}
```

---

## Validation with Extractors

Validate data during extraction:

```rust
#[derive(Deserialize)]
struct CreateUser {
    #[serde(deserialize_with = "validate_username")]
    username: String,
    #[serde(deserialize_with = "validate_email")]
    email: String,
}

fn validate_username<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.len() < 3 {
        return Err(serde::de::Error::custom("Username too short"));
    }
    Ok(s)
}

fn validate_email<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if !s.contains('@') {
        return Err(serde::de::Error::custom("Invalid email"));
    }
    Ok(s)
}
```

---

## Best Practices

1. **Use extractors over manual parsing** - Type-safe and ergonomic
2. **Combine extractors** - Multiple extractors per handler OK
3. **Validate early** - In extractors or deserializers
4. **Return `Result<T, Error>`** - Automatic error handling
5. **Create custom extractors** - For common patterns

---

## Next Steps

- [Error Handling](./errors.md)
- [Request & Response](./request-response.md)
