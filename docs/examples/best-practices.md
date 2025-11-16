# ✨ Best Practices

Recommended patterns and practices for Firework applications.

---

## Project Organization

✅ **DO:** Use modules for routes
```rust
mod routes {
    mod users;
    mod posts;
}
```

❌ **DON'T:** Put everything in main.rs

---

## Error Handling

✅ **DO:** Return Result<T, Error>
```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, Error> {
    let user = find_user(id).await?;
    Ok(Json(user))
}
```

❌ **DON'T:** Use unwrap() in handlers
```rust
let user = find_user(id).await.unwrap(); // Can panic!
```

---

## Async Operations

✅ **DO:** Use tokio async functions
```rust
tokio::time::sleep(Duration::from_secs(1)).await;
```

❌ **DON'T:** Block the runtime
```rust
std::thread::sleep(Duration::from_secs(1)); // Blocks!
```

---

## Middleware

✅ **DO:** Keep middleware fast
```rust
#[middleware]
async fn quick_check(req: &mut Request, res: &mut Response) -> Flow {
    // Fast validation
    Flow::Continue
}
```

❌ **DON'T:** Heavy computation in middleware
```rust
// Slow database queries, expensive operations
```

---

## Type Safety

✅ **DO:** Use extractors
```rust
#[post("/users")]
async fn create(Json(user): Json<User>) -> Json<User> {
    Json(user)
}
```

❌ **DON'T:** Manual parsing
```rust
let body = std::str::from_utf8(&req.body).unwrap();
let user: User = serde_json::from_str(body).unwrap();
```
