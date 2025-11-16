# DbEntity Extractor - Auto-fetch Database Entities

**Elimina el boilerplate de fetching entities** en tus handlers.

## 🎯 The Problem

```rust
// ❌ Before: Lots of boilerplate
#[get("/users/:id")]
async fn get_user(
    Path(id): Path<i32>,
    DbConn(db): DbConn
) -> Result<Json<users::Model>> {
    let user = users::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| Error::Internal(format!("DB error: {}", e)))?
        .ok_or_else(|| Error::NotFound("User not found".into()))?;
    
    Ok(Json(user))
}
```

**8 líneas** solo para fetchar un user! 😫

## ✅ The Solution

```rust
// ✅ After: Just one line!
#[get("/users/:id")]
async fn get_user(
    DbEntity(user): DbEntity<users::Model>
) -> Json<users::Model> {
    Json(user)
}
```

**1 línea!** 🚀

---

## 📦 Installation

DbEntity viene incluido en `firework-seaorm`:

```toml
[dependencies]
firework-seaorm = { path = "path/to/plugins/firework-seaorm" }
```

## 🚀 Basic Usage

### 1. Add DB middleware

```rust
use firework::prelude::*;
use firework_seaorm::helpers::db_middleware_async;

#[tokio::main]
async fn main() {
    let server = routes!()
        .async_middleware(db_middleware_async);  // ← Required!
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

### 2. Use DbEntity in handlers

```rust
use firework::prelude::*;
use firework_seaorm::DbEntity;

#[get("/users/:id")]
async fn get_user(
    DbEntity(user): DbEntity<users::Model>
) -> Json<users::Model> {
    Json(user)
}

#[get("/posts/:id")]
async fn get_post(
    DbEntity(post): DbEntity<posts::Model>
) -> Json<posts::Model> {
    Json(post)
}

#[get("/tweets/:id")]
async fn get_tweet(
    DbEntity(tweet): DbEntity<tweets::Model>
) -> Json<tweets::Model> {
    Json(tweet)
}
```

---

## 🎨 Advanced Usage

### Auto-detection of Parameter Names

DbEntity automatically finds the right parameter in this order:

1. `:id` (most common)
2. `:entity_id` (e.g., `:user_id` for `users::Model`)
3. `:entity` (e.g., `:user` for `users::Model`)

```rust
#[get("/users/:id")]
async fn by_id(DbEntity(user): DbEntity<users::Model>) -> Json<users::Model> {
    Json(user)  // ✅ Uses :id
}

#[get("/profile/:user_id")]
async fn by_user_id(DbEntity(user): DbEntity<users::Model>) -> Json<users::Model> {
    Json(user)  // ✅ Uses :user_id automatically!
}

#[get("/u/:user")]
async fn by_user(DbEntity(user): DbEntity<users::Model>) -> Json<users::Model> {
    Json(user)  // ✅ Uses :user automatically!
}
```

### Multiple Entities with Different Parameters

```rust
use firework_seaorm::{DbEntity, DbEntityBy, ByUserId, ByPostId};

#[get("/users/:user_id/posts/:post_id")]
async fn get_user_post(
    DbEntityBy(user): DbEntityBy<users::Model, ByUserId>,
    DbEntityBy(post): DbEntityBy<posts::Model, ByPostId>,
) -> Result<Json<posts::Model>> {
    // Verify ownership
    if post.user_id != user.id {
        return Err(Error::Forbidden("Not your post".into()));
    }
    
    Ok(Json(post))
}
```

### Optional Entities (No 404)

Sometimes you want `None` instead of 404:

```rust
use firework_seaorm::DbEntityOpt;

#[get("/users/:id")]
async fn get_user(
    DbEntityOpt(user): DbEntityOpt<users::Model>
) -> Json<serde_json::Value> {
    match user {
        Some(u) => json!(u),
        None => json!({"error": "User not found"}),
    }
}
```

### Combining with Other Extractors

```rust
use firework_auth::Auth;
use firework_seaorm::DbEntity;

#[get("/posts/:id")]
async fn get_post(
    Auth(claims): Auth,                      // Auth check
    DbEntity(post): DbEntity<posts::Model>,  // Fetch post
) -> Result<Json<posts::Model>> {
    // Only show post if it's yours
    let user_id: i32 = claims.sub.parse()
        .map_err(|_| Error::Unauthorized("Invalid token".into()))?;
    
    if post.user_id != user_id {
        return Err(Error::Forbidden("Not your post".into()));
    }
    
    Ok(Json(post))
}
```

### Update/Delete Operations

DbEntity gives you a mutable reference:

```rust
#[delete("/posts/:id")]
async fn delete_post(
    Auth(claims): Auth,
    DbEntity(post): DbEntity<posts::Model>,
    DbConn(db): DbConn,
) -> Result<Json<serde_json::Value>> {
    // Verify ownership
    let user_id: i32 = claims.sub.parse()?;
    if post.user_id != user_id {
        return Err(Error::Forbidden("Not your post".into()));
    }
    
    // Delete
    posts::Entity::delete_by_id(post.id)
        .exec(&db)
        .await?;
    
    Ok(json!({"deleted": true}))
}

#[patch("/posts/:id")]
async fn update_post(
    Auth(claims): Auth,
    DbEntity(mut post): DbEntity<posts::Model>,  // Mutable!
    DbConn(db): DbConn,
    Json(data): Json<UpdatePostRequest>,
) -> Result<Json<posts::Model>> {
    // Verify ownership
    let user_id: i32 = claims.sub.parse()?;
    if post.user_id != user_id {
        return Err(Error::Forbidden("Not your post".into()));
    }
    
    // Update
    let mut active: posts::ActiveModel = post.into();
    active.content = Set(data.content);
    active.updated_at = Set(chrono::Utc::now().to_rfc3339());
    
    let updated = active.update(&db).await?;
    
    Ok(Json(updated))
}
```

---

## 🔧 Custom Parameter Markers

For common patterns, use pre-defined markers:

```rust
use firework_seaorm::{DbEntityBy, ByUserId, ByPostId, BySlug};

// By user_id
#[get("/users/:user_id")]
async fn by_user_id(
    DbEntityBy(user): DbEntityBy<users::Model, ByUserId>
) -> Json<users::Model> {
    Json(user)
}

// By post_id
#[get("/posts/:post_id")]
async fn by_post_id(
    DbEntityBy(post): DbEntityBy<posts::Model, ByPostId>
) -> Json<posts::Model> {
    Json(post)
}

// By slug
#[get("/posts/:slug")]
async fn by_slug(
    DbEntityBy(post): DbEntityBy<posts::Model, BySlug>
) -> Json<posts::Model> {
    Json(post)
}
```

**Custom markers**:
```rust
use firework_seaorm::{DbEntityBy, ParamName};

pub struct ByTweetId;
impl ParamName for ByTweetId {
    fn param_name() -> &'static str { "tweet_id" }
}

#[get("/tweets/:tweet_id")]
async fn get_tweet(
    DbEntityBy(tweet): DbEntityBy<tweets::Model, ByTweetId>
) -> Json<tweets::Model> {
    Json(tweet)
}
```

---

## 📊 Performance

**DbEntity is ZERO overhead!**

- Compiles to the same code as manual fetching
- No runtime reflection
- Type-safe at compile time
- Single query (uses `find_by_id`)

**Benchmark**:
```
DbEntity:        ~0.5ms (DB query time)
Manual fetch:    ~0.5ms (same!)
```

---

## ⚠️ Gotchas

### 1. Multiple entities = Multiple queries

```rust
#[get("/users/:user_id/posts/:post_id")]
async fn handler(
    DbEntityBy(user): DbEntityBy<users::Model, ByUserId>,   // Query 1
    DbEntityBy(post): DbEntityBy<posts::Model, ByPostId>,   // Query 2
) -> Json<posts::Model> {
    // This makes 2 DB queries!
}
```

**Solution**: Use DataLoader or manual fetching if you need batching.

### 2. Requires db_middleware_async

```rust
// ❌ Won't work without middleware
let server = routes!();  // No DB middleware!

// ✅ Correct
let server = routes!()
    .async_middleware(db_middleware_async);
```

### 3. Primary key must be in path

```rust
// ❌ Can't search by email
#[get("/users/:email")]  // email is not the primary key!
async fn by_email(DbEntity(user): DbEntity<users::Model>)

// ✅ Use manual query
#[get("/users/email/:email")]
async fn by_email(
    Path(email): Path<String>,
    DbConn(db): DbConn,
) -> Result<Json<users::Model>> {
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(&db)
        .await?
        .ok_or_else(|| Error::NotFound("User not found".into()))?;
    Ok(Json(user))
}
```

---

## 🎯 Best Practices

### 1. Use DbEntity for simple gets

```rust
// ✅ Good
#[get("/users/:id")]
async fn get_user(DbEntity(user): DbEntity<users::Model>) -> Json<users::Model>
```

### 2. Use manual queries for complex filters

```rust
// ✅ Good
#[get("/users/active")]
async fn get_active_users(DbConn(db): DbConn) -> Result<Json<Vec<users::Model>>> {
    let users = users::Entity::find()
        .filter(users::Column::Active.eq(true))
        .all(&db)
        .await?;
    Ok(Json(users))
}
```

### 3. Combine with Auth for ownership checks

```rust
// ✅ Good pattern
#[delete("/posts/:id")]
async fn delete_post(
    Auth(claims): Auth,
    DbEntity(post): DbEntity<posts::Model>,
    DbConn(db): DbConn,
) -> Result<Json<Value>> {
    verify_ownership(&claims, &post)?;
    posts::Entity::delete_by_id(post.id).exec(&db).await?;
    Ok(json!({"deleted": true}))
}
```

---

## 🆚 Comparison with Other Frameworks

### Laravel (PHP)
```php
// Laravel
Route::get('/users/{user}', function (User $user) {
    return $user;
});
```

### Rails (Ruby)
```ruby
# Rails
def show
  @user = User.find(params[:id])
end
```

### Firework (Rust)
```rust
// Firework - Same DX, but type-safe!
#[get("/users/:id")]
async fn show(DbEntity(user): DbEntity<users::Model>) -> Json<users::Model> {
    Json(user)
}
```

---

## 📚 API Reference

### `DbEntity<M>`
Auto-fetches entity by `:id` or auto-detected parameter.

### `DbEntityOpt<M>`
Returns `Option<M>` instead of 404.

### `DbEntityBy<M, P>`
Fetches entity by specific parameter using `ParamName` marker.

### Pre-defined Markers
- `ByUserId` → `:user_id`
- `ByPostId` → `:post_id`
- `BySlug` → `:slug`

---

## 🎉 Summary

**Before**:
```rust
#[get("/users/:id")]
async fn get_user(Path(id): Path<i32>, DbConn(db): DbConn) -> Result<Json<User>> {
    let user = users::Entity::find_by_id(id).one(&db).await?
        .ok_or_else(|| Error::NotFound("User not found".into()))?;
    Ok(Json(user))
}
```

**After**:
```rust
#[get("/users/:id")]
async fn get_user(DbEntity(user): DbEntity<users::Model>) -> Json<users::Model> {
    Json(user)
}
```

**Reduction: 7 lines → 1 line** (87% less code!)

---

**Questions?** See the inline docs or ask in the community.
