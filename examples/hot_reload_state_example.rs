// Advanced Hot Reload Example with State Preservation
//
// This example demonstrates how to preserve state across hot reloads,
// useful for maintaining database connections, caches, etc.
//
// Run with:
//   firework-dev hot_reload_state_example

use firework::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
struct AppState {
    db_connection_count: Arc<AtomicU64>,
    cache_hits: Arc<AtomicU64>,
}

impl AppState {
    fn new() -> Self {
        Self {
            db_connection_count: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
        }
    }
    
    fn restore_or_new() -> Self {
        #[cfg(feature = "hot-reload")]
        {
            use firework::hot_reload_state;
            
            if let Some(state) = hot_reload_state::restore_state::<Self>("app_state") {
                println!("Restored state from previous reload!");
                return state;
            }
        }
        
        println!("Creating new state...");
        Self::new()
    }
    
    fn preserve(&self) {
        #[cfg(feature = "hot-reload")]
        {
            use firework::hot_reload_state;
            hot_reload_state::preserve_state("app_state", self.clone());
        }
    }
}

#[get("/")]
async fn index(_req: Request, res: Response) -> Response {
    res.text(
        "Hot Reload State Preservation Example\n\
        \n\
        Try these endpoints:\n\
        - GET /api/stats - View current stats\n\
        - POST /api/db - Simulate DB connection\n\
        - POST /api/cache - Simulate cache hit\n\
        \n\
        The counters will persist across hot reloads!"
    )
}

#[get("/api/stats")]
async fn stats(req: Request, res: Response) -> Response {
    let state = req.context::<AppState>().unwrap();
    
    res.json(serde_json::json!({
        "db_connections": state.db_connection_count.load(Ordering::SeqCst),
        "cache_hits": state.cache_hits.load(Ordering::SeqCst),
        "message": "These stats persist across hot reloads!"
    }))
}

#[post("/api/db")]
async fn db_connect(req: Request, res: Response) -> Response {
    let state = req.context::<AppState>().unwrap();
    let count = state.db_connection_count.fetch_add(1, Ordering::SeqCst) + 1;
    
    res.json(serde_json::json!({
        "message": "DB connection simulated",
        "total_connections": count
    }))
}

#[post("/api/cache")]
async fn cache_hit(req: Request, res: Response) -> Response {
    let state = req.context::<AppState>().unwrap();
    let count = state.cache_hits.fetch_add(1, Ordering::SeqCst) + 1;
    
    res.json(serde_json::json!({
        "message": "Cache hit simulated",
        "total_hits": count
    }))
}

#[middleware]
fn inject_state(mut req: Request, res: Response) -> Flow {
    // In a real app, you'd initialize this once and share it
    static STATE: std::sync::OnceLock<AppState> = std::sync::OnceLock::new();
    
    let state = STATE.get_or_init(|| AppState::restore_or_new());
    req.set_context(state.clone());
    
    Flow::Next(req, res)
}

#[tokio::main]
async fn main() {
    // Restore state on startup
    let state = AppState::restore_or_new();
    
    // Set up shutdown hook to preserve state
    #[cfg(feature = "hot-reload")]
    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            state_clone.preserve();
        });
    }
    
    let server = Server::new();

    routes!(server => [
        index,
        stats,
        db_connect,
        cache_hit,
    ])
    .middleware(inject_state)
    .listen("127.0.0.1:8080")
    .await
    .expect("Failed to start server");
}
