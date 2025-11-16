# Macros Reference

Complete reference for all Firework macros.

---

## Route Macros

### `#[get(path)]`

Register a GET route.

```rust
#[get("/")]
async fn index() -> &'static str {
    "Home"
}
```

### `#[post(path)]`, `#[put(path)]`, `#[patch(path)]`, `#[delete(path)]`

Register routes for other HTTP methods.

```rust
#[post("/users")]
async fn create_user(Json(user): Json<User>) -> Json<User> {
    Json(user)
}
```

---

## WebSocket Macro

### `#[ws(path)]`

Register a WebSocket route.

```rust
#[ws("/chat")]
async fn chat_handler(mut ws: WebSocket) {
    while let Some(msg) = ws.recv().await {
        ws.send(msg).await.ok();
    }
}
```

---

## Middleware Macro

### `#[middleware]`

Register a middleware function.

```rust
#[middleware]
async fn logger(req: &mut Request, res: &mut Response) -> Flow {
    println!("Request: {}", req.uri.path);
    Flow::Continue
}
```

### `#[middleware(post)]`

Register a post-handler middleware.

```rust
#[middleware(post)]
async fn response_time(req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("X-Response-Time".into(), "10ms".into());
    Flow::Continue
}
```

---

## Scope Macro

### `#[scope(prefix)]`

Create a route scope.

```rust
#[scope("/api")]
mod api {
    #[get("/users")]
    async fn users() -> &'static str {
        "Users"  // Matches: /api/users
    }
}
```

### `#[scope(prefix, middleware = [...])]`

Scope with middleware.

```rust
#[scope("/admin", middleware = [require_auth])]
mod admin {
    #[get("/dashboard")]
    async fn dashboard() -> &'static str {
        "Admin Dashboard"
    }
}
```

---

## routes!()

Build and register all routes.

```rust
#[tokio::main]
async fn main() {
    let server = routes!();
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Plugin Macro

### `#[plugin]`

Mark a struct as a plugin.

```rust
#[plugin]
struct MyPlugin {
    config: String,
}
```
