# 🎨 Your First App

Build a complete app from scratch.

---

## What We'll Build

A simple blog API with:
- List posts
- Get single post
- Create post
- Update post
- Delete post

---

## Step 1: Setup

```bash
cargo new blog-api
cd blog-api
```

Add dependencies in `Cargo.toml`:
```toml
[dependencies]
firework = { git = "https://github.com/your-org/firework" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
lazy_static = "1.4"
```

---

## Step 2: Data Model

Create `src/models.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub content: String,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
}
```

---

## Step 3: Storage Layer

Create `src/storage.rs`:

```rust
use crate::models::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

pub type Storage = Arc<RwLock<HashMap<Uuid, Post>>>;

lazy_static::lazy_static! {
    pub static ref POSTS: Storage = Arc::new(RwLock::new(HashMap::new()));
}

pub async fn create_post(data: CreatePost) -> Post {
    let post = Post {
        id: Uuid::new_v4(),
        title: data.title,
        content: data.content,
        author: data.author,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    POSTS.write().await.insert(post.id, post.clone());
    post
}

pub async fn get_all_posts() -> Vec<Post> {
    POSTS.read().await.values().cloned().collect()
}

pub async fn get_post(id: Uuid) -> Option<Post> {
    POSTS.read().await.get(&id).cloned()
}

pub async fn update_post(id: Uuid, data: UpdatePost) -> Option<Post> {
    let mut posts = POSTS.write().await;
    let post = posts.get_mut(&id)?;
    
    if let Some(title) = data.title {
        post.title = title;
    }
    if let Some(content) = data.content {
        post.content = content;
    }
    post.updated_at = Utc::now();
    
    Some(post.clone())
}

pub async fn delete_post(id: Uuid) -> bool {
    POSTS.write().await.remove(&id).is_some()
}
```

---

## Step 4: API Handlers

Create `src/handlers.rs`:

```rust
use firework::prelude::*;
use uuid::Uuid;
use crate::models::*;
use crate::storage::*;

#[get("/posts")]
pub async fn list_posts() -> Json<Vec<Post>> {
    Json(get_all_posts().await)
}

#[get("/posts/:id")]
pub async fn get_post_handler(Path(id): Path<Uuid>) -> Result<Json<Post>, Error> {
    let post = get_post(id).await
        .ok_or_else(|| Error::NotFound("Post not found".into()))?;
    Ok(Json(post))
}

#[post("/posts")]
pub async fn create_post_handler(Json(data): Json<CreatePost>) -> Json<Post> {
    Json(create_post(data).await)
}

#[put("/posts/:id")]
pub async fn update_post_handler(
    Path(id): Path<Uuid>,
    Json(data): Json<UpdatePost>,
) -> Result<Json<Post>, Error> {
    let post = update_post(id, data).await
        .ok_or_else(|| Error::NotFound("Post not found".into()))?;
    Ok(Json(post))
}

#[delete("/posts/:id")]
pub async fn delete_post_handler(Path(id): Path<Uuid>) -> Result<Response, Error> {
    if delete_post(id).await {
        Ok(Response::new(StatusCode::NoContent, b""))
    } else {
        Err(Error::NotFound("Post not found".into()))
    }
}
```

---

## Step 5: Main Application

Edit `src/main.rs`:

```rust
use firework::prelude::*;

mod models;
mod storage;
mod handlers;

#[middleware]
async fn logger(req: &mut Request, _res: &mut Response) -> Flow {
    println!("→ {} {}", format!("{:?}", req.method), req.uri.path);
    Flow::Continue
}

#[middleware]
async fn cors(_req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("Access-Control-Allow-Origin".into(), "*".into());
    res.headers.insert(
        "Access-Control-Allow-Methods".into(),
        "GET, POST, PUT, DELETE".into()
    );
    Flow::Continue
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("🔥 Blog API running on http://127.0.0.1:8080");
    println!("\n📖 Endpoints:");
    println!("  GET    /posts      - List all posts");
    println!("  GET    /posts/:id  - Get one post");
    println!("  POST   /posts      - Create post");
    println!("  PUT    /posts/:id  - Update post");
    println!("  DELETE /posts/:id  - Delete post\n");
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Step 6: Test It!

```bash
cargo run
```

### Create a post
```bash
curl -X POST http://localhost:8080/posts \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My First Post",
    "content": "This is amazing!",
    "author": "John Doe"
  }'
```

### List all posts
```bash
curl http://localhost:8080/posts
```

### Get one post (use ID from response)
```bash
curl http://localhost:8080/posts/PASTE-UUID-HERE
```

### Update post
```bash
curl -X PUT http://localhost:8080/posts/PASTE-UUID-HERE \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Title"
  }'
```

### Delete post
```bash
curl -X DELETE http://localhost:8080/posts/PASTE-UUID-HERE
```

---

## 🎉 Congratulations!

You've built a complete RESTful API with Firework!

### Next Steps:
1. Add [database persistence](../guides/database.md)
2. Add [authentication](../guides/auth-flow.md)
3. Deploy to [production](../performance/deployment.md)
