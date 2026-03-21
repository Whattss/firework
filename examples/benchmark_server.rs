use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello, World!"
}

#[get("/json")]
async fn json_endpoint() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Hello, World!",
        "status": "ok",
        "timestamp": 1234567890
    }))
}

#[get("/users/:id")]
async fn user(Path(id): Path<u32>) -> String {
    format!("User {}", id)
}

#[get("/query")]
async fn query_test() -> &'static str {
    "Query test"
}

#[tokio::main]
async fn main() {
    let server = routes!();

    println!("🚀 Benchmark server running on http://127.0.0.1:8080");
    println!("Endpoints:");
    println!("  GET /           - Simple text");
    println!("  GET /json       - JSON response");
    println!("  GET /users/:id  - Path params");
    println!("  GET /query      - Query params");

    server.listen("127.0.0.1:8080").await.unwrap();
}
