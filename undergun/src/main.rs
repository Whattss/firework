mod auth;
mod entities;
mod handlers;
mod models;

use firework::{get, routes, scope, Flow, Json, Method, Request, Response, Server};
use std::sync::Arc;

fn cors_middleware(req: Request, res: Response) -> Flow {
    let res = res
        .with_header("Access-Control-Allow-Origin", "*")
        .with_header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .with_header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        );

    if matches!(req.method, Method::OPTIONS) {
        return Flow::Stop(res);
    }

    Flow::Next(req, res)
}

#[get("/health")]
async fn health_check() -> Response {
    firework::text!("Hello")
}

#[scope("/api/auth")]
mod api_auth {
    use super::*;
    use firework::{get, post};

    #[post("/register")]
    async fn register(req: Request, res: Response) -> Response {
        handlers::auth::register(req, res).await
    }

    #[post("/login")]
    async fn login(req: Request, res: Response) -> Response {
        handlers::auth::login(req, res).await
    }

    #[get("/me", middleware = [super::auth::auth_middleware])]
    async fn me(req: Request, res: Response) -> Response {
        handlers::auth::me(req, res).await
    }
}

#[scope("/api/posts")]
mod api_posts {
    use super::*;
    use firework::{delete, get, post};

    #[get("/")]
    async fn list(req: Request, res: Response) -> Response {
        handlers::posts::list_posts(req, res).await
    }

    #[get("/:id")]
    async fn get(req: Request, res: Response) -> Response {
        handlers::posts::get_post(req, res).await
    }

    #[post("/", middleware = [super::auth::auth_middleware])]
    async fn create(req: Request, res: Response) -> Response {
        handlers::posts::create_post(req, res).await
    }

    #[delete("/:id", middleware = [super::auth::auth_middleware])]
    async fn delete(req: Request, res: Response) -> Response {
        handlers::posts::delete_post(req, res).await
    }
}

#[scope("/")]
mod pages {
    use super::*;
    use firework::get;

    #[get("/")]
    async fn index(req: Request, res: Response) -> Response {
        handlers::pages::index(req, res).await
    }

    #[get("/login")]
    async fn login(req: Request, res: Response) -> Response {
        handlers::pages::login_page(req, res).await
    }

    #[get("/register")]
    async fn register(req: Request, res: Response) -> Response {
        handlers::pages::register_page(req, res).await
    }
}

#[tokio::main]
async fn main() {
    // Register plugin
    let plugin = Arc::new(firework_seaorm::SeaOrmPlugin::from_config().await);
    firework::register_plugin(plugin);

    let server = routes!().middleware(cors_middleware);

    println!("Server running on http://127.0.0.1:8080");
    server.listen_with_config().await.unwrap();
}
