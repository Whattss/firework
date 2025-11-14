use firework::prelude::*;
use firework_proxy::{ProxyBuilder, ProxiedResponse, ProxyError};
use std::sync::Arc;

#[get("/api/hello")]
async fn api_hello() -> &'static str {
    "Hello from Firework API!"
}

#[get("/api/status")]
async fn api_status() -> Response {
    json!(serde_json::json!({
        "status": "ok",
        "service": "firework"
    }))
}

#[get("/*")]
async fn proxy_handler(req: Request, mut res: Response) -> Response {
    if let Some(ProxiedResponse(proxied)) = req.get_context::<ProxiedResponse>() {
        return proxied;
    }
    
    if let Some(ProxyError(err)) = req.get_context::<ProxyError>() {
        eprintln!("[Proxy Error] {:?}", err);
        return Response::new(StatusCode::BadGateway, b"Backend unavailable");
    }
    
    res.status = StatusCode::NotFound;
    res.set_body(b"Not found".to_vec());
    res
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 Firework Reverse Proxy Example\n");
    
    let proxy = Arc::new(
        ProxyBuilder::new()
            .route("/", "http://localhost:3001")
            .add_header("X-Proxied-By", "Firework")
            .build()
    );
    
    register_plugin(proxy);
    
    println!("📚 Routes:");
    println!("  /api/*  → Firework");
    println!("  /*      → http://localhost:3001\n");
    
    println!("💡 Start backend: python3 -m http.server 3001\n");
    println!("🚀 Proxy: http://localhost:8080\n");
    
    routes!().listen("127.0.0.1:8080").await?;
    
    Ok(())
}
