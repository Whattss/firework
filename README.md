# Firework

A fast, ergonomic web framework for Rust with zero-cost abstractions.

## Quick Start with CLI

```bash
# Install the CLI tool
./install_fwk.sh

# Create a new project
fwk new my-app

# Run with hot reload
cd my-app
fwk run dev --hot-reload
```

See [fwk/README.md](fwk/README.md) for complete CLI documentation.

## Features

- **Fast HTTP Server** with keep-alive support and async handlers
- **Radix Tree Routing** for efficient path matching
- **Flexible Handler Signatures** - write handlers your way
- **Middleware System** with pre/post execution and context sharing
- **Plugin Architecture** for extensibility
- **Configuration System** via TOML files
- **Database Integration** with SeaORM support
- **Streaming Responses** for large files and real-time data
- **Static File Serving** with SPA support
- **Testing Utilities** for integration tests
- **Hot Reload** for rapid development

## Quick Start

```rust
use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello, World!"
}

#[get("/users/:id")]
async fn get_user(Path(id): Path<i32>) -> Json<User> {
    Json(User::find(id).await)
}

#[tokio::main]
async fn main() {
    let mut server = Server::new("127.0.0.1:8080");
    
    routes!(server, index, get_user);
    
    server.listen().await.unwrap();
}
```

## Handler Signatures

Firework supports multiple handler signatures for maximum ergonomics:

```rust
// Simple text response
#[get("/hello")]
async fn hello() -> &'static str {
    "Hello!"
}

// JSON responses
#[post("/users")]
async fn create_user(Json(user): Json<User>) -> Json<User> {
    Json(user)
}

// With database connection
#[get("/users/:id")]
async fn get_user(db: DbConn, Path(id): Path<i32>) -> Result<Json<User>, Error> {
    let user = User::find_by_id(&db, id).await?;
    Ok(Json(user))
}

// Full request/response control
#[get("/custom")]
async fn custom(req: Request, res: Response) -> Response {
    res.text("Custom response")
}
```

## Routing

### Basic Routes

```rust
#[get("/users")]
async fn list_users() -> Json<Vec<User>> { ... }

#[post("/users")]
async fn create_user(Json(user): Json<User>) -> Json<User> { ... }

#[put("/users/:id")]
async fn update_user(Path(id): Path<i32>, Json(user): Json<User>) -> Json<User> { ... }

#[delete("/users/:id")]
async fn delete_user(Path(id): Path<i32>) -> Response { ... }
```

### Scoped Routes

```rust
#[scope("/api")]
mod api {
    #[get("/users")]
    async fn get_users() -> Json<Vec<User>> { ... }
    
    #[get("/posts")]
    async fn get_posts() -> Json<Vec<Post>> { ... }
}

routes!(server, api);
```

### Scoped Middleware

```rust
#[scope("/admin", middleware = [auth], post = [log])]
mod admin {
    #[get("/dashboard")]
    async fn dashboard() -> Response { ... }
}
```

## Middleware

```rust
#[middleware]
async fn auth(req: Request, res: Response) -> MiddlewareResult {
    if let Some(token) = req.headers.get("authorization") {
        // Validate token and add to context
        req.set_context(User::from_token(token));
        MiddlewareResult::Continue
    } else {
        MiddlewareResult::Response(res.status(401).text("Unauthorized"))
    }
}

#[middleware]
async fn cors(req: Request, res: Response) -> MiddlewareResult {
    let res = res.header("Access-Control-Allow-Origin", "*");
    MiddlewareResult::Continue
}

// Use middleware
server.use_middleware(cors);
server.use_middleware(auth);
```

### Context Sharing

```rust
#[middleware]
async fn set_user(req: Request, res: Response) -> MiddlewareResult {
    let user = User::authenticate(&req);
    req.set_context(user);
    MiddlewareResult::Continue
}

#[get("/profile")]
async fn profile(req: Request) -> Json<User> {
    let user = req.context::<User>().unwrap();
    Json(user)
}
```

## Plugins

### Using Plugins

```rust
use firework::prelude::*;
use firework_seaorm::SeaOrmPlugin;

#[plugins]
async fn configure_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(SeaOrmPlugin::new()),
    ]
}

#[tokio::main]
async fn main() {
    let mut server = Server::new("127.0.0.1:8080");
    server.register_plugins(configure_plugins().await);
    server.listen().await.unwrap();
}
```

### Creating Plugins

```rust
use firework::plugin::{Plugin, PluginContext};

pub struct MyPlugin;

#[async_trait::async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }
    
    async fn on_server_start(&self, ctx: &PluginContext) {
        println!("Plugin started!");
    }
    
    async fn on_request(&self, req: &Request) {
        println!("Request received");
    }
}
```

## Database (SeaORM)

```rust
use firework::prelude::*;
use firework_seaorm::*;

#[model(table = "users")]
pub struct User {
    #[primary]
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[get("/users")]
async fn list_users(db: DbConn) -> Json<Vec<User>> {
    Json(User::all(&db).await.unwrap())
}

#[post("/users")]
async fn create_user(db: DbConn, Json(user): Json<User>) -> Json<User> {
    Json(user.insert(&db).await.unwrap())
}
```

## Configuration

Create a `Firework.toml` file:

```toml
[server]
address = "127.0.0.1"
port = 8080
workers = 8

[plugins.seaorm]
database_url = "sqlite://data.db"
pool_max = 10
```

Access configuration in code:

```rust
let config = Config::load("Firework.toml").unwrap();
let port = config.get::<u16>("server.port").unwrap();
```

## Streaming

```rust
#[get("/video")]
async fn stream_video() -> Response {
    let file = tokio::fs::File::open("video.mp4").await.unwrap();
    stream!(file)
}

#[get("/events")]
async fn sse() -> Response {
    stream!(async_stream::stream! {
        for i in 0..10 {
            yield format!("data: {}\n\n", i);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}
```

## Static Files

```rust
// Serve static files
server.serve("/static", "./public");

// SPA mode (fallback to index.html)
server.spa("/", "./dist");
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use firework::testing::*;
    
    #[tokio::test]
    async fn test_hello() {
        let mut server = TestServer::new();
        server.route(get("/hello", hello));
        
        let res = server.get("/hello").await;
        assert_eq!(res.status(), 200);
        assert_eq!(res.text(), "Hello!");
    }
}
```

## Response Helpers

```rust
// Text response
res.text("Hello")

// JSON response
res.json(&user)

// HTML response
res.html("<h1>Hello</h1>")

// Status code
res.status(404).text("Not found")

// Headers
res.header("X-Custom", "value")

// Redirect
res.redirect("/login")
```

## Extractors

```rust
// Path parameters
#[get("/users/:id")]
async fn get_user(Path(id): Path<i32>) -> Json<User> { ... }

// Query parameters
#[get("/search")]
async fn search(Query(params): Query<SearchParams>) -> Json<Vec<Result>> { ... }

// JSON body
#[post("/users")]
async fn create(Json(user): Json<User>) -> Json<User> { ... }

// Headers
#[get("/auth")]
async fn auth(Headers(headers): Headers) -> Response { ... }

// Full request
#[get("/custom")]
async fn custom(req: Request) -> Response { ... }
```

## Hot Reload (Development)

For rapid development, use the `firework-dev` tool to automatically rebuild and restart your server on file changes:

```bash
# Install the dev tool
cargo install --path . --features hot-reload

# Run with hot reload
firework-dev                    # main app
firework-dev hello_world        # specific example
```

Your application code stays clean - no special hot reload setup needed:

```rust
#[tokio::main]
async fn main() {
    let server = Server::new();
    routes!(server => [index, api])
        .listen("127.0.0.1:8080")
        .await
        .expect("Server failed");
}
```

See `HOT_RELOAD.md` for more details.

## Examples

See the `firework/examples` directory for complete working examples:

- `flexible_handlers.rs` - Various handler signatures
- `complete_static_server.rs` - Static file serving and SPA
- `streaming_example.rs` - Streaming responses
- `seaorm_example.rs` - Database integration
- `plugin_macro.rs` - Plugin development
- `testing_example.rs` - Integration testing
- `config_example.rs` - Configuration system
- `hot_reload_example.rs` - Hot reload development

## License

MIT
