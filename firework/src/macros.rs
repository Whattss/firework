/// Helper macros para crear respuestas

#[macro_export]
macro_rules! text {
    ($body:expr) => {{
        let mut response = $crate::Response::new($crate::StatusCode::Ok, $body.as_bytes());
        response.headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        response
    }};
    ($status:expr, $body:expr) => {{
        let mut response = $crate::Response::new($status, $body.as_bytes());
        response.headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        response
    }};
}

#[macro_export]
macro_rules! html {
    ($body:expr) => {{
        let mut response = $crate::Response::new($crate::StatusCode::Ok, $body.as_bytes());
        response.headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        response
    }};
    ($status:expr, $body:expr) => {{
        let mut response = $crate::Response::new($status, $body.as_bytes());
        response.headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        response
    }};
}

#[macro_export]
macro_rules! json {
    ($data:expr) => {{
        match serde_json::to_vec(&$data) {
            Ok(body) => {
                let mut response = $crate::Response::new($crate::StatusCode::Ok, body);
                response.headers.insert("Content-Type".to_string(), "application/json".to_string());
                response
            }
            Err(_) => {
                let body = b"{\"error\":\"Failed to serialize JSON\"}";
                let mut response = $crate::Response::new($crate::StatusCode::InternalServerError, body);
                response.headers.insert("Content-Type".to_string(), "application/json".to_string());
                response
            }
        }
    }};
    ($status:expr, $data:expr) => {{
        match serde_json::to_vec(&$data) {
            Ok(body) => {
                let mut response = $crate::Response::new($status, body);
                response.headers.insert("Content-Type".to_string(), "application/json".to_string());
                response
            }
            Err(_) => {
                let body = b"{\"error\":\"Failed to serialize JSON\"}";
                let mut response = $crate::Response::new($crate::StatusCode::InternalServerError, body);
                response.headers.insert("Content-Type".to_string(), "application/json".to_string());
                response
            }
        }
    }};
}

#[macro_export]
macro_rules! redirect {
    ($location:expr) => {{
        let mut response = $crate::Response::new($crate::StatusCode::Found, b"");
        response.headers.insert("Location".to_string(), $location.to_string());
        response
    }};
    ($status:expr, $location:expr) => {{
        let mut response = $crate::Response::new($status, b"");
        response.headers.insert("Location".to_string(), $location.to_string());
        response
    }};
}

#[macro_export]
macro_rules! stream {
    ($reader:expr) => {{
        $crate::Response::stream($crate::StatusCode::Ok, $reader)
    }};
    ($status:expr, $reader:expr) => {{
        $crate::Response::stream($status, $reader)
    }};
    ($status:expr, $reader:expr, $content_type:expr) => {{
        let mut response = $crate::Response::stream($status, $reader);
        response.headers.insert("Content-Type".to_string(), $content_type.to_string());
        response
    }};
}

#[macro_export]
macro_rules! serve {
    ($path:expr) => {{
        $crate::serve_file($path).await
    }};
    ($dir:expr, $fallback:expr) => {{
        $crate::serve_dir($dir, $fallback).await
    }};
}
