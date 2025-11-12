use firework::{serve_file, serve_static, Server, Request, Response, routes};

// Serve homepage
#[firework::get("/")]
async fn index(_req: Request, _res: Response) -> Response {
    firework::html!(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Firework Static File Server</title>
            <link rel="stylesheet" href="/static/style.css">
        </head>
        <body>
            <h1>Firework Static File Server</h1>
            <p>Serving static files with style!</p>
            
            <h2>Features:</h2>
            <ul>
                <li>Automatic content type detection</li>
                <li>Security (directory traversal protection)</li>
                <li>Streaming support for large files</li>
                <li>SPA fallback support</li>
            </ul>
            
            <h2>Examples:</h2>
            <ul>
                <li><a href="/static/test.html">HTML File</a></li>
                <li><a href="/static/style.css">CSS File</a></li>
                <li><a href="/static/data.json">JSON File</a></li>
                <li><a href="/download">Download File</a></li>
            </ul>
            
            <h2>API:</h2>
            <ul>
                <li><a href="/api/health">Health Check</a></li>
                <li><a href="/api/stats">Statistics</a></li>
            </ul>
            
            <script src="/static/app.js"></script>
        </body>
        </html>
    "#)
}

// API endpoints
#[firework::get("/api/health")]
async fn health(_req: Request, _res: Response) -> Response {
    use serde_json::json;
    firework::json!(json!({
        "status": "ok",
        "service": "firework",
        "version": "0.1.0"
    }))
}

#[firework::get("/api/stats")]
async fn stats(_req: Request, _res: Response) -> Response {
    use serde_json::json;
    firework::json!(json!({
        "files_served": 1337,
        "uptime_seconds": 12345,
        "requests_per_second": 42.5
    }))
}

// Serve static files from /static directory
#[firework::get("/static/*")]
async fn serve_static_files(req: Request, _res: Response) -> Response {
    let path = req.uri.path.strip_prefix("/static/").unwrap_or("");
    serve_static("./public", path).await
}

// Serve a specific download file
#[firework::get("/download")]
async fn download(_req: Request, _res: Response) -> Response {
    let mut response = serve_file("./public/download.zip").await;
    response.headers.insert(
        "Content-Disposition".to_string(),
        "attachment; filename=\"download.zip\"".to_string()
    );
    response
}

// 404 handler
#[firework::get("/*")]
async fn not_found(_req: Request, _res: Response) -> Response {
    firework::html!(firework::StatusCode::NotFound, r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>404 Not Found</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    text-align: center;
                    padding: 50px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                }
                h1 { font-size: 72px; margin: 0; }
                p { font-size: 24px; }
                a { color: #fff; text-decoration: underline; }
            </style>
        </head>
        <body>
            <h1>404</h1>
            <p>Page not found</p>
            <p><a href="/">Go back home</a></p>
        </body>
        </html>
    "#)
}

#[tokio::main]
async fn main() {
    println!("=== Firework Static File Server ===");
    println!();
    println!("Starting server on http://127.0.0.1:8080");
    println!();
    println!("Available routes:");
    println!("  GET  /                - Homepage");
    println!("  GET  /api/health      - Health check");
    println!("  GET  /api/stats       - Statistics");
    println!("  GET  /static/*        - Static files");
    println!("  GET  /download        - Download example");
    println!();
    println!("Public files directory: ./public/");
    println!();
    println!("Create some test files:");
    println!("  mkdir -p public");
    println!("  echo '<h1>Test</h1>' > public/test.html");
    println!("  echo 'body {{ margin: 0; }}' > public/style.css");
    println!("  echo 'console.log(\"Hello\");' > public/app.js");
    println!(r#"  echo '{{"test": true}}' > public/data.json"#);
    println!();
    
    let server = Server::new();
    
    routes!(server, 
        index,
        health,
        stats,
        serve_static_files,
        download,
        not_found
    );
    
    server.listen("127.0.0.1:8080").await.unwrap();
}
