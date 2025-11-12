// Hot Reload Example
//
// To use hot reload with automatic rebuilding:
//
//   1. Install firework-dev:
//      cargo install --path firework --features hot-reload
//
//   2. Run with hot reload:
//      firework-dev hot_reload_example
//
//   3. Edit this file and save - the server will rebuild and restart automatically!
//
// Features demonstrated:
//   - Automatic rebuild on file changes
//   - Ignores editor temp files (vim, neovim, etc.)
//   - Graceful process management
//   - State preservation (optional)

use firework::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

// This counter will reset on each reload (normal behavior)
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

#[get("/")]
async fn index(_req: Request, res: Response) -> Response {
    let count = REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);

    res.text(format!(
        "Hello from hot reload v3.0!\n\
        Change this text and save to see it reload instantly.\n\
        Request count since last reload: {}\n\n\
        Try editing the message above!\n\n\
        Happy coding with Firework!",
        count
    ))
}

#[get("/api/hello")]
async fn hello(_req: Request, res: Response) -> Response {
    res.json(serde_json::json!({
        "message": "Hot reload is working perfectly!",
        "version": "3.0.0",
        "tip": "Edit this JSON and save to see changes"
    }))
}

#[get("/api/status")]
async fn status(_req: Request, res: Response) -> Response {
    res.json(serde_json::json!({
        "status": "running",
        "hot_reload": true,
        "features": [
            "Auto-rebuild on .rs/.toml changes",
            "Ignores editor temp files",
            "Process management",
            "Graceful shutdown"
        ]
    }))
}

#[middleware]
fn logger(req: Request, res: Response) -> Flow {
    println!(
        "[{:?}] {} - Request #{}",
        req.method,
        req.uri.path,
        REQUEST_COUNT.load(Ordering::SeqCst)
    );
    Flow::Next(req, res)
}

#[tokio::main]
async fn main() {
    routes!(Server::new() => [
        index,
        hello,
        status,
    ])
    .middleware(logger)
    .listen("127.0.0.1:8080")
    .await
    .expect("Failed to start server");
}
