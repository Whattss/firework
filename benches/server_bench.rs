use criterion::{criterion_group, criterion_main, Criterion};
use firework::prelude::*;
use std::time::Duration;
use tokio::runtime::Runtime;

async fn start_test_server() -> u16 {
    use firework::Server;
    
    // Find available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    
    let server = Server::new()
        .get("/", |_req, res| async move { res })
        .get("/hello", |_req, mut res| async move {
            res.set_body(b"Hello, World!".to_vec());
            res
        })
        .get("/json", |_req, mut res| async move {
            let body = serde_json::json!({
                "message": "Hello",
                "status": "ok"
            });
            res.set_body(serde_json::to_vec(&body).unwrap());
            res.headers.insert("Content-Type".to_string(), "application/json".to_string());
            res
        });
    
    tokio::spawn(async move {
        server.listen(&format!("127.0.0.1:{}", port)).await.ok();
    });
    
    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    port
}

fn bench_server_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let port = rt.block_on(start_test_server());
    let client = reqwest::blocking::Client::new();
    let url = format!("http://127.0.0.1:{}", port);
    
    c.bench_function("server_simple_get", |b| {
        b.iter(|| {
            client.get(&url).send().unwrap()
        })
    });
    
    c.bench_function("server_hello_world", |b| {
        b.iter(|| {
            client.get(&format!("{}/hello", url)).send().unwrap()
        })
    });
    
    c.bench_function("server_json_response", |b| {
        b.iter(|| {
            client.get(&format!("{}/json", url)).send().unwrap()
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets = bench_server_throughput
}
criterion_main!(benches);
