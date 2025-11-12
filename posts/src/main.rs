use firework::prelude::*;

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
