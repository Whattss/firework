// src/macros.rs
#[macro_export]
macro_rules! get {
    ($server:expr, $path:expr, $handler:expr) => {
        $server.add_route(
            crate::route::Route::new(crate::route::Method::GET, $path.to_string()),
            $handler
        );
    };
}

#[macro_export]
macro_rules! post {
    ($server:expr, $path:expr, $handler:expr) => {
        $server.add_route(
            crate::route::Route::new(crate::route::Method::POST, $path.to_string()),
            $handler
        );
    };
}

#[macro_export]
macro_rules! put {
    ($server:expr, $path:expr, $handler:expr) => {
        $server.add_route(
            crate::route::Route::new(crate::route::Method::PUT, $path.to_string()),
            $handler
        );
    };
}

#[macro_export]
macro_rules! delete {
    ($server:expr, $path:expr, $handler:expr) => {
        $server.add_route(
            crate::route::Route::new(crate::route::Method::DELETE, $path.to_string()),
            $handler
        );
    };
}

#[macro_export]
macro_rules! response {
    ($response:expr, $type:expr, $body:expr) => {
        $response.set_content_type(&format!("{}; charset=utf-8", $type));
        $response.body = $body.to_string();
    };
}

#[macro_export]
macro_rules! text {
    ($response:expr, $body:expr) => {
        $response.set_content_type("text/plain; charset=utf-8");
        $response.body = $body.to_string();
    };
}

#[macro_export]
macro_rules! html {
    ($response:expr, $body:expr) => {
        $response.set_content_type("text/html; charset=utf-8");
        $response.body = $body.to_string();
    };
}

#[macro_export]
macro_rules! json {
    ($response:expr, $body:expr) => {
        $response.set_content_type("application/json; charset=utf-8");
        $response.body = $body.to_string();
    };
}
