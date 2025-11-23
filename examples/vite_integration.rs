//! Vite Integration Example
//! 
//! Demonstrates fullstack development with Firework + Vite.
//! 
//! # Setup
//! 
//! 1. Create frontend directory:
//!    ```bash
//!    mkdir frontend && cd frontend
//!    npm create vite@latest . -- --template react-ts
//!    npm install
//!    ```
//! 
//! 2. Run this example:
//!    ```bash
//!    cargo run --example vite_integration
//!    ```
//! 
//! 3. Visit http://localhost:8080
//!    - Frontend served from Vite (HMR enabled)
//!    - API routes handled by Firework
//! 
//! # How it works
//! 
//! - VitePlugin auto-starts Vite dev server on :5173
//! - All non-/api requests are proxied to Vite
//! - API routes handled by Firework backend
//! - Changes to frontend reflect instantly (HMR)

use firework::prelude::*;
use firework_vite::{VitePlugin, vite_auto_middleware};
use std::sync::Arc;

// API Routes (handled by Firework)
#[get("/api/health")]
async fn health() -> &'static str {
    "OK"
}

#[get("/api/users")]
async fn get_users() -> Json<Vec<User>> {
    Json(vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ])
}

#[get("/api/users/:id")]
async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(User { id, name: format!("User {}", id) })
}

#[derive(serde::Serialize)]
struct User {
    id: u32,
    name: String,
}

#[tokio::main]
async fn main() {
    println!("🔥 Firework + Vite Fullstack Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Create and register Vite plugin
    let vite = Arc::new(VitePlugin::new());
    register_plugin(vite.clone());
    
    println!("\n📋 How it works:");
    println!("  - Frontend: Proxied to Vite dev server (:5173)");
    println!("  - API routes (/api/*): Handled by Firework");
    println!("  - HMR: Enabled (edit frontend code and see changes)");
    
    println!("\n🌐 Endpoints:");
    println!("  GET  /              → Vite (index.html)");
    println!("  GET  /api/health    → Firework backend");
    println!("  GET  /api/users     → Firework backend");
    println!("  GET  /api/users/:id → Firework backend");
    
    println!("\n⚡ Starting server...\n");
    
    routes!()
        .async_middleware(vite_auto_middleware(vite))
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
