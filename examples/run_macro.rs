use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello, Firework! 🔥"
}

#[get("/users/:id")]
async fn get_user(Path(id): Path<u32>) -> String {
    format!("User ID: {}", id)
}

#[post("/users")]
async fn create_user(Json(data): Json<serde_json::Value>) -> Response {
    Response::new(StatusCode::Ok, vec![])
        .json(serde_json::json!({
            "message": "User created",
            "data": data
        }))
}

#[middleware]
async fn logger(req: &mut Request, _res: &mut Response) -> Flow {
    println!("[{:?}] {}", req.method, req.uri.path);
    Flow::Continue
}

fn main() {
    // That's it! Auto-loads config, routes, middleware, plugins
    run!();
}
