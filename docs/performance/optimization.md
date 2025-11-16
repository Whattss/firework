# ⚡ Optimization Guide

Performance optimization tips for Firework applications.

---

## Compiler Optimizations

**Cargo.toml:**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## Server Configuration

**Firework.toml:**
```toml
[server]
workers = 8  # Number of CPU cores
```

---

## Database Connection Pooling

```rust
// Use connection pools
let pool = sea_orm::Database::connect(&config.database_url)
    .await?;

// Reuse connections
static DB_POOL: OnceCell<DatabaseConnection> = OnceCell::new();
```

---

## Caching

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    static ref CACHE: Arc<RwLock<HashMap<String, String>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

#[get("/data/:id")]
async fn get_data(Path(id): Path<String>) -> String {
    let cache = CACHE.read().await;
    
    if let Some(data) = cache.get(&id) {
        return data.clone(); // Cache hit!
    }
    
    drop(cache); // Release read lock
    
    // Fetch from database
    let data = expensive_operation(&id).await;
    
    // Store in cache
    CACHE.write().await.insert(id, data.clone());
    
    data
}
```

---

## Avoid Cloning

```rust
// ❌ BAD
#[middleware]
async fn bad(req: &mut Request, res: &mut Response) -> Flow {
    let path = req.uri.path.clone();  // Unnecessary
    println!("{}", path);
    Flow::Continue
}

// ✅ GOOD
#[middleware]
async fn good(req: &mut Request, res: &mut Response) -> Flow {
    println!("{}", req.uri.path);  // No clone
    Flow::Continue
}
```

---

## Use `&str` over `String`

```rust
// ✅ GOOD
#[get("/")]
async fn index() -> &'static str {
    "Hello"
}

// ❌ AVOID (if possible)
#[get("/")]
async fn index() -> String {
    String::from("Hello")
}
```

---

## Lazy Static for Globals

```rust
use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG: Config = load_config();
}
```

---

## Batch Operations

```rust
// ✅ GOOD - Batch fetch
let users = User::find()
    .filter(user::Column::Id.is_in(ids))
    .all(&db)
    .await?;

// ❌ BAD - N+1 queries
for id in ids {
    let user = User::find_by_id(id).one(&db).await?;
}
```
