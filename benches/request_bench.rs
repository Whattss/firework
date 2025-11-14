use criterion::{black_box, criterion_group, criterion_main, Criterion};
use firework::{Method, Request, Response, Uri, Version};
use std::collections::HashMap;

fn create_simple_request() -> Request {
    Request::new(
        Method::GET,
        Uri::new("/test", None),
        Version::Http11,
        HashMap::new(),
        Vec::new(),
        None,
    )
}

fn create_request_with_headers() -> Request {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), vec!["application/json".to_string()]);
    headers.insert("Authorization".to_string(), vec!["Bearer token123".to_string()]);
    headers.insert("User-Agent".to_string(), vec!["Firework-Bench/1.0".to_string()]);
    
    Request::new(
        Method::POST,
        Uri::new("/api/users", Some(HashMap::from([
            ("page".to_string(), "1".to_string()),
            ("limit".to_string(), "10".to_string()),
        ]))),
        Version::Http11,
        headers,
        b"{\"name\":\"test\"}".to_vec(),
        None,
    )
}

fn bench_request_creation(c: &mut Criterion) {
    c.bench_function("request_create_simple", |b| {
        b.iter(|| create_simple_request())
    });

    c.bench_function("request_create_with_headers", |b| {
        b.iter(|| create_request_with_headers())
    });
}

fn bench_request_cloning(c: &mut Criterion) {
    let simple_req = create_simple_request();
    let complex_req = create_request_with_headers();

    c.bench_function("request_clone_simple", |b| {
        b.iter(|| black_box(simple_req.clone()))
    });

    c.bench_function("request_clone_complex", |b| {
        b.iter(|| black_box(complex_req.clone()))
    });
}

fn bench_response_creation(c: &mut Criterion) {
    c.bench_function("response_create_empty", |b| {
        b.iter(|| Response::default())
    });

    c.bench_function("response_create_with_body", |b| {
        b.iter(|| {
            Response::new(
                firework::StatusCode::Ok,
                b"Hello, World!".to_vec(),
            )
        })
    });

    c.bench_function("response_create_json", |b| {
        b.iter(|| {
            let body = serde_json::json!({
                "message": "Hello, World!",
                "status": "ok"
            });
            Response::new(
                firework::StatusCode::Ok,
                serde_json::to_vec(&body).unwrap(),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_request_creation,
    bench_request_cloning,
    bench_response_creation
);
criterion_main!(benches);
