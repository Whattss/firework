//! 🔥 Threads Clone - Fullstack Firework + Vite

use chrono::{DateTime, Utc};
use firework::prelude::*;
use firework_vite::VitePlugin;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Thread {
    id: u64,
    author: String,
    avatar: String,
    handle: String,
    content: String,
    likes: u64,
    replies: u64,
    reposts: u64,
    timestamp: DateTime<Utc>,
    liked: bool,
    reposted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateThreadRequest {
    content: String,
}

#[derive(Clone)]
struct AppState {
    threads: Arc<RwLock<Vec<Thread>>>,
    next_id: Arc<RwLock<u64>>,
}

impl AppState {
    fn new() -> Self {
        let mut threads = Vec::new();

        threads.push(Thread {
            id: 1,
            author: "Firework".to_string(),
            avatar: "🔥".to_string(),
            handle: "@fireworkrs".to_string(),
            content: "Just launched the fastest Rust web framework. 200k+ req/s!".to_string(),
            likes: 234,
            replies: 45,
            reposts: 89,
            timestamp: Utc::now(),
            liked: false,
            reposted: false,
        });

        threads.push(Thread {
            id: 2,
            author: "Rust Dev".to_string(),
            avatar: "🦀".to_string(),
            handle: "@rustacean".to_string(),
            content: "Finally, a fullstack framework for Rust developers!".to_string(),
            likes: 567,
            replies: 78,
            reposts: 123,
            timestamp: Utc::now(),
            liked: true,
            reposted: false,
        });

        Self {
            threads: Arc::new(RwLock::new(threads)),
            next_id: Arc::new(RwLock::new(3)),
        }
    }
}

#[middleware]
fn inject_state(req: &mut Request, _res: &mut Response) -> Flow {
    static STATE: std::sync::OnceLock<AppState> = std::sync::OnceLock::new();
    let state = STATE.get_or_init(|| AppState::new());
    req.set_context(state.clone());
    Flow::Continue
}

#[get("/")]
async fn index() -> &'static str {
    "Threads Clone API"
}

#[get("/health")]
async fn health() -> &'static str {
    "OK"
}

#[get("/api/threads")]
async fn get_threads() -> Response {
    let state = unsafe {
        static mut STATE: Option<AppState> = None;
        if STATE.is_none() {
            STATE = Some(AppState::new());
        }
        STATE.as_ref().unwrap()
    };
    
    let threads = state.threads.read().await;
    let mut sorted = threads.clone();
    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    firework::json!(sorted)
}

#[post("/api/threads")]
async fn create_thread(Json(body): Json<CreateThreadRequest>) -> Response {
    let state = unsafe {
        static mut STATE: Option<AppState> = None;
        if STATE.is_none() {
            STATE = Some(AppState::new());
        }
        STATE.as_mut().unwrap()
    };

    if body.content.is_empty() || body.content.len() > 500 {
        return firework::text!("Thread must be 1-500 characters");
    }

    let id = {
        let mut next_id = state.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
    };

    let thread = Thread {
        id,
        author: "You".to_string(),
        avatar: "👤".to_string(),
        handle: "@you".to_string(),
        content: body.content,
        likes: 0,
        replies: 0,
        reposts: 0,
        timestamp: Utc::now(),
        liked: false,
        reposted: false,
    };

    state.threads.write().await.push(thread.clone());
    firework::json!(thread)
}

#[post("/api/threads/:id/like")]
async fn like_thread(Path(id): Path<u64>) -> Response {
    let state = unsafe {
        static mut STATE: Option<AppState> = None;
        if STATE.is_none() {
            STATE = Some(AppState::new());
        }
        STATE.as_mut().unwrap()
    };
    
    let mut threads = state.threads.write().await;

    if let Some(thread) = threads.iter_mut().find(|t| t.id == id) {
        if thread.liked {
            thread.likes = thread.likes.saturating_sub(1);
            thread.liked = false;
        } else {
            thread.likes += 1;
            thread.liked = true;
        }
        firework::json!(thread.clone())
    } else {
        firework::text!("Thread not found")
    }
}

#[tokio::main]
async fn main() {
    let plugin = Arc::new(
        VitePlugin::with_config(firework_vite::ViteConfig {
            root: std::path::PathBuf::from("../frontend"),
            dev_port: 5173,
            ..Default::default()
        })
        .development(),
    );
    firework::register_plugin(plugin);

    println!("🔥 Threads Clone on http://localhost:8080");

    let server = routes!().middleware(inject_state);
    server.listen("0.0.0.0:8080").await.unwrap();
}
