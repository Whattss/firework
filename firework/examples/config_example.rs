use firework::*;

async fn index_handler(_req: Request, _res: Response) -> Response {
    text!("Config System Example\n\nEndpoints:\n- GET /config - View configuration")
}

async fn config_handler(_req: Request, _res: Response) -> Response {
    let config = get_config().await;
    json!(&config)
}

#[tokio::main]
async fn main() {
    println!("=== Firework Config System Example ===\n");
    
    // Configuration is automatically loaded from Firework.toml or firework.toml
    // You can also manually load a specific config file:
    // init_config("path/to/config.toml").await.unwrap();
    
    // Get the configuration
    let config = get_config().await;
    println!("Server Configuration:");
    println!("  Address: {}", config.server.address);
    println!("  Port: {}", config.server.port);
    println!("  Workers: {}", config.server.workers);
    println!();
    
    // Display all plugin configurations
    if !config.plugins.is_empty() {
        println!("Plugin Configurations:");
        for (plugin_name, plugin_config) in &config.plugins {
            println!("  {}: {:?}", plugin_name, plugin_config);
        }
        println!();
    }
    
    // Create server
    let server = Server::new()
        .get("/", index_handler)
        .get("/config", config_handler);
    
    // Start server using configuration from Firework.toml
    println!("Starting server on {}...\n", config.bind_address());
    server.listen_with_config().await.unwrap();
}


