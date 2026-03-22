use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello!"
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
        .block_on(async {
            let _ = firework::init_config("Firework.toml").await;
            let config = firework::get_config().await;
            
            let mut server = firework::Server::new();
            
            for route in firework::ROUTES {
                server = match route.method {
                    "GET" => server.get(route.path, route.handler),
                    _ => server.route(route.method, route.path, route.handler),
                };
            }
            
            let address = format!("{}:{}", config.server.address, config.server.port);
            
            println!("Starting on http://{}", address);
            
            server.listen(&address)
                .await
                .expect("Failed to start server");
        });
}
