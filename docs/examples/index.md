# 📚 Code Examples

Collection of practical examples.

---

## Basic Examples

- [Hello World](../../examples/hello_world.rs)
- [Flexible Handlers](../../examples/flexible_handlers.rs)
- [Config Example](../../examples/config_example.rs)

## Advanced Examples

- [WebSocket Chat](../../examples/websocket_chat.rs)
- [SeaORM Integration](../../examples/seaorm_example.rs)
- [Static File Server](../../examples/complete_static_server.rs)
- [Plugin Example](../../examples/plugin_example.rs)

## Real-World Examples

- [Twitter Clone](../../examples/twitter-clone/)
- [Undergun App](../../undergun/)

---

## Quick Snippets

### JSON API

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(User { id, name: "John".into() })
}
```

### File Download

```rust
#[get("/download/:file")]
async fn download(Path(file): Path<String>) -> Result<Response, Error> {
    let path = format!("./files/{}", file);
    let file = tokio::fs::File::open(path).await?;
    Ok(Response::stream(StatusCode::Ok, file))
}
```

### Middleware Chain

```rust
#[middleware]
async fn logger(req: &mut Request, res: &mut Response) -> Flow {
    println!("→ {}", req.uri.path);
    Flow::Continue
}

#[middleware]
async fn auth(req: &mut Request, res: &mut Response) -> Flow {
    match req.header("Authorization") {
        Some(_) => Flow::Continue,
        None => Flow::Stop(Response::new(StatusCode::Unauthorized, b"Unauthorized"))
    }
}
```
