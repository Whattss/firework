use firework::{serve_file, serve_static, serve_dir, Response, StatusCode};

#[tokio::test]
async fn test_serve_file() {
    // Create test file
    tokio::fs::create_dir_all("test_public").await.unwrap();
    tokio::fs::write("test_public/test.txt", b"Hello World").await.unwrap();
    
    let response = serve_file("test_public/test.txt").await;
    
    assert_eq!(response.status, StatusCode::Ok);
    assert_eq!(response.headers.get("Content-Type").unwrap(), "text/plain; charset=utf-8");
    
    // Cleanup
    tokio::fs::remove_dir_all("test_public").await.ok();
}

#[tokio::test]
async fn test_serve_file_not_found() {
    let response = serve_file("nonexistent.txt").await;
    assert_eq!(response.status, StatusCode::NotFound);
}

#[tokio::test]
async fn test_serve_static_security() {
    tokio::fs::create_dir_all("test_public").await.unwrap();
    tokio::fs::write("test_public/secret.txt", b"secret").await.unwrap();
    
    // Try directory traversal
    let response = serve_static("test_public", "../../../etc/passwd").await;
    assert_eq!(response.status, StatusCode::Forbidden);
    
    // Cleanup
    tokio::fs::remove_dir_all("test_public").await.ok();
}

#[tokio::test]
async fn test_content_type_detection() {
    tokio::fs::create_dir_all("test_public").await.unwrap();
    
    // Test HTML
    tokio::fs::write("test_public/test.html", b"<html></html>").await.unwrap();
    let response = serve_file("test_public/test.html").await;
    assert_eq!(response.headers.get("Content-Type").unwrap(), "text/html; charset=utf-8");
    
    // Test CSS
    tokio::fs::write("test_public/test.css", b"body{}").await.unwrap();
    let response = serve_file("test_public/test.css").await;
    assert_eq!(response.headers.get("Content-Type").unwrap(), "text/css; charset=utf-8");
    
    // Test JS
    tokio::fs::write("test_public/test.js", b"console.log()").await.unwrap();
    let response = serve_file("test_public/test.js").await;
    assert_eq!(response.headers.get("Content-Type").unwrap(), "application/javascript; charset=utf-8");
    
    // Test JSON
    tokio::fs::write("test_public/test.json", b"{}").await.unwrap();
    let response = serve_file("test_public/test.json").await;
    assert_eq!(response.headers.get("Content-Type").unwrap(), "application/json");
    
    // Cleanup
    tokio::fs::remove_dir_all("test_public").await.ok();
}

#[tokio::test]
async fn test_serve_dir_with_index() {
    tokio::fs::create_dir_all("test_public").await.unwrap();
    tokio::fs::write("test_public/index.html", b"<html>Index</html>").await.unwrap();
    
    let response = serve_dir("test_public", None).await;
    assert_eq!(response.status, StatusCode::Ok);
    assert_eq!(response.headers.get("Content-Type").unwrap(), "text/html; charset=utf-8");
    
    // Cleanup
    tokio::fs::remove_dir_all("test_public").await.ok();
}

#[tokio::test]
async fn test_serve_dir_with_fallback() {
    tokio::fs::create_dir_all("test_public").await.unwrap();
    tokio::fs::write("test_public/fallback.html", b"<html>Fallback</html>").await.unwrap();
    
    let response = serve_dir("test_public", Some("fallback.html")).await;
    assert_eq!(response.status, StatusCode::Ok);
    
    // Cleanup
    tokio::fs::remove_dir_all("test_public").await.ok();
}
