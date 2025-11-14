// Integration tests for Firework framework

use firework::prelude::*;
use std::time::Duration;

#[tokio::test]
async fn test_error_types() {
    // Test all error types return correct status codes
    let errors = vec![
        (Error::BadRequest("test".into()), 400),
        (Error::Unauthorized("test".into()), 401),
        (Error::Forbidden("test".into()), 403),
        (Error::NotFound("test".into()), 404),
        (Error::Conflict("test".into()), 409),
        (Error::Gone("test".into()), 410),
        (Error::PayloadTooLarge("test".into()), 413),
        (Error::UriTooLong("test".into()), 414),
        (Error::UnprocessableEntity("test".into()), 422),
        (Error::TooManyRequests("test".into()), 429),
        (Error::InternalServerError("test".into()), 500),
        (Error::ServiceUnavailable("test".into()), 503),
        (Error::GatewayTimeout("test".into()), 504),
    ];

    for (error, expected_code) in errors {
        let response = error.into_response();
        assert_eq!(response.status.code(), expected_code);
    }
}

#[tokio::test]
async fn test_concurrent_requests() {
    use std::sync::Arc;
    use tokio::sync::Barrier;
    
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];
    
    for i in 0..10 {
        let barrier = Arc::clone(&barrier);
        let handle = tokio::spawn(async move {
            barrier.wait().await;
            // Simulate concurrent request handling
            let req = Request::new(
                Method::GET,
                Uri::new(&format!("/test/{}", i), None),
                Version::Http11,
                std::collections::HashMap::new(),
                Vec::new(),
                None,
            );
            req
        });
        handles.push(handle);
    }
    
    let results = futures_util::future::join_all(handles).await;
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_large_headers() {
    let mut headers = std::collections::HashMap::new();
    
    // Add many headers
    for i in 0..100 {
        headers.insert(
            format!("X-Custom-Header-{}", i),
            vec![format!("value-{}", i)],
        );
    }
    
    let req = Request::new(
        Method::GET,
        Uri::new("/test", None),
        Version::Http11,
        headers.clone(),
        Vec::new(),
        None,
    );
    
    assert_eq!(req.headers.len(), 100);
}

#[tokio::test]
async fn test_large_body() {
    // Test with 1MB body
    let body = vec![0u8; 1024 * 1024];
    
    let req = Request::new(
        Method::POST,
        Uri::new("/upload", None),
        Version::Http11,
        std::collections::HashMap::new(),
        body.clone(),
        None,
    );
    
    assert_eq!(req.body.len(), 1024 * 1024);
}

#[tokio::test]
async fn test_router_performance() {
    use firework::Router;
    use std::future::Future;
    use std::pin::Pin;
    
    fn handler(_req: Request, res: Response) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(async move { res })
    }
    
    let mut router = Router::new();
    
    // Add 1000 routes
    for i in 0..1000 {
        router.add_route("GET", &format!("/route{}", i), Box::new(handler));
    }
    
    // Lookup should still be fast
    let start = std::time::Instant::now();
    let _ = router.find(&Method::GET, "/route500");
    let elapsed = start.elapsed();
    
    assert!(elapsed < Duration::from_micros(100)); // Should be sub-100µs
}

#[tokio::test]
async fn test_router_param_extraction() {
    use firework::Router;
    use std::future::Future;
    use std::pin::Pin;
    
    fn handler(_req: Request, res: Response) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(async move { res })
    }
    
    let mut router = Router::new();
    router.add_route("GET", "/users/:id/posts/:post_id", Box::new(handler));
    
    let result = router.find(&Method::GET, "/users/123/posts/456");
    assert!(result.is_some());
    
    let (_, params) = result.unwrap();
    assert_eq!(params.get("id"), Some(&"123".to_string()));
    assert_eq!(params.get("post_id"), Some(&"456".to_string()));
}

#[tokio::test]
async fn test_request_timeout_simulation() {
    // Simulate timeout handling
    let timeout = Duration::from_millis(100);
    
    let slow_operation = async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        "done"
    };
    
    let result = tokio::time::timeout(timeout, slow_operation).await;
    assert!(result.is_err()); // Should timeout
}

#[tokio::test]
async fn test_json_parsing() {
    use serde_json::json;
    
    let data = json!({
        "name": "test",
        "age": 30,
        "active": true
    });
    
    let body = serde_json::to_vec(&data).unwrap();
    
    let req = Request::new(
        Method::POST,
        Uri::new("/api/users", None),
        Version::Http11,
        std::collections::HashMap::new(),
        body,
        None,
    );
    
    let parsed: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
    assert_eq!(parsed["name"], "test");
    assert_eq!(parsed["age"], 30);
}

#[tokio::test]
async fn test_query_params() {
    let mut query = std::collections::HashMap::new();
    query.insert("page".to_string(), "1".to_string());
    query.insert("limit".to_string(), "10".to_string());
    query.insert("sort".to_string(), "desc".to_string());
    
    let uri = Uri::new("/api/users", Some(query.clone()));
    
    assert_eq!(uri.path, "/api/users");
    assert!(uri.query.is_some());
    assert_eq!(uri.query.as_ref().unwrap().get("page"), Some(&"1".to_string()));
}

#[tokio::test]
async fn test_response_builder() {
    let mut response = Response::new(StatusCode::Ok, b"test".to_vec());
    response.headers.insert("Content-Type".to_string(), "text/plain".to_string());
    response.headers.insert("X-Custom".to_string(), "value".to_string());
    
    assert_eq!(response.status.code(), 200);
    assert_eq!(response.headers.len(), 2);
}

#[tokio::test]
async fn test_method_parsing() {
    let methods = vec![
        (Method::GET, "GET"),
        (Method::POST, "POST"),
        (Method::PUT, "PUT"),
        (Method::DELETE, "DELETE"),
        (Method::PATCH, "PATCH"),
        (Method::HEAD, "HEAD"),
        (Method::OPTIONS, "OPTIONS"),
    ];
    
    for (method, _name) in methods {
        // Just ensure they exist and can be created
        let _req = Request::new(
            method,
            Uri::new("/test", None),
            Version::Http11,
            std::collections::HashMap::new(),
            Vec::new(),
            None,
        );
    }
}

#[tokio::test]
async fn test_edge_case_empty_path() {
    let uri = Uri::new("", None);
    assert_eq!(uri.path, "");
}

#[tokio::test]
async fn test_edge_case_long_path() {
    let long_path = "/".to_string() + &"a/".repeat(100);
    let uri = Uri::new(&long_path, None);
    assert!(uri.path.len() > 200);
}

#[tokio::test]
async fn test_concurrent_router_access() {
    use firework::Router;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    
    fn handler(_req: Request, res: Response) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(async move { res })
    }
    
    let mut router = Router::new();
    router.add_route("GET", "/test", Box::new(handler));
    let router = Arc::new(router);
    
    let mut handles = vec![];
    for _ in 0..100 {
        let router = Arc::clone(&router);
        let handle = tokio::spawn(async move {
            router.find(&Method::GET, "/test")
        });
        handles.push(handle);
    }
    
    let results = futures_util::future::join_all(handles).await;
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
