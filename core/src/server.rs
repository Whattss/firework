use bytes::{BufMut, BytesMut};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::response::ResponseBody;
use crate::{
    AsyncHandler, AsyncMiddleware, Flow, Method, Middleware, Request, Response, Router, Uri,
    Version,
};

// Buffer pool para reutilizar buffers
use lazy_static::lazy_static;
use std::sync::Mutex;

const BUFFER_SIZE: usize = 8192;
const MAX_POOLED_BUFFERS: usize = 1024;

lazy_static! {
    static ref BUFFER_POOL: Mutex<Vec<BytesMut>> =
        Mutex::new(Vec::with_capacity(MAX_POOLED_BUFFERS));
}

fn get_buffer() -> BytesMut {
    BUFFER_POOL
        .lock()
        .unwrap()
        .pop()
        .unwrap_or_else(|| BytesMut::with_capacity(BUFFER_SIZE))
}

fn return_buffer(mut buf: BytesMut) {
    buf.clear();
    if let Ok(mut pool) = BUFFER_POOL.lock() {
        if pool.len() < MAX_POOLED_BUFFERS {
            pool.push(buf);
        }
    }
}

pub struct Server {
    pub(crate) router: Router,
    pub(crate) middlewares: Vec<Middleware>,
    pub(crate) async_middlewares: Vec<AsyncMiddleware>,
    prefix: String,
    ws_routes: std::collections::HashMap<String, Arc<dyn crate::websocket::WebSocketHandler>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            router: Router::new(),
            middlewares: Vec::new(),
            async_middlewares: Vec::new(),
            prefix: String::new(),
            ws_routes: std::collections::HashMap::new(),
        }
    }

    /// Establece un prefijo global para todas las rutas
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.trim_end_matches('/').to_string();
        self
    }

    pub fn route<H>(mut self, method: &str, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        let full_path = if self.prefix.is_empty() {
            path.to_string()
        } else {
            format!("{}{}", self.prefix, path)
        };
        self.router.add_route(method, &full_path, Box::new(handler));
        self
    }

    /// Crea un scope con un prefijo específico
    pub fn scope<F>(mut self, prefix: &str, configurator: F) -> Self
    where
        F: FnOnce(ServerScope) -> ServerScope,
    {
        let scope = ServerScope {
            prefix: prefix.trim_end_matches('/').to_string(),
            routes: Vec::new(),
        };

        let configured_scope = configurator(scope);

        for (method, path, handler) in configured_scope.routes {
            let full_path = format!("{}{}", configured_scope.prefix, path);
            self.router.add_route(&method, &full_path, handler);
        }

        self
    }

    pub fn get<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("GET", path, handler)
    }

    pub fn post<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("POST", path, handler)
    }

    pub fn put<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("PUT", path, handler)
    }

    pub fn patch<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("PATCH", path, handler)
    }

    pub fn delete<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("DELETE", path, handler)
    }

    pub fn middleware(mut self, mw: Middleware) -> Self {
        self.middlewares.push(mw);
        self
    }

    pub fn async_middleware(mut self, mw: AsyncMiddleware) -> Self {
        self.async_middlewares.push(mw);
        self
    }
    
    /// Register a WebSocket route
    pub fn websocket<H>(mut self, path: &str, handler: H) -> Self
    where
        H: crate::websocket::WebSocketHandler + 'static,
    {
        let full_path = if self.prefix.is_empty() {
            path.to_string()
        } else {
            format!("{}{}", self.prefix, path)
        };
        self.ws_routes.insert(full_path, Arc::new(handler));
        self
    }

    pub async fn listen(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let router = Arc::new(self.router);
        let middlewares = Arc::new(self.middlewares);
        let async_middlewares = Arc::new(self.async_middlewares);
        
        let ws_routes = Arc::new(self.ws_routes);

        // Load config if not already loaded
        let _ = crate::config::config();

        // Initialize plugins ONCE
        let plugin_registry = crate::plugin::registry();
        {
            let registry = plugin_registry.read().await;
            registry.init_all().await?;
            registry.start_all().await?;
        }

        // Create listener with SO_REUSEPORT and SO_REUSEADDR for better performance
        use socket2::{Domain, Protocol, Socket, Type};
        use std::net::SocketAddr;

        let socket_addr: SocketAddr = addr.parse()?;
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

        socket.set_reuse_address(true)?;
        #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
        {
            use std::os::fd::AsRawFd;
            unsafe {
                let optval: libc::c_int = 1;
                libc::setsockopt(
                    socket.as_raw_fd(),
                    libc::SOL_SOCKET,
                    libc::SO_REUSEPORT,
                    &optval as *const _ as *const libc::c_void,
                    std::mem::size_of_val(&optval) as libc::socklen_t,
                );
            }
        }

        socket.set_nonblocking(true)?;
        socket.bind(&socket_addr.into())?;
        socket.listen(1024)?;

        let listener: std::net::TcpListener = socket.into();
        listener.set_nonblocking(true)?;
        let listener = TcpListener::from_std(listener)?;

        println!("[SERVER] Listening on {} with SO_REUSEPORT", addr);

        loop {
            let (socket, remote_addr) = listener.accept().await?;

            // Disable Nagle's algorithm for lower latency
            let _ = socket.set_nodelay(true);

            let router = Arc::clone(&router);
            let middlewares = Arc::clone(&middlewares);
            let async_middlewares = Arc::clone(&async_middlewares);
            
            let ws_routes = Arc::clone(&ws_routes);

            tokio::spawn(async move {
                let result = handle_connection(socket, router, middlewares, async_middlewares, remote_addr, ws_routes).await;
                
                
                if let Err(e) = result {
                    eprintln!("[ERROR] Connection handler error: {}", e);
                }
            });
        }
    }

    /// Start server using configuration from Firework.toml
    pub async fn listen_with_config(self) -> Result<(), Box<dyn std::error::Error>> {
        let config = crate::config::get_config().await;
        let addr = config.bind_address();
        self.listen(&addr).await
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    router: Arc<Router>,
    middlewares: Arc<Vec<Middleware>>,
    async_middlewares: Arc<Vec<AsyncMiddleware>>,
    remote_addr: std::net::SocketAddr,
    ws_routes: Arc<std::collections::HashMap<String, Arc<dyn crate::websocket::WebSocketHandler>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut read_buf = get_buffer();

    loop {
        // Read request data
        read_buf.clear();

        let mut total_read = 0;
        loop {
            if read_buf.len() >= BUFFER_SIZE * 2 {
                // Request too large, reject
                return Err("Request too large".into());
            }

            read_buf.resize(read_buf.len() + 4096, 0);
            match socket.read(&mut read_buf[total_read..]).await {
                Ok(0) => {
                    return_buffer(read_buf);
                    return Ok(()); // Connection closed
                }
                Ok(n) => {
                    total_read += n;
                    read_buf.truncate(total_read);

                    // Check if we have complete headers (look for \r\n\r\n)
                    if let Some(pos) = find_header_end(&read_buf[..total_read]) {
                        // Parse headers
                        let mut headers = [httparse::EMPTY_HEADER; 64];
                        let mut req = httparse::Request::new(&mut headers);

                        match req.parse(&read_buf[..pos]) {
                            Ok(httparse::Status::Complete(headers_len)) => {
                                // Extract method, path, version
                                let method = parse_method(req.method.unwrap_or("GET"));
                                let path = req.path.unwrap_or("/");
                                let version = parse_version(req.version.unwrap_or(1));

                                // Parse headers into HashMap (reuse strings where possible)
                                let mut header_map = HashMap::with_capacity(req.headers.len());
                                let mut content_length = 0;
                                let mut keep_alive = version == Version::Http11; // HTTP/1.1 defaults to keep-alive

                                for header in req.headers {
                                    let name = header.name;
                                    let value = std::str::from_utf8(header.value).unwrap_or("");

                                    if name.eq_ignore_ascii_case("content-length") {
                                        content_length = value.parse().unwrap_or(0);
                                    } else if name.eq_ignore_ascii_case("connection") {
                                        keep_alive = value.eq_ignore_ascii_case("keep-alive");
                                    }

                                    header_map
                                        .entry(name.to_string())
                                        .or_insert_with(Vec::new)
                                        .push(value.to_string());
                                }

                                // Read body if Content-Length > 0
                                let body_start = headers_len;
                                let body = if content_length > 0 {
                                    let mut body_data = vec![0u8; content_length];
                                    let already_read = total_read.saturating_sub(body_start);

                                    if already_read > 0 {
                                        let copy_len = already_read.min(content_length);
                                        body_data[..copy_len].copy_from_slice(
                                            &read_buf[body_start..body_start + copy_len],
                                        );
                                    }

                                    if already_read < content_length {
                                        socket.read_exact(&mut body_data[already_read..]).await?;
                                    }

                                    body_data
                                } else {
                                    Vec::new()
                                };

                                // Parse path and query
                                let (path_only, query) = parse_path_and_query(path);
                                let uri = Uri::new(path_only, query);

                                // Create request (NO CLONING in hot path)
                                let mut request = Request::new(
                                    method,
                                    uri,
                                    version,
                                    header_map,
                                    body,
                                    Some(remote_addr),
                                );
                                let mut response = Response::default();

                                // Set keep-alive header early
                                if keep_alive {
                                    response
                                        .headers
                                        .insert("Connection".to_string(), "keep-alive".to_string());
                                }

                                // Execute middlewares (minimize cloning)
                                let mut stopped = false;

                                // Sync middlewares
                                for (i, mw) in middlewares.iter().enumerate() {
                                    let is_last =
                                        i == middlewares.len() - 1 && async_middlewares.is_empty();
                                    let req = if is_last {
                                        request.clone() // Still need clone for the conditional
                                    } else {
                                        request.clone()
                                    };

                                    match mw(req, response) {
                                        Flow::Stop(final_res) => {
                                            response = final_res;
                                            stopped = true;
                                            break;
                                        }
                                        Flow::Next(r, s) => {
                                            request = r;
                                            response = s;
                                        }
                                    }
                                }

                                // Execute async middlewares if not stopped
                                if !stopped {
                                    for (i, mw) in async_middlewares.iter().enumerate() {
                                        let is_last = i == async_middlewares.len() - 1;
                                        let req = if is_last {
                                            request.clone()
                                        } else {
                                            request.clone()
                                        };

                                        match mw(req, response).await {
                                            Flow::Stop(final_res) => {
                                                response = final_res;
                                                stopped = true;
                                                break;
                                            }
                                            Flow::Next(r, s) => {
                                                request = r;
                                                response = s;
                                            }
                                        }
                                    }
                                }

                                // Route and execute handler if not stopped
                                if !stopped {
                                    // Check if this is a WebSocket upgrade request
                                    if crate::websocket::is_websocket_upgrade(&request) {
                                        if let Some(ws_handler) = ws_routes.get(path_only) {
                                            // Perform WebSocket handshake
                                            if let Some(mut upgrade_response) = crate::websocket::websocket_upgrade(&request) {
                                                // Send upgrade response
                                                write_response(&mut socket, &mut upgrade_response, false).await?;
                                                
                                                // Create WebSocket and handle it
                                                let ws = crate::websocket::WebSocket::new(socket).await;
                                                ws_handler.call(ws).await;
                                                
                                                // WebSocket connection handled, return
                                                return_buffer(read_buf);
                                                return Ok(());
                                            }
                                        }
                                    }
                                    
                                    // Normal HTTP request handling
                                    if let Some((handler, params)) =
                                        router.find(&request.method, path_only)
                                    {
                                        request.params = params;
                                        response = handler.call(request, response).await;
                                    } else {
                                        response = Response::new(
                                            crate::response::StatusCode::NotFound,
                                            b"Not Found\n",
                                        );
                                        if keep_alive {
                                            response.headers.insert(
                                                "Connection".to_string(),
                                                "keep-alive".to_string(),
                                            );
                                        }
                                    }
                                }

                                // Write response
                                write_response(&mut socket, &mut response, keep_alive).await?;

                                if !keep_alive {
                                    return_buffer(read_buf);
                                    return Ok(());
                                }

                                // Continue to next request on same connection
                                break;
                            }
                            Ok(httparse::Status::Partial) => {
                                // Need more data
                                continue;
                            }
                            Err(_) => {
                                return_buffer(read_buf);
                                return Err("Invalid HTTP request".into());
                            }
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::task::yield_now().await;
                    continue;
                }
                Err(e) => {
                    return_buffer(read_buf);
                    return Err(e.into());
                }
            }
        }
    }
}

#[inline]
fn find_header_end(buf: &[u8]) -> Option<usize> {
    for i in 0..buf.len().saturating_sub(3) {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' && buf[i + 2] == b'\r' && buf[i + 3] == b'\n' {
            return Some(i + 4);
        }
    }
    None
}

#[inline]
fn parse_method(method: &str) -> Method {
    match method {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        "PATCH" => Method::PATCH,
        _ => Method::Unknown(method.to_string()),
    }
}

#[inline]
fn parse_version(version: u8) -> Version {
    match version {
        0 => Version::Http10,
        1 => Version::Http11,
        2 => Version::Http2,
        _ => Version::Unknown(format!("HTTP/{}", version)),
    }
}

#[inline]
fn parse_path_and_query(path: &str) -> (&str, Option<HashMap<String, String>>) {
    if let Some(pos) = path.find('?') {
        let (path_only, query_str) = path.split_at(pos);
        let query_str = &query_str[1..]; // skip '?'

        if query_str.is_empty() {
            (path_only, None)
        } else {
            // Simple query parsing (could be optimized further)
            let mut query = HashMap::new();
            for pair in query_str.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let (key, value) = pair.split_at(eq_pos);
                    query.insert(key.to_string(), value[1..].to_string());
                } else {
                    query.insert(pair.to_string(), String::new());
                }
            }
            (path_only, Some(query))
        }
    } else {
        (path, None)
    }
}

async fn write_response(
    socket: &mut TcpStream,
    response: &mut Response,
    keep_alive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Pre-allocate buffer for headers
    let mut write_buf = get_buffer();

    // Write status line
    write_buf.put_slice(b"HTTP/1.1 ");
    write_buf.put_slice(response.status.as_str().as_bytes());
    write_buf.put_slice(b"\r\n");

    // Ensure keep-alive header is set correctly
    if keep_alive {
        response
            .headers
            .insert("Connection".to_string(), "keep-alive".to_string());
    } else {
        response
            .headers
            .insert("Connection".to_string(), "close".to_string());
    }

    // Write headers
    if !response.is_streaming() {
        if let ResponseBody::Static(ref body_bytes) = response.body {
            response
                .headers
                .insert("Content-Length".to_string(), body_bytes.len().to_string());
        }
    }

    response
        .headers
        .entry("Content-Type".to_string())
        .or_insert_with(|| "text/plain; charset=utf-8".to_string());

    for (key, value) in &response.headers {
        write_buf.put_slice(key.as_bytes());
        write_buf.put_slice(b": ");
        write_buf.put_slice(value.as_bytes());
        write_buf.put_slice(b"\r\n");
    }

    write_buf.put_slice(b"\r\n");

    // Write headers
    socket.write_all(&write_buf).await?;
    return_buffer(write_buf);

    // Write body
    match &mut response.body {
        ResponseBody::Static(body_bytes) => {
            socket.write_all(body_bytes).await?;
        }
        ResponseBody::Stream(_) => {
            response.write_stream_to(socket).await?;
        }
    }

    Ok(())
}


/// Builder para crear scopes de rutas
pub struct ServerScope {
    prefix: String,
    routes: Vec<(String, String, Box<dyn AsyncHandler>)>,
}

impl ServerScope {
    pub fn route<H>(mut self, method: &str, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.routes
            .push((method.to_string(), path.to_string(), Box::new(handler)));
        self
    }

    pub fn get<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("GET", path, handler)
    }

    pub fn post<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("POST", path, handler)
    }

    pub fn put<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("PUT", path, handler)
    }

    pub fn patch<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("PATCH", path, handler)
    }

    pub fn delete<H>(self, path: &str, handler: H) -> Self
    where
        H: AsyncHandler + 'static,
    {
        self.route("DELETE", path, handler)
    }
}
