# 🔗 Building a REST API

Complete guide to building a RESTful API with Firework.

---

## Project Setup

```bash
cargo new todo-api
cd todo-api
```

**Cargo.toml:**
```toml
[dependencies]
firework = { git = "..." }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

---

## Data Model

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}
```

---

## In-Memory Storage

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

type TodoStore = Arc<RwLock<HashMap<Uuid, Todo>>>;

lazy_static::lazy_static! {
    static ref STORE: TodoStore = Arc::new(RwLock::new(HashMap::new()));
}
```

---

## CRUD Handlers

```rust
use firework::prelude::*;

// LIST /todos
#[get("/todos")]
async fn list_todos() -> Json<Vec<Todo>> {
    let store = STORE.read().await;
    let todos: Vec<Todo> = store.values().cloned().collect();
    Json(todos)
}

// GET /todos/:id
#[get("/todos/:id")]
async fn get_todo(Path(id): Path<Uuid>) -> Result<Json<Todo>, Error> {
    let store = STORE.read().await;
    let todo = store.get(&id)
        .cloned()
        .ok_or_else(|| Error::NotFound("Todo not found".into()))?;
    Ok(Json(todo))
}

// POST /todos
#[post("/todos")]
async fn create_todo(Json(data): Json<CreateTodo>) -> Json<Todo> {
    let todo = Todo {
        id: Uuid::new_v4(),
        title: data.title,
        completed: false,
        created_at: chrono::Utc::now(),
    };
    
    let mut store = STORE.write().await;
    store.insert(todo.id, todo.clone());
    
    Json(todo)
}

// PUT /todos/:id
#[put("/todos/:id")]
async fn update_todo(
    Path(id): Path<Uuid>,
    Json(data): Json<UpdateTodo>,
) -> Result<Json<Todo>, Error> {
    let mut store = STORE.write().await;
    
    let todo = store.get_mut(&id)
        .ok_or_else(|| Error::NotFound("Todo not found".into()))?;
    
    if let Some(title) = data.title {
        todo.title = title;
    }
    if let Some(completed) = data.completed {
        todo.completed = completed;
    }
    
    Ok(Json(todo.clone()))
}

// DELETE /todos/:id
#[delete("/todos/:id")]
async fn delete_todo(Path(id): Path<Uuid>) -> Result<Response, Error> {
    let mut store = STORE.write().await;
    
    store.remove(&id)
        .ok_or_else(|| Error::NotFound("Todo not found".into()))?;
    
    Ok(Response::new(StatusCode::NoContent, b""))
}
```

---

## Main Application

```rust
#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("🔥 Todo API running on http://127.0.0.1:8080");
    println!("📝 Endpoints:");
    println!("  GET    /todos      - List all todos");
    println!("  GET    /todos/:id  - Get one todo");
    println!("  POST   /todos      - Create todo");
    println!("  PUT    /todos/:id  - Update todo");
    println!("  DELETE /todos/:id  - Delete todo");
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
```

---

## Testing

```bash
# Create
curl -X POST http://localhost:8080/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Learn Firework"}'

# List
curl http://localhost:8080/todos

# Get one (replace ID)
curl http://localhost:8080/todos/YOUR-UUID-HERE

# Update
curl -X PUT http://localhost:8080/todos/YOUR-UUID-HERE \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# Delete
curl -X DELETE http://localhost:8080/todos/YOUR-UUID-HERE
```

---

## Next: Add Database

See [Database Guide](./database.md) to persist data.
