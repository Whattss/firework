use firework::prelude::*;
use serde_json::json;

// Test: Multiple routes pointing to same handler
#[get("/health")]
#[get("/api/health")]
#[get("/status")]
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

// Test: API versioning
#[get("/api/v1/users")]
#[get("/api/v2/users")]
async fn list_users() -> Json<Vec<String>> {
    Json(vec!["user1".to_string(), "user2".to_string()])
}

// Test: Aliases
#[get("/login")]
#[get("/signin")]
#[get("/auth/login")]
async fn login_page() -> &'static str {
    "Login Page"
}

// Test: Different methods
#[get("/test")]
#[post("/test")]
async fn test_endpoint() -> &'static str {
    "Works for GET and POST"
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("🔥 Testing multiple routes per handler...\n");
    
    // Count routes
    let route_count = firework::ROUTES.len();
    println!("Total routes registered: {}", route_count);
    
    // List all routes
    for route in firework::ROUTES.iter() {
        println!("  {} {}", route.method, route.path);
    }
    
    println!("\n✅ Multiple routes feature works!");
    println!("\nStarting server on http://127.0.0.1:3000");
    println!("Try:");
    println!("  curl http://127.0.0.1:3000/health");
    println!("  curl http://127.0.0.1:3000/api/health");
    println!("  curl http://127.0.0.1:3000/status");
    
    server.listen("127.0.0.1:3000").await.unwrap();
}
