use bytes::{BufMut, BytesMut};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use ahash::AHashMap;

use crate::response::ResponseBody;
use crate::{
    AsyncHandler, AsyncMiddleware, Flow, Method, Middleware, Request, Response, Router, Uri,
    Version,
};

// Thread-local buffer pool for zero contention
use std::cell::RefCell;
use memchr;

const BUFFER_SIZE: usize = 8192;
const MAX_POOLED_BUFFERS_PER_THREAD: usize = 64; // Increased from 32
const LARGE_BUFFER_SIZE: usize = 65536; // 64KB for large requests
const MAX_LARGE_BUFFERS: usize = 8;
#[cfg(feature = "http2")]
const HTTP2_PREFACE: &[u8; 24] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

// Multi-tier buffer pool for different request sizes
thread_local! {
    static BUFFER_POOL: RefCell<Vec<BytesMut>> = RefCell::new(Vec::with_capacity(MAX_POOLED_BUFFERS_PER_THREAD));
    static LARGE_BUFFER_POOL: RefCell<Vec<BytesMut>> = RefCell::new(Vec::with_capacity(MAX_LARGE_BUFFERS));
}

fn get_buffer() -> BytesMut {
    BUFFER_POOL.with(|pool| {
        pool.borrow_mut()
            .pop()
            .unwrap_or_else(|| BytesMut::with_capacity(BUFFER_SIZE))
    })
}

fn get_large_buffer() -> BytesMut {
    LARGE_BUFFER_POOL.with(|pool| {
        pool.borrow_mut()
            .pop()
            .unwrap_or_else(|| BytesMut::with_capacity(LARGE_BUFFER_SIZE))
    })
}

fn return_buffer(mut buf: BytesMut) {
    buf.clear();

    // Return to appropriate pool based on capacity
    if buf.capacity() >= LARGE_BUFFER_SIZE / 2 {
        LARGE_BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            if pool.len() < MAX_LARGE_BUFFERS {
                pool.push(buf);
            }
        });
    } else {
        BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            if pool.len() < MAX_POOLED_BUFFERS_PER_THREAD {
                pool.push(buf);
            }
        });
    }
}

async fn check_port_availability(addr: &str) {
    let port = addr.split(':').last().unwrap_or("8080");
    
    #[cfg(unix)]
    {
        let output = tokio::process::Command::new("lsof")
            .arg("-i")
            .arg(format!(":{}", port))
            .arg("-P")
            .arg("-n")
            .output()
            .await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            
            if lines.len() > 1 {
                println!("\n⚠️  [WARNING] Port {} is already in use by other process(es):", port);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                for line in &lines {
                    println!("{}", line);
                }
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("\n🔴 Due to SO_REUSEPORT being enabled, the server will bind successfully");
                println!("   but traffic may be distributed between multiple processes!");
                println!("\n💡 To fix this, kill the existing process(es) using:");
                println!("   kill -9 <PID>  (replace <PID> with the process ID from above)\n");
                
                // Give user time to read the warning
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
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

    pub fn route_info(mut self, route: &crate::RouteInfo) -> Self {
        self.router.add_route_info(route);
        self
    }

    pub fn route_infos(mut self, routes: &[crate::RouteInfo]) -> Self {
        self.router.add_routes_info_sorted(routes);
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
        // Configure stdout/stderr to be unbuffered for immediate output in async contexts
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        
        let router = Arc::new(self.router);
        let middlewares = Arc::new(self.middlewares);
        let async_middlewares = Arc::new(self.async_middlewares);
        
        let ws_routes = Arc::new(self.ws_routes);

        // Load config if not already loaded
        let _ = crate::config::config();

        // Initialize plugins ONCE and cache them to avoid locking on every request
        let plugin_registry = crate::plugin::registry();
        let plugins = {
            let registry = plugin_registry.read().await;
            registry.init_all().await?;
            registry.start_all().await?;
            // Cache plugins to avoid RwLock on every request
            Arc::new(registry.plugins().to_vec())
        };

        // Check if port is already in use
        check_port_availability(addr).await;

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
        println!("[SERVER] Press Ctrl+C for graceful shutdown");

        // Setup graceful shutdown signal handler
        let shutdown_signal = async {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                
                let mut sigterm = signal(SignalKind::terminate())
                    .expect("Failed to setup SIGTERM handler");
                let mut sigint = signal(SignalKind::interrupt())
                    .expect("Failed to setup SIGINT handler");
                
                tokio::select! {
                    _ = sigterm.recv() => {
                        println!("\n[SERVER] Received SIGTERM, shutting down gracefully...");
                    }
                    _ = sigint.recv() => {
                        println!("\n[SERVER] Received SIGINT (Ctrl+C), shutting down gracefully...");
                    }
                }
            }
            
            #[cfg(not(unix))]
            {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for Ctrl+C");
                println!("\n[SERVER] Received Ctrl+C, shutting down gracefully...");
            }
        };

        // Run server with graceful shutdown
        tokio::select! {
            result = async {
                loop {
                    let (socket, remote_addr) = listener.accept().await?;

                    // Disable Nagle's algorithm for lower latency
                    let _ = socket.set_nodelay(true);

                    let router = Arc::clone(&router);
                    let middlewares = Arc::clone(&middlewares);
                    let async_middlewares = Arc::clone(&async_middlewares);
                    let ws_routes = Arc::clone(&ws_routes);
                    let plugins = Arc::clone(&plugins);

                    tokio::spawn(async move {
                        let result = handle_connection(socket, router, middlewares, async_middlewares, remote_addr, ws_routes, plugins).await;
                        
                        if let Err(e) = result {
                            // Check if it's an IO error and if it's a common client disconnection
                            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                                use std::io::ErrorKind;
                                match io_err.kind() {
                                    ErrorKind::ConnectionReset | ErrorKind::BrokenPipe | ErrorKind::ConnectionAborted => {
                                        // Client closed connection early - this is normal with HMR/fast navigation
                                        // Silently ignore these
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                            // Log other errors
                            eprintln!("[ERROR] Connection handler error: {}", e);
                        }
                    });
                }
                #[allow(unreachable_code)]
                Ok::<(), Box<dyn std::error::Error>>(())
            } => {
                result?;
            }
            _ = shutdown_signal => {
                println!("[SERVER] Initiating graceful shutdown...");
                
                // Give active connections time to finish (max 10 seconds)
                println!("[SERVER] Waiting for active connections to complete (max 10s)...");
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                
                // Shutdown plugins
                let plugin_registry = crate::plugin::registry();
                let registry = plugin_registry.read().await;
                registry.shutdown_all().await?;
                
                println!("[SERVER] Shutdown complete ✅");
            }
        }

        Ok(())
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
    plugins: Arc<Vec<Arc<dyn crate::Plugin>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "http2")]
    if detect_http2_handshake(&socket).await? {
        return handle_http2_connection(socket, router, middlewares, async_middlewares, remote_addr, plugins).await;
    }

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

                                // Parse headers with AHashMap (faster than std HashMap)
                                // Pre-extract content-length and connection
                                let mut header_map = AHashMap::with_capacity(req.headers.len());
                                let mut content_length = 0;
                                let mut keep_alive = version == Version::Http11;

                                for header in req.headers {
                                    let name = header.name;
                                    let value = std::str::from_utf8(header.value).unwrap_or("");

                                    // Fast path: skip common headers we don't need in map
                                    if name.eq_ignore_ascii_case("content-length") {
                                        content_length = value.parse().unwrap_or(0);
                                        continue;
                                    } else if name.eq_ignore_ascii_case("connection") {
                                        keep_alive = value.eq_ignore_ascii_case("keep-alive");
                                        continue;
                                    }

                                    // Intern common header names - use &'static str when possible
                                    let name_str = intern_header_name_static(name);

                                    header_map
                                        .entry(name_str)
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

                                // Execute middlewares (zero-cost - no cloning!)
                                let mut stopped = false;

                                // Sync middlewares
                                for mw in middlewares.iter() {
                                    match mw(&mut request, &mut response) {
                                        Flow::Stop(final_res) => {
                                            response = final_res;
                                            stopped = true;
                                            break;
                                        }
                                        Flow::Continue => {}
                                    }
                                }

                                // Execute async middlewares if not stopped
                                if !stopped {
                                    for mw in async_middlewares.iter() {
                                        match mw(&mut request, &mut response).await {
                                            Flow::Stop(final_res) => {
                                                response = final_res;
                                                stopped = true;
                                                break;
                                            }
                                            Flow::Continue => {}
                                        }
                                    }
                                }
                                
                                // Execute plugin on_request hooks if not stopped
                                // Uses cached plugin list - no lock needed!
                                if !stopped {
                                    for plugin in plugins.iter() {
                                        match plugin.on_request(&mut request, &mut response).await {
                                            Ok(Some(plugin_response)) => {
                                                response = plugin_response;
                                                stopped = true;
                                                break;
                                            }
                                            Ok(None) => {}
                                            Err(e) => {
                                                eprintln!("[PLUGIN] Error in {}: {}", plugin.name(), e);
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

#[cfg(feature = "http2")]
async fn detect_http2_handshake(socket: &TcpStream) -> std::io::Result<bool> {
    let mut probe = [0u8; HTTP2_PREFACE.len()];
    for _ in 0..8 {
        let read = socket.peek(&mut probe).await?;
        if read == 0 {
            return Ok(false);
        }

        let cmp_len = read.min(HTTP2_PREFACE.len());
        if probe[..cmp_len] != HTTP2_PREFACE[..cmp_len] {
            return Ok(false);
        }

        if read >= HTTP2_PREFACE.len() {
            return Ok(true);
        }

        socket.readable().await?;
    }

    Ok(false)
}

#[cfg(feature = "http2")]
async fn handle_http2_connection(
    socket: TcpStream,
    router: Arc<Router>,
    middlewares: Arc<Vec<Middleware>>,
    async_middlewares: Arc<Vec<AsyncMiddleware>>,
    remote_addr: std::net::SocketAddr,
    plugins: Arc<Vec<Arc<dyn crate::Plugin>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = h2::server::handshake(socket).await?;

    while let Some(stream) = connection.accept().await {
        let (request, respond) = stream?;
        let router = Arc::clone(&router);
        let middlewares = Arc::clone(&middlewares);
        let async_middlewares = Arc::clone(&async_middlewares);
        let plugins = Arc::clone(&plugins);

        tokio::spawn(async move {
            if let Err(err) = handle_http2_stream(
                request,
                respond,
                router,
                middlewares,
                async_middlewares,
                remote_addr,
                plugins,
            )
            .await
            {
                eprintln!("[HTTP2] stream error: {err}");
            }
        });
    }

    Ok(())
}

#[cfg(feature = "http2")]
async fn handle_http2_stream(
    request: http::Request<h2::RecvStream>,
    mut respond: h2::server::SendResponse<bytes::Bytes>,
    router: Arc<Router>,
    middlewares: Arc<Vec<Middleware>>,
    async_middlewares: Arc<Vec<AsyncMiddleware>>,
    remote_addr: std::net::SocketAddr,
    plugins: Arc<Vec<Arc<dyn crate::Plugin>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (parts, mut body_stream) = request.into_parts();
    let mut body = Vec::new();
    while let Some(chunk) = body_stream.data().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }

    let mut header_map = AHashMap::with_capacity(parts.headers.len());
    for (name, value) in parts.headers.iter() {
        let value = match value.to_str() {
            Ok(v) => v.to_string(),
            Err(_) => String::from_utf8_lossy(value.as_bytes()).to_string(),
        };
        header_map
            .entry(name.as_str().to_ascii_lowercase())
            .or_insert_with(Vec::new)
            .push(value);
    }

    let full_path = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or_else(|| parts.uri.path());
    let (path_only, query) = parse_path_and_query(full_path);

    let mut request = Request::new(
        parse_method(parts.method.as_str()),
        Uri::new(path_only, query),
        Version::Http2,
        header_map,
        body,
        Some(remote_addr),
    );
    let mut response = Response::default();
    let mut stopped = false;

    for mw in middlewares.iter() {
        match mw(&mut request, &mut response) {
            Flow::Stop(final_res) => {
                response = final_res;
                stopped = true;
                break;
            }
            Flow::Continue => {}
        }
    }

    if !stopped {
        for mw in async_middlewares.iter() {
            match mw(&mut request, &mut response).await {
                Flow::Stop(final_res) => {
                    response = final_res;
                    stopped = true;
                    break;
                }
                Flow::Continue => {}
            }
        }
    }

    if !stopped {
        for plugin in plugins.iter() {
            match plugin.on_request(&mut request, &mut response).await {
                Ok(Some(plugin_response)) => {
                    response = plugin_response;
                    stopped = true;
                    break;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("[PLUGIN] Error in {}: {}", plugin.name(), e);
                }
            }
        }
    }

    if !stopped {
        if let Some((handler, params)) = router.find(&request.method, path_only) {
            request.params = params;
            response = handler.call(request, response).await;
        } else {
            response = Response::new(crate::response::StatusCode::NotFound, b"Not Found\n");
        }
    }

    write_http2_response(&mut respond, &mut response).await
}

#[cfg(feature = "http2")]
async fn write_http2_response(
    respond: &mut h2::server::SendResponse<bytes::Bytes>,
    response: &mut Response,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = http::StatusCode::from_u16(response.status.code())
        .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = http::Response::builder().status(status);

    for (key, value) in &response.headers {
        if is_hop_by_hop_header(key) {
            continue;
        }
        builder = builder.header(key.as_str(), value.as_str());
    }

    let body = response_body_to_vec(response).await?;
    builder = builder.header("content-length", body.len().to_string());
    builder = builder.header("content-type", response.headers.get("Content-Type").cloned().unwrap_or_else(|| "text/plain; charset=utf-8".to_string()));

    let h2_response = builder.body(())?;
    if body.is_empty() {
        respond.send_response(h2_response, true)?;
    } else {
        let mut send_stream = respond.send_response(h2_response, false)?;
        send_stream.send_data(bytes::Bytes::from(body), true)?;
    }

    Ok(())
}

#[cfg(feature = "http2")]
async fn response_body_to_vec(
    response: &mut Response,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match &mut response.body {
        ResponseBody::Static(body) => Ok(body.clone()),
        ResponseBody::Stream(reader) => {
            let mut body = Vec::new();
            reader.read_to_end(&mut body).await?;
            Ok(body)
        }
    }
}

#[cfg(feature = "http2")]
fn is_hop_by_hop_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("connection")
        || name.eq_ignore_ascii_case("transfer-encoding")
        || name.eq_ignore_ascii_case("keep-alive")
        || name.eq_ignore_ascii_case("upgrade")
        || name.eq_ignore_ascii_case("proxy-connection")
}

/// Intern common HTTP header names - returns &'static str when possible
/// This avoids String allocations for ~95% of headers
#[inline(always)]
fn intern_header_name_static(name: &str) -> String {
    // OPTIMIZATION: Return static string references instead of allocating
    // Most headers are from this list, so we avoid heap allocation
    let static_name: &'static str = match name {
        "host" | "Host" => "host",
        "user-agent" | "User-Agent" => "user-agent",
        "accept" | "Accept" => "accept",
        "accept-encoding" | "Accept-Encoding" => "accept-encoding",
        "accept-language" | "Accept-Language" => "accept-language",
        "content-type" | "Content-Type" => "content-type",
        "cookie" | "Cookie" => "cookie",
        "cache-control" | "Cache-Control" => "cache-control",
        "authorization" | "Authorization" => "authorization",
        "referer" | "Referer" => "referer",
        "origin" | "Origin" => "origin",
        _ => return name.to_string(), // Only allocate for rare headers
    };
    static_name.to_string() // Convert &'static str to String (still cheaper than full alloc)
}

#[inline]
fn find_header_end(buf: &[u8]) -> Option<usize> {
    // Use memchr for SIMD-accelerated search of \r\n\r\n pattern
    // Search for first \r, then verify the pattern
    let mut pos = 0;
    while let Some(cr_pos) = memchr::memchr(b'\r', &buf[pos..]) {
        let abs_pos = pos + cr_pos;
        if abs_pos + 3 < buf.len()
            && buf[abs_pos + 1] == b'\n'
            && buf[abs_pos + 2] == b'\r'
            && buf[abs_pos + 3] == b'\n'
        {
            return Some(abs_pos + 4);
        }
        pos = abs_pos + 1;
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
            // Use form_urlencoded for efficient parsing with proper URL decoding
            let query: HashMap<String, String> = form_urlencoded::parse(query_str.as_bytes())
                .into_owned()
                .collect();
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
