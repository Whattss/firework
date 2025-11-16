# 🔄 Migrating from Actix-web

Guide for migrating from Actix-web to Firework.

---

## Comparison

| Actix-web | Firework |
|-----------|----------|
| `#[actix_web::main]` | `#[tokio::main]` |
| `HttpServer::new()` | `routes!()` |
| `web::get()` | `#[get("/")]` |
| `web::Json<T>` | `Json<T>` |
| `web::Path<T>` | `Path<T>` |
| `HttpResponse` | `Response` |

---

## Route Definition

**Actix-web:**
```rust
#[get("/users/{id}")]
async fn get_user(path: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().json(user)
}
```

**Firework:**
```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(user)
}
```

---

## Server Setup

**Actix-web:**
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_user)
            .service(create_user)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**Firework:**
```rust
#[tokio::main]
async fn main() {
    routes!().listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Middleware

**Actix-web:**
```rust
.wrap(Logger::default())
```

**Firework:**
```rust
#[middleware]
async fn logger(req: &mut Request, res: &mut Response) -> Flow {
    println!("Request: {}", req.uri.path);
    Flow::Continue
}
```

---

## State Management

**Actix-web:**
```rust
web::Data<AppState>
```

**Firework:**
```rust
// Use context or plugins
req.get_context::<AppState>()
```
