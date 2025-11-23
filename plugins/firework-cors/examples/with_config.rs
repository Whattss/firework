use firework::prelude::*;
use firework_cors::CorsPlugin;
use std::sync::Arc;

#[get("/")]
async fn index() -> Response {
    html!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>CORS with Config Example</title>
</head>
<body>
    <h1>Firework CORS - Config Example</h1>
    <p>This server loads CORS configuration from Firework.toml</p>
    
    <h2>Test CORS:</h2>
    <pre id="result">Waiting...</pre>
    
    <script>
        fetch('/api/data')
            .then(r => r.json())
            .then(data => {
                document.getElementById('result').textContent = JSON.stringify(data, null, 2);
            })
            .catch(err => {
                document.getElementById('result').textContent = 'Error: ' + err;
            });
    </script>
</body>
</html>
    "#)
}

#[get("/api/data")]
async fn get_data() -> Response {
    json!({
        "message": "Hello from Firework!",
        "cors": "configured from Firework.toml",
        "timestamp": chrono::Utc::now().timestamp()
    })
}

#[get("/api/config")]
async fn get_config() -> Response {
    json!({
        "info": "CORS is configured from Firework.toml",
        "see": "examples/Firework.toml for configuration example"
    })
}

#[tokio::main]
async fn main() {
    println!("🔥 Firework CORS - Config Example");
    println!("");
    
    // Initialize config (looks for Firework.toml in current directory)
    firework::init_config("Firework.toml").await.ok();
    
    // Load CORS plugin from config
    // This reads [plugins.cors] section from Firework.toml
    let cors = CorsPlugin::from_config().await;
    
    firework::register_plugin(Arc::new(cors));
    
    println!("✅ CORS plugin loaded from Firework.toml");
    println!("");
    println!("Endpoints:");
    println!("  http://localhost:8080/");
    println!("  http://localhost:8080/api/data");
    println!("  http://localhost:8080/api/config");
    println!("");
    println!("Configuration: See examples/Firework.toml");
    println!("");
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
