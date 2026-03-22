use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use firework::Router;
use firework::{Method, Request, Response};
use std::future::Future;
use std::pin::Pin;

// Simple handler for benchmarking
fn simple_handler(
    _req: Request,
    res: Response,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    Box::pin(async move { res })
}

fn bench_router_insert(c: &mut Criterion) {
    c.bench_function("router_insert_simple", |b| {
        b.iter(|| {
            let mut router = Router::new();
            router.add_route("GET", black_box("/users"), Box::new(simple_handler));
        })
    });

    c.bench_function("router_insert_with_param", |b| {
        b.iter(|| {
            let mut router = Router::new();
            router.add_route("GET", black_box("/users/:id"), Box::new(simple_handler));
        })
    });

    c.bench_function("router_insert_complex", |b| {
        b.iter(|| {
            let mut router = Router::new();
            router.add_route(
                "GET",
                black_box("/api/v1/users/:id/posts/:post_id/comments"),
                Box::new(simple_handler),
            );
        })
    });
}

fn bench_router_lookup(c: &mut Criterion) {
    let mut router = Router::new();
    router.add_route("GET", "/users", Box::new(simple_handler));
    router.add_route("GET", "/users/:id", Box::new(simple_handler));
    router.add_route("GET", "/posts/:id", Box::new(simple_handler));
    router.add_route("GET", "/api/v1/users/:id/posts/:post_id", Box::new(simple_handler));
    
    c.bench_function("router_lookup_simple", |b| {
        b.iter(|| {
            router.find(black_box(&Method::GET), black_box("/users"))
        })
    });

    c.bench_function("router_lookup_with_param", |b| {
        b.iter(|| {
            router.find(black_box(&Method::GET), black_box("/users/123"))
        })
    });

    c.bench_function("router_lookup_complex", |b| {
        b.iter(|| {
            router.find(black_box(&Method::GET), black_box("/api/v1/users/42/posts/999"))
        })
    });

    c.bench_function("router_lookup_not_found", |b| {
        b.iter(|| {
            router.find(black_box(&Method::GET), black_box("/nonexistent/path"))
        })
    });
}

fn bench_router_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("router_scaling");
    
    for size in [10, 50, 100, 500].iter() {
        let mut router = Router::new();
        for i in 0..*size {
            router.add_route("GET", &format!("/route{}", i), Box::new(simple_handler));
        }
        
        group.bench_with_input(BenchmarkId::new("lookup", size), size, |b, _| {
            b.iter(|| {
                router.find(black_box(&Method::GET), black_box("/route42"))
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_router_insert,
    bench_router_lookup,
    bench_router_scaling
);
criterion_main!(benches);
