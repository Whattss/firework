//! 🔥 Firework Vite Plugin - Fullstack Rust Development
//! 
//! Seamless integration between Firework backend and Vite frontend.
//! 
//! ## Features
//! - Hot Module Replacement (HMR) proxy
//! - Development mode with Vite dev server  
//! - Production build integration
//! - Automatic asset serving
//! - SSR support (future)
//! - TypeScript API generation (future)

use firework::{Flow, Plugin, PluginError, PluginMetadata, PluginResult, Request, Response, Result, StatusCode, Error};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;

/// Vite plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViteConfig {
    /// Vite dev server port (default: 5173)
    pub dev_port: u16,
    /// Vite config file path (default: vite.config.js)
    pub config_file: String,
    /// Frontend root directory (default: ./frontend)
    pub root: PathBuf,
    /// Build output directory (default: ./frontend/dist)
    pub out_dir: PathBuf,
    /// Enable HMR proxy in development
    pub hmr: bool,
    /// Auto-start Vite dev server
    pub auto_start: bool,
}

impl Default for ViteConfig {
    fn default() -> Self {
        Self {
            dev_port: 5173,
            config_file: "vite.config.js".to_string(),
            root: PathBuf::from("./frontend"),
            out_dir: PathBuf::from("./frontend/dist"),
            hmr: true,
            auto_start: true,
        }
    }
}

/// Vite plugin state
pub struct VitePlugin {
    config: ViteConfig,
    vite_process: Arc<RwLock<Option<Child>>>,
    is_production: bool,
    http_client: Arc<reqwest::Client>,
}

impl VitePlugin {
    /// Create new Vite plugin with default config
    pub fn new() -> Self {
        Self {
            config: ViteConfig::default(),
            vite_process: Arc::new(RwLock::new(None)),
            is_production: false,
            http_client: Arc::new(
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new())
            ),
        }
    }
    
    /// Auto-configure and register Vite plugin (ONE-LINER MAGIC!)
    /// 
    /// This is the easiest way to use Vite with Firework:
    /// ```rust
    /// VitePlugin::auto();
    /// 
    /// routes!()
    ///     .listen("127.0.0.1:8080")
    ///     .await
    ///     .unwrap();
    /// ```
    /// 
    /// Features:
    /// - Auto-detects frontend directory (frontend/, client/, app/, etc.)
    /// - Auto-starts Vite dev server on :5173
    /// - Auto-proxies non-API requests to Vite
    /// - Auto-configures HMR WebSocket
    /// - Auto-installs npm dependencies if needed
    pub fn auto() -> Arc<Self> {
        let frontend_dir = Self::detect_frontend_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("./frontend"));
        
        let config = ViteConfig {
            root: frontend_dir.clone(),
            out_dir: frontend_dir.join("dist"),
            ..Default::default()
        };
        
        let plugin = Arc::new(Self::with_config(config));
        firework::register_plugin(plugin.clone());
        
        println!("[Vite] ✅ Auto-configured (frontend: {})", frontend_dir.display());
        
        plugin
    }
    
    /// Detect frontend directory automatically
    fn detect_frontend_dir() -> Option<std::path::PathBuf> {
        for dir in ["frontend", "client", "app", "ui", "web", "www"] {
            let path = std::path::PathBuf::from(format!("./{}", dir));
            if path.join("package.json").exists() || path.join("vite.config.js").exists() {
                return Some(path);
            }
        }
        None
    }

    /// Create Vite plugin with custom config
    pub fn with_config(config: ViteConfig) -> Self {
        Self {
            config,
            vite_process: Arc::new(RwLock::new(None)),
            is_production: false,
            http_client: Arc::new(
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new())
            ),
        }
    }

    /// Set production mode
    pub fn production(mut self) -> Self {
        self.is_production = true;
        self
    }

    /// Set development mode
    pub fn development(mut self) -> Self {
        self.is_production = false;
        self
    }

    /// Start Vite dev server
    pub async fn start_dev_server(&self) -> Result<()> {
        if self.is_production {
            return Ok(());
        }

        println!("[Vite] Starting dev server on port {}...", self.config.dev_port);

        let root = self.config.root.clone();
        let port = self.config.dev_port;

        // Check if vite is installed
        let check = Command::new("npm")
            .args(&["list", "vite"])
            .current_dir(&root)
            .output()
            .await;

        if check.is_err() {
            eprintln!("[Vite] Warning: Vite not found. Installing dependencies...");
            Command::new("npm")
                .args(&["install", "--production=false"])
                .current_dir(&root)
                .spawn()
                .map_err(|e| Error::Internal(format!("Failed to install dependencies: {}", e)))?
                .wait()
                .await
                .map_err(|e| Error::Internal(format!("npm install failed: {}", e)))?;
        }

        // Start Vite dev server with piped output to avoid cluttering Firework logs
        let mut child = Command::new("npm")
            .args(&["run", "dev", "--", "--port", &port.to_string(), "--host", "0.0.0.0"])
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Internal(format!("Failed to start Vite: {}", e)))?;

        // Spawn tasks to prefix and print Vite output
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    println!("[Vite] {}", line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("[Vite] {}", line);
                }
            });
        }

        *self.vite_process.write().await = Some(child);

        println!("[Vite] Dev server started at http://localhost:{}", port);
        println!("[Vite] HMR enabled");

        Ok(())
    }

    /// Build production assets
    pub async fn build(&self) -> Result<()> {
        println!("[Vite] Building production assets...");

        let output = Command::new("npm")
            .args(&["run", "build"])
            .current_dir(&self.config.root)
            .output()
            .await
            .map_err(|e| Error::Internal(format!("Build failed: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Internal(format!("Vite build failed: {}", error)));
        }

        println!("[Vite] Build complete! Output: {}", self.config.out_dir.display());
        Ok(())
    }

    /// Get Vite dev server URL
    pub fn dev_url(&self) -> String {
        format!("http://localhost:{}", self.config.dev_port)
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        self.is_production
    }
    
    /// Proxy a request to Vite dev server
    async fn proxy_request(&self, req: &Request) -> Result<Option<Response>> {
        // Build full URL with query string
        let query_string = if let Some(query) = &req.uri.query {
            let params: Vec<String> = query
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            if !params.is_empty() {
                format!("?{}", params.join("&"))
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        let vite_url = format!("{}{}{}", self.dev_url(), req.uri.path, query_string);

        // Use cached HTTP client
        let client = &self.http_client;
        
        // Build request with proper method
        let proxy_req = match req.method {
            firework::Method::GET => client.get(&vite_url),
            firework::Method::POST => client.post(&vite_url).body(req.body.clone()),
            firework::Method::PUT => client.put(&vite_url).body(req.body.clone()),
            firework::Method::DELETE => client.delete(&vite_url),
            firework::Method::PATCH => client.patch(&vite_url).body(req.body.clone()),
            _ => client.get(&vite_url),
        };

        match proxy_req.send().await {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let headers = response.headers().clone();
                
                match response.bytes().await {
                    Ok(body) => {
                        let status = match status_code {
                            200 => StatusCode::Ok,
                            404 => StatusCode::NotFound,
                            code => StatusCode::Custom(code, "Vite".to_string()),
                        };

                        let mut new_res = Response::new(status, body.to_vec());

                        // Copy headers FIRST (preserves Content-Type)
                        for (key, value) in headers.iter() {
                            if let Ok(v) = value.to_str() {
                                new_res.headers.insert(key.to_string(), v.to_string());
                            }
                        }
                        
                        // Ensure Content-Type from Vite is preserved
                        if let Some(ct) = headers.get("content-type") {
                            if let Ok(v) = ct.to_str() {
                                new_res.headers.insert("Content-Type".to_string(), v.to_string());
                            }
                        }

                        Ok(Some(new_res))
                    }
                    Err(_) => Ok(None),
                }
            }
            Err(_) => Ok(None),
        }
    }
    
    /// Auto-install npm dependencies if needed
    async fn auto_install_dependencies(&self) -> PluginResult<()> {
        let package_json = self.config.root.join("package.json");
        let node_modules = self.config.root.join("node_modules");
        
        if package_json.exists() && !node_modules.exists() {
            println!("[Vite] 📦 Installing frontend dependencies...");
            
            let status = Command::new("npm")
                .args(&["install"])
                .current_dir(&self.config.root)
                .spawn()
                .map_err(|e| PluginError(format!("Failed to spawn npm: {}", e)))?
                .wait()
                .await
                .map_err(|e| PluginError(format!("npm install failed: {}", e)))?;
            
            if status.success() {
                println!("[Vite] ✅ Dependencies installed");
            } else {
                eprintln!("[Vite] ⚠️  npm install had errors, continuing anyway...");
            }
        }
        
        Ok(())
    }
    
    /// Ensure HMR configuration in vite.config.js
    async fn ensure_hmr_config(&self) -> PluginResult<()> {
        let config_path = self.config.root.join("vite.config.js");
        
        if !config_path.exists() {
            return Ok(());
        }
        
        let content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| PluginError(format!("Failed to read vite.config.js: {}", e)))?;
        
        // Check if HMR is already configured
        if content.contains("hmr:") || content.contains("clientPort") {
            return Ok(());
        }
        
        // Auto-patch vite.config.js
        let patched = self.inject_hmr_config(&content);
        
        tokio::fs::write(&config_path, patched)
            .await
            .map_err(|e| PluginError(format!("Failed to write vite.config.js: {}", e)))?;
        
        println!("[Vite] ✅ Auto-configured HMR in vite.config.js");
        
        Ok(())
    }
    
    /// Inject HMR config into vite.config.js
    fn inject_hmr_config(&self, content: &str) -> String {
        // Simple pattern matching to inject HMR config
        if let Some(pos) = content.find("server: {") {
            let before = &content[..pos + 9]; // "server: {"
            let after = &content[pos + 9..];
            
            format!(
                "{}
    port: 5173,
    host: '0.0.0.0',
    hmr: {{
      clientPort: 5173,
      host: 'localhost',
    }},{}",
                before, after
            )
        } else if let Some(pos) = content.find("export default defineConfig({") {
            let before = &content[..pos + 29];
            let after = &content[pos + 29..];
            
            format!(
                "{}
  server: {{
    port: 5173,
    host: '0.0.0.0',
    hmr: {{
      clientPort: 5173,
      host: 'localhost',
    }},
  }},{}",
                before, after
            )
        } else {
            content.to_string()
        }
    }
}

impl Default for VitePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VitePlugin {
    fn drop(&mut self) {
        // Kill Vite process on drop (sync context)
        if let Ok(mut process) = self.vite_process.try_write() {
            if let Some(mut child) = process.take() {
                let _ = child.kill();
                println!("[Vite] Dev server stopped");
            }
        }
    }
}

#[async_trait::async_trait]
impl Plugin for VitePlugin {
    fn name(&self) -> &'static str {
        "Vite"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Vite",
            version: "0.1.0",
            author: "Firework Contributors",
            description: "Vite integration for fullstack development",
        }
    }

    async fn on_init(&self) -> PluginResult<()> {
        println!("[Vite] Initializing...");

        // Check if frontend directory exists
        if !self.config.root.exists() {
            eprintln!("[Vite] Warning: Frontend directory not found: {}", self.config.root.display());
            eprintln!("[Vite] Run: npm create vite@latest {} -- --template react", self.config.root.display());
            return Ok(());
        }
        
        // Auto-install dependencies if needed
        if !self.is_production {
            self.auto_install_dependencies().await?;
        }
        
        // Auto-configure HMR in vite.config.js
        if !self.is_production {
            self.ensure_hmr_config().await?;
        }

        Ok(())
    }

    async fn on_start(&self) -> PluginResult<()> {
        if !self.is_production && self.config.auto_start {
            self.start_dev_server()
                .await
                .map_err(|e| PluginError(format!("Failed to start Vite: {:?}", e)))?;
        } else if self.is_production {
            println!("[Vite] Production mode - serving static assets from {}", self.config.out_dir.display());
        }

        Ok(())
    }
    
    /// AUTO-PROXY MAGIC! This intercepts requests and proxies to Vite
    async fn on_request(&self, req: &mut Request, _res: &mut Response) -> PluginResult<Option<Response>> {
        // Skip API routes
        if req.uri.path.starts_with("/api") {
            return Ok(None);
        }
        
        // In production, let static file serving handle it
        if self.is_production() {
            return Ok(None);
        }
        
        // Proxy to Vite dev server
        match self.proxy_request(req).await {
            Ok(Some(response)) => Ok(Some(response)),
            Ok(None) => Ok(None),
            Err(e) => {
                eprintln!("[Vite] Proxy error: {}", e);
                Ok(None)
            }
        }
    }

    async fn on_shutdown(&self) -> PluginResult<()> {
        println!("[Vite] Shutting down...");
        
        let mut process = self.vite_process.write().await;
        if let Some(mut child) = process.take() {
            let _ = child.kill();
            println!("[Vite] Dev server stopped");
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Vite middleware - proxies requests to Vite dev server in development
pub async fn vite_middleware(
    req: &mut Request,
    _res: &mut Response,
    vite: &VitePlugin,
) -> Flow {
    // In production, let static file middleware handle it
    if vite.is_production() {
        return Flow::Continue;
    }

    // Only proxy frontend routes (not API routes)
    if req.uri.path.starts_with("/api") {
        return Flow::Continue;
    }

    // Build full URL with query string
    let query_string = if let Some(query) = &req.uri.query {
        let params: Vec<String> = query
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        if !params.is_empty() {
            format!("?{}", params.join("&"))
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    let vite_url = format!("{}{}{}", vite.dev_url(), req.uri.path, query_string);

    // Use cached HTTP client (no need to create on every request!)
    let client = &vite.http_client;
    
    // Build request with proper method and headers
    let mut proxy_req = match req.method {
        firework::Method::GET => client.get(&vite_url),
        firework::Method::POST => client.post(&vite_url).body(req.body.clone()),
        firework::Method::PUT => client.put(&vite_url).body(req.body.clone()),
        firework::Method::DELETE => client.delete(&vite_url),
        firework::Method::PATCH => client.patch(&vite_url).body(req.body.clone()),
        _ => client.get(&vite_url),
    };
    
    // Forward important headers
    for (key, values) in &req.headers {
        if !key.to_lowercase().starts_with("host") {
            if let Some(first_value) = values.first() {
                proxy_req = proxy_req.header(key, first_value);
            }
        }
    }

    match proxy_req.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let headers = response.headers().clone();
            
            match response.bytes().await {
                Ok(body) => {
                    let status = match status_code {
                        200 => StatusCode::Ok,
                        404 => StatusCode::NotFound,
                        code => StatusCode::Custom(code, "Vite".to_string()),
                    };

                    let mut new_res = Response::new(status, body.to_vec());

                    // Copy headers FIRST (this will set Content-Type before server adds default)
                    for (key, value) in headers.iter() {
                        if let Ok(v) = value.to_str() {
                            new_res.headers.insert(key.to_string(), v.to_string());
                        }
                    }
                    
                    // Ensure Content-Type from Vite is preserved
                    if let Some(ct) = headers.get("content-type") {
                        if let Ok(v) = ct.to_str() {
                            new_res.headers.insert("Content-Type".to_string(), v.to_string());
                        }
                    }

                    Flow::Stop(new_res)
                }
                Err(_) => Flow::Continue,
            }
        }
        Err(_) => {
            // Vite dev server not ready yet, serve index.html as fallback
            serve_vite_assets(req, _res, vite).await
        }
    }
}

/// Helper to serve Vite assets in production
pub async fn serve_vite_assets(
    req: &mut Request,
    _res: &mut Response,
    vite: &VitePlugin,
) -> Flow {
    if !vite.is_production() {
        return Flow::Continue;
    }

    let path = req.uri.path.trim_start_matches('/');
    let file_path = vite.config.out_dir.join(path);

    // Serve index.html for SPA routes
    let file_to_serve = if file_path.exists() && file_path.is_file() {
        file_path
    } else {
        vite.config.out_dir.join("index.html")
    };

    if !file_to_serve.exists() {
        return Flow::Continue;
    }

    match tokio::fs::read(&file_to_serve).await {
        Ok(contents) => {
            let content_type = match file_to_serve.extension().and_then(|e| e.to_str()) {
                Some("html") => "text/html; charset=utf-8",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("json") => "application/json",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                Some("ico") => "image/x-icon",
                _ => "application/octet-stream",
            };

            let mut new_res = Response::new(StatusCode::Ok, contents);
            new_res.headers.insert("Content-Type".to_string(), content_type.to_string());
            
            Flow::Stop(new_res)
        }
        Err(_) => Flow::Continue,
    }
}

/// Helper to add Vite middleware to server automatically
/// 
/// This function creates an async middleware closure that:
/// - In development: Proxies requests to Vite dev server (5173)
/// - In production: Serves static assets from dist/
/// 
/// # Example
/// ```rust
/// use firework_vite::{VitePlugin, vite_auto_middleware};
/// use std::sync::Arc;
/// 
/// let vite = Arc::new(VitePlugin::new());
/// firework::register_plugin(vite.clone());
/// 
/// firework::routes!()
///     .async_middleware(vite_auto_middleware(vite))
///     .listen("127.0.0.1:8080")
///     .await
///     .unwrap();
/// ```
pub fn vite_auto_middleware(
    vite: Arc<VitePlugin>,
) -> impl for<'a> Fn(&'a mut Request, &'a mut Response) -> std::pin::Pin<Box<dyn std::future::Future<Output = Flow> + Send + 'a>> + Clone + Send + Sync + 'static {
    move |req: &mut Request, res: &mut Response| {
        let vite = vite.clone();
        Box::pin(async move {
            vite_middleware(req, res, &vite).await
        })
    }
}

/// Macro to create Vite-enabled server
#[macro_export]
macro_rules! vite_server {
    ($vite:expr) => {{
        let vite = $vite;
        firework::Server::new()
            .middleware(move |req, res| {
                let vite = vite.clone();
                async move {
                    if vite.is_production() {
                        firework_vite::serve_vite_assets(req, res, &vite).await
                    } else {
                        firework_vite::vite_middleware(req, res, &vite).await
                    }
                }
            })
    }};
}
