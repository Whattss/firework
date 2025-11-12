use firework::{Error, Json, Path, Request, Response, Result, Server, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
}

// Example 1: Simple string response
#[firework::get("/")]
async fn hello() -> &'static str {
    "Hello, World!"
}

// Example 2: JSON response
#[firework::get("/api/status")]
async fn status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": "1.0.0"
    }))
}

// Example 3: Path parameter
#[firework::get("/users/:id")]
async fn get_user(Path(id): Path<i32>) -> Result<Json<User>> {
    if id == 1 {
        Ok(Json(User {
            id,
            username: "john_doe".to_string(),
            email: "john@example.com".to_string(),
        }))
    } else {
        Err(Error::NotFound(format!("User {} not found", id)))
    }
}

// Example 4: JSON body extraction
#[firework::post("/users")]
async fn create_user(Json(user): Json<User>) -> Json<User> {
    // In a real app, this would save to database
    Json(user)
}

// Example 5: Standard signature (still supported)
#[firework::get("/legacy")]
async fn legacy_handler(req: Request, mut res: Response) -> Response {
    res.status(StatusCode::Ok);
    res.set_body(b"Legacy handler".to_vec());
    res
}

// Example 6: Multiple extractors
#[firework::put("/users/:id")]
async fn update_user(Path(id): Path<i32>, Json(mut user): Json<User>) -> Result<Json<User>> {
    user.id = id;
    Ok(Json(user))
}

// Example 7: Error handling
#[firework::get("/error")]
async fn error_handler() -> Result<&'static str> {
    Err(Error::BadRequest("Something went wrong".into()))
}

// Example 8: Using text! macro
#[firework::get("/text")]
async fn text_response() -> Response {
    firework::text!("This is plain text")
}

// Example 9: Using json! macro
#[firework::get("/json")]
async fn json_response() -> Response {
    firework::json!(serde_json::json!({"message": "Hello from macro"}))
}

// Example with scope and extractors
#[firework::scope("/api/v1")]
mod api {
    use super::*;

    #[firework::get("/users")]
    async fn list_users() -> Json<Vec<User>> {
        Json(vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ])
    }

    #[firework::get("/users/:id")]
    async fn get_user_v1(Path(id): Path<i32>) -> Result<Json<User>> {
        if id <= 2 {
            Ok(Json(User {
                id,
                username: format!("user_{}", id),
                email: format!("user{}@example.com", id),
            }))
        } else {
            Err(Error::NotFound(format!("User {} not found", id)))
        }
    }

    #[firework::post("/users")]
    async fn create_user_v1(Json(user): Json<User>) -> Json<User> {
        Json(user)
    }
}

#[tokio::main]
async fn main() {
    println!("Starting Firework server with flexible handlers...");

    let server = firework::routes!();

    server
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
