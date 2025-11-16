# 💾 Database Integration

Integrate SeaORM with Firework for database operations.

---

## Setup

**Cargo.toml:**
```toml
[dependencies]
firework = { git = "..." }
firework-seaorm = { path = "../firework/plugins/firework-seaorm" }
sea-orm = { version = "0.12", features = ["sqlx-sqlite", "runtime-tokio-native-tls"] }
```

**Firework.toml:**
```toml
[plugins.seaorm]
database_url = "sqlite://data.db"
```

---

## Define Entity

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

---

## Using in Handlers

```rust
use sea_orm::*;

#[get("/users")]
async fn list_users(db: Extract<DatabaseConnection>) -> Result<Json<Vec<users::Model>>, Error> {
    let users = users::Entity::find()
        .all(&*db)
        .await
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    Ok(Json(users))
}

#[get("/users/:id")]
async fn get_user(
    Path(id): Path<i32>,
    db: Extract<DatabaseConnection>,
) -> Result<Json<users::Model>, Error> {
    let user = users::Entity::find_by_id(id)
        .one(&*db)
        .await
        .map_err(|e| Error::Internal(e.to_string()))?
        .ok_or_else(|| Error::NotFound("User not found".into()))?;
    
    Ok(Json(user))
}
```

---

## Migrations

Create `migrations/001_create_users.sql`:
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Run migrations:
```bash
sea-orm-cli migrate up
```
