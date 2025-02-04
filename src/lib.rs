pub mod route;

use route::{Route, Method};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;
use log::{error, info, warn}; // Log para registrar errores

// Definición del tipo de error centralizado
#[derive(Debug)]
pub enum ServerError {
    InvalidRequest,
    NotFound,
    InternalServerError(String), // Puede contener detalles adicionales
    BadRequest(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ServerError::InvalidRequest => write!(f, "Invalid request"),
            ServerError::NotFound => write!(f, "Not found"),
            ServerError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ServerError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::InternalServerError(err.to_string())
    }
}

// Trait para manejar rutas
pub trait RouteHandler: Fn(Request, &mut Response) + Send + Sync + 'static {}
impl<T: Fn(Request, &mut Response) + Send + Sync + 'static> RouteHandler for T {}

#[derive(Clone)]
struct RouteHandlerCloneWrapper(Arc<dyn RouteHandler>);

impl RouteHandlerCloneWrapper {
    fn new<F>(f: F) -> Self
    where
        F: RouteHandler + Clone + 'static,
    {
        RouteHandlerCloneWrapper(Arc::new(f))
    }
}

// Trait para manejar middleware
pub trait MiddlewareHandler: Fn(&mut Request, &mut Response) + Send + Sync + 'static {}
impl<T: Fn(&mut Request, &mut Response) + Send + Sync + 'static> MiddlewareHandler for T {}

#[derive(Clone)]
struct MiddlewareCloneWrapper(Arc<dyn MiddlewareHandler>);

impl MiddlewareCloneWrapper {
    fn new<F>(f: F) -> Self
    where
        F: MiddlewareHandler + Clone + 'static,
    {
        MiddlewareCloneWrapper(Arc::new(f))
    }
}

// Servidor
#[derive(Clone)]
pub struct Server {
    routemap: HashMap<Route, RouteHandlerCloneWrapper>,
    middleware: Vec<MiddlewareCloneWrapper>,
    port: u16,
}

impl Server {
    pub fn new() -> Self {
        Server {
            routemap: HashMap::new(),
            middleware: Vec::new(),
            port: 8080,
        }
    }

    pub fn add_route<F>(&mut self, route: Route, handler: F)
    where
        F: RouteHandler + Clone + 'static,
    {
        self.routemap.insert(route, RouteHandlerCloneWrapper::new(handler));
    }

    pub fn add_middleware<F>(&mut self, middleware: F)
    where
        F: MiddlewareHandler + Clone + 'static,
    {
        self.middleware.push(MiddlewareCloneWrapper::new(middleware));
    }

    async fn handle_request(&self, stream: &mut tokio::net::TcpStream) -> Result<(), ServerError> {
        let mut buffer = [0; 4096];
        let bytes_read = stream.read(&mut buffer).await.map_err(|e| {
            error!("Failed to read from stream: {}", e);
            ServerError::InternalServerError(e.to_string())
        })?;

        let request_str = std::str::from_utf8(&buffer[..bytes_read]).map_err(|e| {
            error!("Failed to parse request: {}", e);
            ServerError::BadRequest(e.to_string())
        })?;
        
        let parts: Vec<&str> = request_str.split("\r\n\r\n").collect();
        let headers_part = parts[0];
        let body = if parts.len() > 1 { parts[1] } else { "" };
        
        let mut headers = HashMap::new();
        for line in headers_part.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_string(), value.to_string());
            }
        }
        
        let request_line: Vec<&str> = headers_part.lines().next().unwrap_or("").split_whitespace().collect();
        if request_line.len() < 3 {
            return Err(ServerError::BadRequest("Invalid request line.".to_string()));
        }
        
        let method = request_line[0];
        let path = request_line[1];
        
        let mut request = Request::new(path.to_string(), headers.clone(), body.to_string());
        request.headers.insert("Method".to_string(), method.to_string());
        let mut response = Response::new();
        
        for mw in &self.middleware {
            mw.0(&mut request, &mut response);
        }
        
        if response.body.is_empty() {
            let route_method = Method::from_str(method).map_err(|e| {
                error!("Invalid HTTP method: {:?}", e);
                ServerError::BadRequest("Invalid HTTP method.".to_string())
            })?;
            let mut found_route = false;
    
            for (route, handler) in &self.routemap {
                if route.method == route_method {
                    if let Some(params) = route.matches(path) {
                        found_route = true;
    
                        let mut req_with_params = request.clone();
                        req_with_params.params = params;
    
                        handler.0(req_with_params, &mut response);
                        break;
                    }
                }
            }
        
            if !found_route {
                response.status_code = 404;
                response.body = "404 Not Found: The requested route was not found.".to_string();
            }
        }
        
        stream.write_all(response.to_string().as_bytes()).await.map_err(|e| {
            error!("Failed to send response: {}", e);
            ServerError::InternalServerError(e.to_string())
        })?;
        stream.flush().await.map_err(|e| {
            error!("Failed to flush stream: {}", e);
            ServerError::InternalServerError(e.to_string())
        })?;

        Ok(())
    }

    pub async fn listen(self) {
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .await
            .unwrap();

        println!("Server listening on port {}", self.port);

        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let server = self.clone();

            tokio::spawn(async move {
                if let Err(e) = server.handle_request(&mut stream).await {
                    let mut response = Response::new();
                    response.error_response(e);
                    let _ = stream.write_all(response.to_string().as_bytes()).await;
                }
            });
        }
    }

    pub fn set_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub params: HashMap<String, String>
}

impl Request {
    pub fn new(path: String, headers: HashMap<String, String>, body: String) -> Self {
        Request {
            path,
            headers,
            body,
            params: HashMap::new()
        }
    }
}

pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status_code: 200,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn set_content_type(&mut self, content_type: &str) {
        self.headers.insert("Content-Type".to_string(), content_type.to_string());
    }

    pub fn error_response(&mut self, error: ServerError) {
        match error {
            ServerError::NotFound => {
                self.status_code = 404;
                self.body = "404 Not Found: The requested resource could not be found.".to_string();
            }
            ServerError::InvalidRequest => {
                self.status_code = 400;
                self.body = "400 Bad Request: Invalid request format.".to_string();
            }
            ServerError::InternalServerError(msg) => {
                self.status_code = 500;
                self.body = format!("500 Internal Server Error: {}", msg);
            }
            ServerError::BadRequest(msg) => {
                self.status_code = 400;
                self.body = format!("400 Bad Request: {}", msg);
            }
        }
        self.headers.insert("Content-Type".to_string(), "application/json; charset=utf-8".to_string());
    }

    pub fn to_string(&self) -> String {
        let mut response = format!("HTTP/1.1 {} OK\r\n", self.status_code);
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");
        response.push_str(&self.body);
        response
    }
}

#[macro_export]
macro_rules! get {
    ($server:expr, $path:expr, $handler:expr) => {
        $server.add_route(
            Route::new(Method::GET, $path.to_string()),
            $handler
        );
    };
}

#[macro_export]
macro_rules! post {
    ($server:expr, $path:expr, $handler:expr) => {
        $server.add_route(
            Route::new(Method::POST, $path.to_string()),
            $handler
        );
    };
}

#[macro_export]
macro_rules! put {
    ($server:expr, $path:expr, $handler:expr) => {
        $server.add_route(
            Route::new(Method::PUT, $path.to_string()),
            $handler
        );
    };
}

#[macro_export]
macro_rules! delete {
    ($server:expr, $path:expr, $handler:expr) => {
        $server.add_route(
            Route::new(Method::DELETE, $path.to_string()),
            $handler
        );
    };
}

#[macro_export]
macro_rules! response {
    ($response:expr, $type:expr, $body:expr) => {
        $response.set_content_type(format!("{}; charset=utf-8", type));
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
