# 🚀 Quick Start

Get up and running with Firework in **5 minutes**!

---

## Step 1: Create a New Project

Using the CLI:
```bash
fwk new my-app
cd my-app
```

Or manually:
```bash
cargo new my-app
cd my-app
```

---

## Step 2: Add Dependencies

Edit `Cargo.toml`:

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
firework = { git = "https://github.com/your-org/firework" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## Step 3: Write Your First Handler

Edit `src/main.rs`:

```rust
use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello, Firework! 🔥"
}

#[get("/hello/:name")]
async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

#[post("/api/users")]
async fn create_user(Json(user): Json<User>) -> Json<User> {
    // Process the user...
    Json(user)
}

#[derive(serde::Deserialize, serde::Serialize)]
struct User {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("🔥 Server running on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Step 4: Run Your App

```bash
cargo run
```

You should see:
```
🔥 Server running on http://127.0.0.1:8080
[SERVER] Listening on 127.0.0.1:8080 with SO_REUSEPORT
```

---

## Step 5: Test Your Routes

**Terminal 1** - Hello World:
```bash
curl http://127.0.0.1:8080/
# Output: Hello, Firework! 🔥
```

**Terminal 2** - Path parameter:
```bash
curl http://127.0.0.1:8080/hello/Alice
# Output: Hello, Alice!
```

**Terminal 3** - JSON API:
```bash
curl -X POST http://127.0.0.1:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Bob","email":"bob@example.com"}'
# Output: {"name":"Bob","email":"bob@example.com"}
```

---

## Step 6: Add Middleware

Add logging middleware:

```rust
#[middleware]
async fn logger(req: &mut Request, res: &mut Response) -> Flow {
    let start = std::time::Instant::now();
    let method = format!("{:?}", req.method);
    let path = req.uri.path.clone();
    
    println!("→ {} {}", method, path);
    
    Flow::Continue
}
```

The `#[middleware]` macro auto-registers it!

---

## Step 7: Add Error Handling

Use Result and custom errors:

```rust
#[get("/api/user/:id")]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, Error> {
    if id == 0 {
        return Err(Error::BadRequest("ID cannot be 0".into()));
    }
    
    // Fetch user from database...
    let user = User {
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };
    
    Ok(Json(user))
}
```

Test error:
```bash
curl http://127.0.0.1:8080/api/user/0
# Output: {"error":"ID cannot be 0","status":400}
```

---

## Step 8: Enable Hot Reload (Optional)

For development, enable hot-reload:

**Terminal 1:**
```bash
cargo install --path . --features hot-reload --bin firework-dev
firework-dev
```

Now your changes auto-reload on save! 🔄

---

## Complete Example

Here's a complete REST API example:

```rust
use firework::prelude::*;
use serde::{Deserialize, Serialize};

// Models
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
}

// Routes
#[get("/")]
async fn index() -> &'static str {
    "Todo API - Use /api/todos"
}

#[get("/api/todos")]
async fn list_todos() -> Json<Vec<Todo>> {
    let todos = vec![
        Todo { id: 1, title: "Learn Firework".into(), completed: true },
        Todo { id: 2, title: "Build an app".into(), completed: false },
    ];
    Json(todos)
}

#[get("/api/todos/:id")]
async fn get_todo(Path(id): Path<u32>) -> Result<Json<Todo>, Error> {
    if id == 0 {
        return Err(Error::BadRequest("Invalid ID".into()));
    }
    
    // Mock data
    let todo = Todo {
        id,
        title: format!("Todo #{}", id),
        completed: false,
    };
    
    Ok(Json(todo))
}

#[post("/api/todos")]
async fn create_todo(Json(mut todo): Json<Todo>) -> Json<Todo> {
    todo.id = 3; // In real app, generate from DB
    Json(todo)
}

#[put("/api/todos/:id")]
async fn update_todo(
    Path(id): Path<u32>,
    Json(todo): Json<Todo>,
) -> Json<Todo> {
    // Update in database...
    Json(todo)
}

#[delete("/api/todos/:id")]
async fn delete_todo(Path(id): Path<u32>) -> Response {
    // Delete from database...
    Response::new(StatusCode::NoContent, b"")
}

// Middleware
#[middleware]
async fn cors(_req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("Access-Control-Allow-Origin".into(), "*".into());
    Flow::Continue
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("🔥 Todo API running on http://127.0.0.1:8080");
    println!("📝 Try: curl http://127.0.0.1:8080/api/todos");
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

**Test it:**
```bash
# List all
curl http://127.0.0.1:8080/api/todos

# Get one
curl http://127.0.0.1:8080/api/todos/1

# Create
curl -X POST http://127.0.0.1:8080/api/todos \
  -H "Content-Type: application/json" \
  -d '{"id":0,"title":"New todo","completed":false}'

# Update
curl -X PUT http://127.0.0.1:8080/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"id":1,"title":"Updated","completed":true}'

# Delete
curl -X DELETE http://127.0.0.1:8080/api/todos/1
```

---

## What's Next?

🎉 **Congratulations!** You've built your first Firework app!

**Continue learning:**

1. **[Routing](../core/routing.md)** - Advanced routing patterns
2. **[Handlers](../core/handlers.md)** - Different handler signatures
3. **[Middleware](../core/middleware.md)** - Request/response processing
4. **[Database](../guides/database.md)** - Integrate SeaORM
5. **[Authentication](../plugins/auth.md)** - Add JWT auth
6. **[WebSockets](../advanced/websockets.md)** - Real-time features

**Check examples:**
```bash
cd examples/
cargo run --example hello_world
cargo run --example websocket_chat
```

---

## Common Next Steps

### Add a Database

```toml
[dependencies]
firework-seaorm = { path = "plugins/firework-seaorm" }
sea-orm = { version = "0.12", features = ["sqlx-sqlite", "runtime-tokio-native-tls"] }
```

See [Database Guide](../guides/database.md)

### Add Authentication

```toml
[dependencies]
firework-auth = { path = "plugins/firework-auth" }
```

See [Auth Guide](../plugins/auth.md)

### Add Static Files

```rust
#[get("/static/*")]
async fn static_files(req: Request) -> Response {
    serve_static("./static", &req.uri.path).await
}
```

See [Static Files Guide](../advanced/static-files.md)

---

## Tips for Beginners

1. **Use `#[middleware]` liberally** - They auto-register
2. **Prefer extractors** - `Path<T>`, `Json<T>` instead of manual parsing
3. **Return `Result<T, Error>`** - Automatic error responses
4. **Enable hot-reload** - Save development time
5. **Check examples/** - Real-world patterns

---

## Need Help?

- 📖 [Full Documentation](../README.md)
- 💬 [Discord](https://discord.gg/firework)
- 🐛 [GitHub Issues](https://github.com/your-org/firework/issues)
- 📧 support@firework-rs.dev
