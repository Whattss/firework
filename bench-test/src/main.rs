use firework::prelude::*;

#[get("/")]
async fn index(_req: Request, _res: Response) -> Response {
    Response::new(StatusCode::Ok, b"Hello, World!")
}

#[get("/json")]
async fn json(_req: Request, res: Response) -> Response {
    res.json(serde_json::json!({"message": "Hello, Firework!"}))
}

#[tokio::main]
async fn main() {
    let server = routes!();
    
    println!("🔥 Firework Benchmark Server");
    println!("   http://127.0.0.1:8080");
    println!("");
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
