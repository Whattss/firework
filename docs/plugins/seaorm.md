# SeaORM Plugin

Complete database integration for Firework using SeaORM - an async, dynamic ORM for Rust.

## Features

- Async database operations with Tokio
- Connection pooling
- Multiple database support (PostgreSQL, MySQL, SQLite)
- Type-safe queries
- Migrations support
- Global connection access
- Request-scoped extraction

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
firework-seaorm = { git = "https://github.com/Whattss/firework" }
sea-orm = { version = "1.1", features = ["sqlx-sqlite", "runtime-tokio-native-tls"] }
```

For other databases:
```toml
# PostgreSQL
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio-native-tls"] }

# MySQL
sea-orm = { version = "1.1", features = ["sqlx-mysql", "runtime-tokio-native-tls"] }
```

## Quick Start

### 1. Register the Plugin

```rust
use firework::prelude::*;
use firework_seaorm::SeaOrmPlugin;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Register SeaORM plugin
    let db_plugin = Arc::new(SeaOrmPlugin::new("sqlite://data.db"));
    firework::register_plugin(db_plugin);
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

### 2. Configuration via Firework.toml

```toml
[plugins.seaorm]
database_url = "sqlite://data.db"
```

Then load automatically:
```rust
let db_plugin = Arc::new(SeaOrmPlugin::from_config().await);
firework::register_plugin(db_plugin);
```

### 3. Define Entities

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

## Usage in Handlers

### Method 1: DbConn Extractor (Recommended)

```rust
use firework::prelude::*;
use firework_seaorm::DbConn;
use entity::users;

#[get("/users")]
async fn list_users(DbConn(db): DbConn) -> Response {
    match users::Entity::find().all(&db).await {
        Ok(users) => json!(users),
        Err(e) => Response::new(StatusCode::InternalServerError, vec![])
            .json(json!({"error": e.to_string()}))
    }
}

#[get("/users/:id")]
async fn get_user(
    Path(id): Path<i32>,
    DbConn(db): DbConn
) -> Response {
    match users::Entity::find_by_id(id).one(&db).await {
        Ok(Some(user)) => json!(user),
        Ok(None) => Response::new(StatusCode::NotFound, vec![])
            .json(json!({"error": "User not found"})),
        Err(e) => Response::new(StatusCode::InternalServerError, vec![])
            .json(json!({"error": e.to_string()}))
    }
}
```

### Method 2: Request Extension

```rust
use firework_seaorm::RequestDbExt;

#[post("/users")]
async fn create_user(
    req: Request,
    Json(data): Json<CreateUserDto>
) -> Response {
    let db = match req.db() {
        Some(db) => db,
        None => return Response::new(StatusCode::InternalServerError, vec![])
            .json(json!({"error": "Database not available"}))
    };
    
    let user = users::ActiveModel {
        username: Set(data.username),
        email: Set(data.email),
        ..Default::default()
    };
    
    match user.insert(&db).await {
        Ok(user) => json!(user),
        Err(e) => Response::new(StatusCode::InternalServerError, vec![])
            .json(json!({"error": e.to_string()}))
    }
}
```

## CRUD Operations

### Create

```rust
use sea_orm::ActiveModelTrait;

#[post("/users")]
async fn create_user(
    DbConn(db): DbConn,
    Json(data): Json<CreateUserDto>
) -> Response {
    let user = users::ActiveModel {
        username: Set(data.username),
        email: Set(data.email),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    
    match user.insert(&db).await {
        Ok(model) => Response::new(StatusCode::Created, vec![]).json(model),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}
```

### Read

```rust
use sea_orm::EntityTrait;

// Find all
#[get("/users")]
async fn list_users(DbConn(db): DbConn) -> Response {
    match users::Entity::find().all(&db).await {
        Ok(users) => json!(users),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}

// Find by ID
#[get("/users/:id")]
async fn get_user(Path(id): Path<i32>, DbConn(db): DbConn) -> Response {
    match users::Entity::find_by_id(id).one(&db).await {
        Ok(Some(user)) => json!(user),
        Ok(None) => Response::new(StatusCode::NotFound, vec![])
            .json(json!({"error": "Not found"})),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}

// With filters
#[get("/users/search")]
async fn search_users(
    Query(params): Query<SearchParams>,
    DbConn(db): DbConn
) -> Response {
    match users::Entity::find()
        .filter(users::Column::Username.contains(&params.query))
        .all(&db)
        .await
    {
        Ok(users) => json!(users),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}
```

### Update

```rust
use sea_orm::ActiveModelTrait;

#[put("/users/:id")]
async fn update_user(
    Path(id): Path<i32>,
    DbConn(db): DbConn,
    Json(data): Json<UpdateUserDto>
) -> Response {
    let user = match users::Entity::find_by_id(id).one(&db).await {
        Ok(Some(user)) => user,
        Ok(None) => return Response::new(StatusCode::NotFound, vec![])
            .json(json!({"error": "Not found"})),
        Err(e) => return firework_seaorm::helpers::db_error_to_response(e)
    };
    
    let mut active_user: users::ActiveModel = user.into();
    if let Some(username) = data.username {
        active_user.username = Set(username);
    }
    if let Some(email) = data.email {
        active_user.email = Set(email);
    }
    
    match active_user.update(&db).await {
        Ok(model) => json!(model),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}
```

### Delete

```rust
use sea_orm::EntityTrait;

#[delete("/users/:id")]
async fn delete_user(Path(id): Path<i32>, DbConn(db): DbConn) -> Response {
    match users::Entity::delete_by_id(id).exec(&db).await {
        Ok(_) => Response::new(StatusCode::NoContent, vec![]),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}
```

## Transactions

```rust
use sea_orm::TransactionTrait;

#[post("/users/bulk")]
async fn create_bulk_users(
    DbConn(db): DbConn,
    Json(users_data): Json<Vec<CreateUserDto>>
) -> Response {
    // Start transaction
    let txn = match db.begin().await {
        Ok(txn) => txn,
        Err(e) => return firework_seaorm::helpers::db_error_to_response(e)
    };
    
    for user_data in users_data {
        let user = users::ActiveModel {
            username: Set(user_data.username),
            email: Set(user_data.email),
            ..Default::default()
        };
        
        if let Err(e) = user.insert(&txn).await {
            txn.rollback().await.ok();
            return firework_seaorm::helpers::db_error_to_response(e);
        }
    }
    
    // Commit transaction
    match txn.commit().await {
        Ok(_) => json!({"success": true}),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}
```

## Relationships

```rust
use sea_orm::entity::prelude::*;

// User has many Posts
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::posts::Entity")]
    Posts,
}

// Query with relations
#[get("/users/:id/posts")]
async fn get_user_posts(
    Path(id): Path<i32>,
    DbConn(db): DbConn
) -> Response {
    match users::Entity::find_by_id(id)
        .find_with_related(posts::Entity)
        .all(&db)
        .await
    {
        Ok(results) => json!(results),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}
```

## Pagination

```rust
use sea_orm::{PaginatorTrait, QuerySelect};

#[get("/users")]
async fn list_users_paginated(
    Query(params): Query<PaginationParams>,
    DbConn(db): DbConn
) -> Response {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    
    let paginator = users::Entity::find()
        .paginate(&db, per_page);
    
    let total = match paginator.num_pages().await {
        Ok(pages) => pages,
        Err(e) => return firework_seaorm::helpers::db_error_to_response(e)
    };
    
    match paginator.fetch_page(page - 1).await {
        Ok(users) => json!({
            "data": users,
            "page": page,
            "per_page": per_page,
            "total_pages": total
        }),
        Err(e) => firework_seaorm::helpers::db_error_to_response(e)
    }
}
```

## Migrations

### Using sea-orm-cli

```bash
# Install CLI
cargo install sea-orm-cli

# Generate entity from database
sea-orm-cli generate entity -o src/entity

# Create migration
sea-orm-cli migrate generate create_users_table

# Run migrations
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down
```

### Programmatic Migrations

```rust
use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    let db = Database::connect("sqlite://data.db").await.unwrap();
    
    Migrator::up(&db, None).await.unwrap();
}
```

## Best Practices

### 1. Use DTOs for Input/Output

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
}
```

### 2. Error Handling Helper

```rust
use firework_seaorm::helpers::db_error_to_response;

#[get("/users/:id")]
async fn get_user(Path(id): Path<i32>, DbConn(db): DbConn) -> Response {
    match users::Entity::find_by_id(id).one(&db).await {
        Ok(Some(user)) => json!(user),
        Ok(None) => Response::new(StatusCode::NotFound, vec![])
            .json(json!({"error": "Not found"})),
        Err(e) => db_error_to_response(e) // Helper function
    }
}
```

### 3. Middleware for DB Access

```rust
#[middleware]
async fn db_middleware(req: &mut Request, res: &mut Response) -> Flow {
    // Database connection is automatically available via DbConn extractor
    Flow::Continue
}

#[scope("/api", middleware = [db_middleware])]
mod api {
    // All routes here have DB access
}
```

## Configuration

### Environment Variables

```bash
DATABASE_URL=postgresql://user:pass@localhost/mydb
```

### Multiple Databases

```rust
// Primary database
let main_db = Arc::new(SeaOrmPlugin::new("sqlite://main.db"));
firework::register_plugin_with_name("main_db", main_db);

// Analytics database
let analytics_db = Arc::new(SeaOrmPlugin::new("sqlite://analytics.db"));
firework::register_plugin_with_name("analytics_db", analytics_db);
```

## Performance Tips

1. **Use connection pooling** (SeaORM does this by default)
2. **Select only needed columns** with `.select_only()`
3. **Use pagination** for large datasets
4. **Index frequently queried columns**
5. **Use transactions** for multiple related operations
6. **Lazy loading** for relations when needed

## See Also

- [Database Guide](../guides/database.md)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Custom Plugin Development](./custom.md)
