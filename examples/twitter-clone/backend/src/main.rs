//! 🔥 Twitter Clone - Fullstack Firework + Vite

use firework::{get, post, scope, routes, Flow, Request, Response, Method};
use firework_vite::VitePlugin;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tweet {
    id: u64,
    author: String,
    avatar: String,
    handle: String,
    content: String,
    likes: u64,
    retweets: u64,
    replies: u64,
    timestamp: DateTime<Utc>,
    liked: bool,
    retweeted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateTweetRequest {
    content: String,
}

#[derive(Clone)]
struct AppState {
    tweets: Arc<RwLock<Vec<Tweet>>>,
    next_id: Arc<RwLock<u64>>,
}

impl AppState {
    fn new() -> Self {
        let mut tweets = Vec::new();
        
        tweets.push(Tweet {
            id: 1,
            author: "Firework".to_string(),
            avatar: "🔥".to_string(),
            handle: "@fireworkrs".to_string(),
            content: "Just launched the fastest Rust web framework. 200k+ req/s. Zero compromises. 🚀".to_string(),
            likes: 234,
            retweets: 89,
            replies: 45,
            timestamp: Utc::now(),
            liked: false,
            retweeted: false,
        });

        tweets.push(Tweet {
            id: 2,
            author: "Rust Dev".to_string(),
            avatar: "🦀".to_string(),
            handle: "@rustacean".to_string(),
            content: "Finally, a fullstack framework that doesn't force me to use Node.js on the backend. This is what we needed.".to_string(),
            likes: 567,
            retweets: 123,
            replies: 78,
            timestamp: Utc::now(),
            liked: true,
            retweeted: false,
        });

        tweets.push(Tweet {
            id: 3,
            author: "Web Dev".to_string(),
            avatar: "💻".to_string(),
            handle: "@webdev".to_string(),
            content: "Benchmarked Firework vs everything else. The numbers don't lie. This changes everything.".to_string(),
            likes: 891,
            retweets: 234,
            replies: 156,
            timestamp: Utc::now(),
            liked: false,
            retweeted: true,
        });

        Self {
            tweets: Arc::new(RwLock::new(tweets)),
            next_id: Arc::new(RwLock::new(4)),
        }
    }
}

fn inject_state(req: &mut Request, _res: &mut Response) -> Flow {
    static STATE: std::sync::OnceLock<AppState> = std::sync::OnceLock::new();
    let state = STATE.get_or_init(|| AppState::new());
    req.set_context(state.clone());
    Flow::Continue
}

#[scope("/api")]
mod api {
    use super::*;
    
    #[get("/tweets")]
    async fn get_tweets(req: Request, res: Response) -> Response {
        let state = req.get_context::<AppState>().unwrap();
        let tweets = state.tweets.read().await;
        let mut sorted = tweets.clone();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        firework::json!(sorted)
    }

    #[post("/tweets")]
    async fn create_tweet(req: Request, res: Response) -> Response {
        let state = req.get_context::<AppState>().unwrap();
        
        let body: CreateTweetRequest = match serde_json::from_slice(&req.body) {
            Ok(b) => b,
            Err(_) => return firework::text!("Invalid JSON"),
        };
        
        if body.content.is_empty() || body.content.len() > 280 {
            return firework::text!("Tweet must be 1-280 characters");
        }
        
        let id = {
            let mut next_id = state.next_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let tweet = Tweet {
            id,
            author: "You".to_string(),
            avatar: "👤".to_string(),
            handle: "@you".to_string(),
            content: body.content,
            likes: 0,
            retweets: 0,
            replies: 0,
            timestamp: Utc::now(),
            liked: false,
            retweeted: false,
        };
        
        state.tweets.write().await.push(tweet.clone());
        firework::json!(tweet)
    }

    #[post("/tweets/:id/like")]
    async fn like_tweet(req: Request, res: Response) -> Response {
        let state = req.get_context::<AppState>().unwrap();
        
        let id: u64 = match req.param("id").and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => return firework::text!("Invalid ID"),
        };
        
        let mut tweets = state.tweets.write().await;
        
        if let Some(tweet) = tweets.iter_mut().find(|t| t.id == id) {
            if tweet.liked {
                tweet.likes = tweet.likes.saturating_sub(1);
                tweet.liked = false;
            } else {
                tweet.likes += 1;
                tweet.liked = true;
            }
            firework::json!(tweet.clone())
        } else {
            firework::text!("Tweet not found")
        }
    }

    #[post("/tweets/:id/retweet")]
    async fn retweet_tweet(req: Request, res: Response) -> Response {
        let state = req.get_context::<AppState>().unwrap();
        
        let id: u64 = match req.param("id").and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => return firework::text!("Invalid ID"),
        };
        
        let mut tweets = state.tweets.write().await;
        
        if let Some(tweet) = tweets.iter_mut().find(|t| t.id == id) {
            if tweet.retweeted {
                tweet.retweets = tweet.retweets.saturating_sub(1);
                tweet.retweeted = false;
            } else {
                tweet.retweets += 1;
                tweet.retweeted = true;
            }
            firework::json!(tweet.clone())
        } else {
            firework::text!("Tweet not found")
        }
    }
}

#[tokio::main]
async fn main() {
    // Register Vite plugin
    let plugin = Arc::new(
        VitePlugin::with_config(firework_vite::ViteConfig {
            root: std::path::PathBuf::from("../frontend"),
            dev_port: 5173,
            ..Default::default()
        })
        .development()
    );
    firework::register_plugin(plugin);
    
    println!("🔥 Twitter Clone running on http://localhost:8080");
    println!("📱 Frontend: http://localhost:5173 (Vite HMR)");
    println!("🔌 API: http://localhost:8080/api");
    
    let server = routes!().middleware(inject_state);
    
    server.listen("0.0.0.0:8080").await.unwrap();
}
