pub fn get_config_template() -> &'static str {
    r#"[server]
address = "127.0.0.1"
port = 8080
workers = 4

[scripts]
dev = "cargo run"
build = "cargo build --release"
test = "cargo test"

# [plugins.seaorm]
# database_url = "sqlite://data.db"
"#
}

pub fn get_cargo_template(name: &str, template: &str) -> String {
    match template {
        "basic" => format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
firework = {{ git = "https://github.com/whattss/firework", features = [] }}
tokio = {{ version = "1", features = ["full"] }}
linkme = "0.3"
"#,
            name
        ),
        "api" => format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
firework = {{ git = "https://github.com/whattss/firework", features = [] }}
firework-seaorm = {{ git = "https://github.com/whattss/firework" }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
linkme = "0.3"
"#,
            name
        ),
        "fullstack" => format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
firework = {{ git = "https://github.com/whattss/firework", features = [] }}
firework-seaorm = {{ git = "https://github.com/whattss/firework" }}
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
linkme = "0.3"
"#,
            name
        ),
        _ => panic!("Unknown template"),
    }
}

pub fn get_main_template(template: &str) -> &'static str {
    match template {
        "basic" => r#"use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello, Firework!"
}

#[get("/hello/:name")]
async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("Server running on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await.unwrap();
}
"#,
        "api" => r#"use firework::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[middleware]
async fn cors(req: Request, res: Response) -> Flow {
    let mut res = res;
    res.headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
    res.headers.insert("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE".to_string());
    res.headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type".to_string());
    Flow::Next(req, res)
}

#[get("/api/users")]
async fn get_users() -> impl Responder {
    let users = vec![
        User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
        User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
    ];
    Json(users)
}

#[get("/api/users/:id")]
async fn get_user(Path(id): Path<u32>) -> impl Responder {
    let user = User {
        id,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    Json(user)
}

#[post("/api/users")]
async fn create_user(Json(user): Json<User>) -> impl Responder {
    Json(user)
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("API server listening on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await.unwrap();
}
"#,
        "fullstack" => r#"use firework::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    username: String,
    email: String,
}

#[middleware]
async fn cors(req: Request, res: Response) -> Flow {
    let mut res = res;
    res.headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
    Flow::Next(req, res)
}

#[get("/")]
async fn index() -> Response {
    serve_file("static/index.html").await
}

#[scope("/api")]
mod api {
    use super::*;
    
    #[get("/users")]
    async fn get_users() -> impl IntoResponse {
        let users = vec![
            User { id: 1, username: "alice".to_string(), email: "alice@example.com".to_string() },
            User { id: 2, username: "bob".to_string(), email: "bob@example.com".to_string() },
        ];
        Json(users)
    }
    
    #[post("/users")]
    async fn create_user(Json(user): Json<User>) -> impl IntoResponse {
        Json(user)
    }
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("Server listening on http://127.0.0.1:8080");
    server.listen("127.0.0.1:8080").await.unwrap();
}
"#,
        _ => panic!("Unknown template"),
    }
}
