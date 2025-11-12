use firework::hot_reload::HotReload;

#[tokio::main]
async fn main() {
    println!("Starting server with hot reload...");
    
    HotReload::new()
        .watch_path("src")
        .watch_path("examples")
        .ignore_pattern("4913")
        .ignore_pattern("*.swp")
        .ignore_pattern("*~")
        .start()
        .await
        .expect("Hot reload failed");
}
