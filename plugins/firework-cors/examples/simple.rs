use firework::prelude::*;
use firework_cors::CorsPlugin;
use std::sync::Arc;

#[get("/")]
async fn index() -> Response {
    html!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>CORS Example</title>
</head>
<body>
    <h1>Firework CORS Example</h1>
    <p>This API has CORS enabled.</p>
    <p>Try fetching from a different origin:</p>
    <pre>
fetch('http://localhost:8080/api/data')
  .then(r => r.json())
  .then(console.log)
    </pre>
</body>
</html>
    "#)
}

#[get("/api/data")]
async fn get_data() -> Response {
    json!({
        "message": "Hello from Firework!",
        "cors": "enabled",
        "timestamp": chrono::Utc::now().timestamp()
    })
}

#[post("/api/data")]
async fn post_data(req: Request) -> Response {
    json!({
        "message": "Data received",
        "body_length": req.body.len()
    })
}

#[get("/api/users")]
async fn get_users() -> Response {
    json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"},
            {"id": 3, "name": "Charlie"}
        ]
    })
}

#[tokio::main]
async fn main() {
    println!("🔥 Firework CORS Example");
    println!("");
    
    // Register CORS plugin
    // For development: allow all origins
    let cors = CorsPlugin::permissive();
    
    // For production, use specific origins:
    // let cors = CorsPlugin::new()
    //     .allow_origin("https://myapp.com")
    //     .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    //     .allow_headers(vec!["Content-Type", "Authorization"])
    //     .allow_credentials(true);
    
    firework::register_plugin(Arc::new(cors));
    
    println!("✅ CORS plugin registered");
    println!("");
    println!("Try these endpoints:");
    println!("  http://localhost:8080/");
    println!("  http://localhost:8080/api/data");
    println!("  http://localhost:8080/api/users");
    println!("");
    println!("Test CORS from browser console:");
    println!(r#"  fetch('http://localhost:8080/api/data').then(r => r.json()).then(console.log)"#);
    println!("");
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
