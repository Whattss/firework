// src/server.rs
use crate::router::RadixNode;
use crate::request::Request;
use crate::response::Response;
use crate::error::ServerError;
use crate::route::{Route, Method};
use crate::middleware::MiddlewareCloneWrapper;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;
use log::error;

/// Trait para definir un manejador de ruta.
#[async_trait]
pub trait RouteHandler: Fn(Request, &mut Response) + Send + Sync + 'static {}
impl<T: Fn(Request, &mut Response) + Send + Sync + 'static> RouteHandler for T {}

/// Estructura principal del servidor.
#[derive(Clone)]
pub struct Server {
    pub routemap: RadixNode,
    pub middleware: Vec<MiddlewareCloneWrapper>,
    pub port: u16,
    pub direction: [u8; 4],
}

impl Server {
    /// Crea un nuevo servidor con puerto por defecto 8080.
    pub fn new() -> Self {
        Server {
            routemap: RadixNode::new(""),
            middleware: Vec::new(),
            direction: [127, 0, 0 ,1],
            port: 8080,
        }
    }

    /// Agrega una ruta y su manejador correspondiente.
    pub fn add_route<F>(&mut self, route: Route, handler: F)
    where
        F: RouteHandler + Clone + 'static,
    {
        self.routemap.insert(
            &route.path,
            route.method.clone(),
            Arc::new(handler),
        );
    }

    /// Agrega un middleware que se ejecutará antes de los manejadores de ruta.
    pub fn add_middleware<F>(&mut self, middleware: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
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

        // Ejecuta cada middleware
        for mw in &self.middleware {
            (mw.0)(&mut request, &mut response);
        }

        let mut ignore_handler = false; 

        if let Some(k) = request.extra.get("ignore_handler") {
            if k.to_lowercase() == "true".to_owned() {
                ignore_handler = true;
            }
        } 

        if !ignore_handler {
            let route_method = Method::from_str(method).map_err(|_| {
                error!("Invalid HTTP method: {}", method);
                ServerError::BadRequest("Invalid HTTP method.".to_string())
            })?;
            let mut found_route = false;

            if let Some((route, handler, params)) = self.routemap.find(path) {
                if route.method == route_method {
                    found_route = true;
                    let filtered_params: HashMap<String, String> = params
                    .iter()
                    .filter(|(_, v)| !v.is_empty()) 
                    .map(|(k, v)| (k.clone(), v.clone())) 
                    .collect();
                    let mut req_with_params = request.clone();
                    req_with_params.params = filtered_params;
                    handler(req_with_params, &mut response);
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

    /// Inicia el servidor escuchando en el puerto configurado.
    pub async fn listen(self) {
        let listener = TcpListener::bind(format!("{}.{}.{}.{}:{}", self.direction[0], self.direction[1], self.direction[2], self.direction[3], self.port))
            .await
            .expect("Failed trying to bind the socket.");
        println!("Server listening on port {}", self.port);

        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_request(&mut stream).await {
                            let mut response = Response::new();
                            response.error_response(e);
                            let _ = &stream.write_all(response.to_string().as_bytes()).await;
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Permite configurar un puerto distinto.
    pub fn set_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn set_direction(mut self, direction: [u8; 4]) -> Self {
        self.direction = direction;
        self
    }
}
