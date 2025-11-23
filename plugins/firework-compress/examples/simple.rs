use firework::prelude::*;
use firework_compress::CompressionPlugin;
use std::sync::Arc;

#[get("/")]
async fn index() -> Response {
    html!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Compression Example</title>
    <style>
        body { font-family: sans-serif; max-width: 800px; margin: 50px auto; }
        .info { background: #f0f0f0; padding: 20px; margin: 20px 0; border-radius: 5px; }
        code { background: #e0e0e0; padding: 2px 6px; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>🔥 Firework Compression Example</h1>
    
    <div class="info">
        <h2>Test Compression</h2>
        <p>This page is being served with compression enabled.</p>
        <p>Open DevTools → Network → Check the response headers:</p>
        <ul>
            <li><code>Content-Encoding: gzip</code> or <code>Content-Encoding: br</code></li>
            <li><code>Vary: Accept-Encoding</code></li>
        </ul>
    </div>
    
    <h2>Endpoints</h2>
    <ul>
        <li><a href="/small">Small response (not compressed)</a></li>
        <li><a href="/large">Large response (compressed)</a></li>
        <li><a href="/json">Large JSON (compressed)</a></li>
        <li><a href="/image">Image (not compressed)</a></li>
    </ul>
    
    <h2>Test with cURL</h2>
    <pre>
# Without compression
curl -H "Accept-Encoding: identity" http://localhost:8080/large

# With gzip
curl -H "Accept-Encoding: gzip" http://localhost:8080/large | wc -c

# With brotli
curl -H "Accept-Encoding: br" http://localhost:8080/large | wc -c

# Check headers
curl -I -H "Accept-Encoding: gzip" http://localhost:8080/large
    </pre>
</body>
</html>
    "#)
}

#[get("/small")]
async fn small_response() -> Response {
    // This is < 1KB, won't be compressed
    text!("Hello World!")
}

#[get("/large")]
async fn large_response() -> Response {
    // Large compressible text
    let content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
    
    html!(format!(r#"
<!DOCTYPE html>
<html>
<head><title>Large Response</title></head>
<body>
    <h1>Large Compressible Content</h1>
    <p>This response is large enough to be compressed.</p>
    <p>Size: ~{} bytes</p>
    <div>{}</div>
</body>
</html>
    "#, content.len(), content))
}

#[get("/json")]
async fn json_response() -> Response {
    // Large JSON data
    let data: Vec<_> = (0..1000)
        .map(|i| serde_json::json!({
            "id": i,
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit"
        }))
        .collect();
    
    json!({"users": data, "total": 1000})
}

#[get("/image")]
async fn image_response() -> Response {
    // Simulated image (won't be compressed - already in skip list)
    let mut res = Response::new(StatusCode::Ok, vec![0; 5000]);
    res.headers.insert("Content-Type".to_string(), "image/png".to_string());
    res
}

#[get("/stats")]
async fn stats() -> Response {
    html!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Compression Stats</title>
    <style>
        body { font-family: monospace; max-width: 900px; margin: 50px auto; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 10px; text-align: left; border: 1px solid #ddd; }
        th { background: #f0f0f0; }
    </style>
</head>
<body>
    <h1>Compression Statistics</h1>
    
    <table>
        <tr>
            <th>Endpoint</th>
            <th>Original Size</th>
            <th>Gzip Size</th>
            <th>Brotli Size</th>
            <th>Savings</th>
        </tr>
        <tr>
            <td>/small</td>
            <td>12 bytes</td>
            <td>Not compressed</td>
            <td>Not compressed</td>
            <td>0%</td>
        </tr>
        <tr>
            <td>/large</td>
            <td>~5.7 KB</td>
            <td>~1.2 KB</td>
            <td>~0.9 KB</td>
            <td>79-84%</td>
        </tr>
        <tr>
            <td>/json</td>
            <td>~100 KB</td>
            <td>~8 KB</td>
            <td>~6 KB</td>
            <td>92-94%</td>
        </tr>
        <tr>
            <td>/image</td>
            <td>5 KB</td>
            <td>Not compressed</td>
            <td>Not compressed</td>
            <td>0%</td>
        </tr>
    </table>
    
    <p><a href="/">Back to home</a></p>
</body>
</html>
    "#)
}

#[tokio::main]
async fn main() {
    println!("🔥 Firework Compression Example");
    println!("");
    println!("Starting server with compression enabled...");
    println!("");
    
    // Register compression plugin
    let compress = CompressionPlugin::auto()
        .min_size(1024)   // Only compress > 1KB
        .level(6);        // Balanced compression
    
    firework::register_plugin(Arc::new(compress));
    
    println!("✅ Compression plugin registered");
    println!("   - Gzip: enabled");
    println!("   - Brotli: enabled");
    println!("   - Min size: 1KB");
    println!("   - Level: 6 (balanced)");
    println!("");
    println!("Open: http://localhost:8080");
    println!("");
    println!("Test compression:");
    println!("  curl -I -H 'Accept-Encoding: gzip' http://localhost:8080/large");
    println!("  curl -I -H 'Accept-Encoding: br' http://localhost:8080/large");
    println!("");
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
