use crate::{Response, StatusCode};
use std::path::{Path, PathBuf};
use tokio::fs::File;

/// Serve a single file
pub async fn serve_file<P: AsRef<Path>>(path: P) -> Response {
    let path = path.as_ref();
    
    match File::open(path).await {
        Ok(file) => {
            let content_type = guess_content_type(path);
            let mut response = Response::stream(StatusCode::Ok, file);
            response.headers.insert("Content-Type".to_string(), content_type);
            response
        }
        Err(_) => {
            Response::new(StatusCode::NotFound, b"File not found")
        }
    }
}

/// Serve files from a directory with optional fallback
pub async fn serve_dir<P: AsRef<Path>>(dir: P, fallback: Option<&str>) -> Response {
    let dir = dir.as_ref();
    
    // Try to serve index.html if directory
    let index_path = dir.join("index.html");
    if index_path.exists() && index_path.is_file() {
        return serve_file(index_path).await;
    }
    
    // Try fallback if provided
    if let Some(fallback_file) = fallback {
        let fallback_path = dir.join(fallback_file);
        if fallback_path.exists() && fallback_path.is_file() {
            return serve_file(fallback_path).await;
        }
    }
    
    Response::new(StatusCode::NotFound, b"Not found")
}

/// Serve a static file based on request path
pub async fn serve_static<P: AsRef<Path>>(base_dir: P, request_path: &str) -> Response {
    let base_dir = base_dir.as_ref();
    
    // Remove leading slash and clean the path
    let request_path = request_path.trim_start_matches('/');
    
    // Prevent directory traversal attacks
    let safe_path = PathBuf::from(request_path);
    if safe_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Response::new(StatusCode::Forbidden, b"Access denied");
    }
    
    let file_path = base_dir.join(safe_path);
    
    // Check if path exists and is a file
    if !file_path.exists() {
        return Response::new(StatusCode::NotFound, b"File not found");
    }
    
    if file_path.is_dir() {
        // Try to serve index.html from directory
        let index_path = file_path.join("index.html");
        if index_path.exists() && index_path.is_file() {
            return serve_file(index_path).await;
        }
        return Response::new(StatusCode::Forbidden, b"Directory listing not allowed");
    }
    
    serve_file(file_path).await
}

/// Guess content type based on file extension
fn guess_content_type(path: &Path) -> String {
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    match extension {
        // Text
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        
        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" => "video/ogg",
        
        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        
        // Fonts
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        
        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        
        // Documents
        "pdf" => "application/pdf",
        
        // Default
        _ => "application/octet-stream",
    }.to_string()
}
