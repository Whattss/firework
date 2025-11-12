use firework::{get, middleware, routes, scope, Flow, Request, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Clone, Debug)]
struct AuthUser {
    user_id: u32,
    username: String,
}

#[derive(Clone, Debug)]
struct RequestId(String);

#[derive(Clone, Debug)]
struct Timing {
    start_ms: u128,
}

// Global routes
#[get("/")]
async fn home(_req: Request, _res: Response) -> Response {
    firework::html!(
        r#"
        <h1>Firework - Advanced Middlewares</h1>
        <h2>Features:</h2>
        <ul>
            <li>Async middlewares</li>
            <li>Request context sharing</li>
            <li>Separate pre/post middleware lists</li>
        </ul>
        <h2>Routes:</h2>
        <ul>
            <li>GET /public - Public (no auth)</li>
            <li>GET /api/profile - User profile (auth required)</li>
            <li>POST /api/users - Create user (auth + validation)</li>
        </ul>
    "#
    )
}

#[get("/public")]
async fn public(_req: Request, _res: Response) -> Response {
    println!("  [HANDLER] Public route");
    firework::text!("Public route - no authentication")
}

// Async middleware: Auth
#[middleware]
async fn async_auth(mut req: Request, res: Response) -> Flow {
    println!("[ASYNC AUTH PRE] Checking token...");

    // Simulate async operation (e.g., database lookup)
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    if let Some(token) = req.header("Authorization") {
        if token.starts_with("Bearer ") {
            let user_id = token
                .strip_prefix("Bearer ")
                .unwrap()
                .parse::<u32>()
                .unwrap_or(1);

            // Store auth user in context
            let auth_user = AuthUser {
                user_id,
                username: format!("user_{}", user_id),
            };
            req.set_context(auth_user);

            println!("[ASYNC AUTH PRE] User authenticated: user_{}", user_id);
            return Flow::Next(req, res);
        }
    }

    println!("[ASYNC AUTH PRE] Unauthorized");
    Flow::Stop(firework::Error::Unauthorized("Token required".into()).into_response())
}

// Sync middleware: Request ID generator
#[middleware]
fn request_id(mut req: Request, res: Response) -> Flow {
    let id = format!("req_{}", rand::random::<u32>());
    println!("[REQUEST_ID PRE] Generated: {}", id);

    req.set_context(RequestId(id));
    Flow::Next(req, res)
}

// Sync middleware: Timing start
#[middleware]
fn timing_start(mut req: Request, res: Response) -> Flow {
    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    println!("[TIMING PRE] Request started at {}ms", start);
    req.set_context(Timing { start_ms: start });

    Flow::Next(req, res)
}

// Async middleware: Response logger (POST)
#[middleware(post)]
async fn async_response_logger(req: Request, res: Response) -> Flow {
    println!("[ASYNC RESPONSE POST] Processing response...");

    // Simulate async logging (e.g., to external service)
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

    if let Some(req_id) = req.get_context::<RequestId>() {
        println!("[ASYNC RESPONSE POST] Request ID: {}", req_id.0);
    }

    if let Some(user) = req.get_context::<AuthUser>() {
        println!(
            "[ASYNC RESPONSE POST] User: {} (id: {})",
            user.username, user.user_id
        );
    }

    println!(
        "[ASYNC RESPONSE POST] Status: {}, Body size: {}",
        res.status.code(),
        res.body.len().map(|l| l.to_string()).unwrap_or_else(|| "streaming".to_string())
    );

    Flow::Next(req, res)
}

// Sync middleware: Timing end (POST)
#[middleware(post)]
fn timing_end(req: Request, res: Response) -> Flow {
    let end = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    if let Some(timing) = req.get_context::<Timing>() {
        let duration = end - timing.start_ms;
        println!("[TIMING POST] Request took {}ms", duration);
    }

    Flow::Next(req, res)
}

// API Module with separate pre/post middlewares
#[scope("/api", middleware = [async_auth, request_id, timing_start], post = [async_response_logger, timing_end])]
mod api {
    use super::*;

    #[get("/profile")]
    async fn get_profile(req: Request, _res: Response) -> Response {
        println!("  [HANDLER] Getting profile");

        // Access auth user from context
        if let Some(user) = req.get_context::<AuthUser>() {
            println!("  [HANDLER] Authenticated user: {}", user.username);

            return firework::json!(serde_json::json!({
                "user_id": user.user_id,
                "username": user.username,
                "email": format!("{}@example.com", user.username)
            }));
        }

        firework::Error::Unauthorized("Not authenticated".into()).into_response()
    }

    #[post("/users")]
    async fn create_user(req: Request, _res: Response) -> Response {
        println!("  [HANDLER] Creating user");

        // Access request ID from context
        if let Some(req_id) = req.get_context::<RequestId>() {
            println!("  [HANDLER] Request ID: {}", req_id.0);
        }

        match req.body_str() {
            Ok(body) if !body.is_empty() => match serde_json::from_str::<User>(body) {
                Ok(user) => {
                    firework::json!(
                        firework::StatusCode::Created,
                        serde_json::json!({
                            "message": "User created",
                            "user": user
                        })
                    )
                }
                Err(e) => {
                    firework::Error::BadRequest(format!("Invalid JSON: {}", e)).into_response()
                }
            },
            _ => firework::Error::BadRequest("Empty body".into()).into_response(),
        }
    }
}

#[tokio::main]
async fn main() {
    let server = routes!();

    println!("Listening on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await.unwrap();
}
