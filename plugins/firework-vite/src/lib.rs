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

use firework::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
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
}

impl VitePlugin {
    /// Create new Vite plugin with default config
    pub fn new() -> Self {
        Self {
            config: ViteConfig::default(),
            vite_process: Arc::new(RwLock::new(None)),
            is_production: false,
        }
    }

    /// Create Vite plugin with custom config
    pub fn with_config(config: ViteConfig) -> Self {
        Self {
            config,
            vite_process: Arc::new(RwLock::new(None)),
            is_production: false,
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
            .output();

        if check.is_err() {
            eprintln!("[Vite] Warning: Vite not found. Installing dependencies...");
            Command::new("npm")
                .arg("install")
                .current_dir(&root)
                .spawn()
                .map_err(|e| Error::Internal(format!("Failed to install dependencies: {}", e)))?
                .wait()
                .map_err(|e| Error::Internal(format!("npm install failed: {}", e)))?;
        }

        // Start Vite dev server
        let child = Command::new("npm")
            .args(&["run", "dev", "--", "--port", &port.to_string()])
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Internal(format!("Failed to start Vite: {}", e)))?;

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

    fn metadata(&self) -> firework::PluginMetadata {
        firework::PluginMetadata {
            name: "Vite",
            version: "0.1.0",
            author: "Firework Contributors",
            description: "Vite integration for fullstack development",
        }
    }

    async fn on_init(&self) -> firework::PluginResult<()> {
        println!("[Vite] Initializing...");

        // Check if frontend directory exists
        if !self.config.root.exists() {
            eprintln!("[Vite] Warning: Frontend directory not found: {}", self.config.root.display());
            eprintln!("[Vite] Run: firework init frontend");
        }

        Ok(())
    }

    async fn on_start(&self) -> firework::PluginResult<()> {
        if !self.is_production && self.config.auto_start {
            self.start_dev_server()
                .await
                .map_err(|e| firework::PluginError(format!("Failed to start Vite: {:?}", e)))?;
        } else if self.is_production {
            println!("[Vite] Production mode - serving static assets from {}", self.config.out_dir.display());
        }

        Ok(())
    }

    async fn on_shutdown(&self) -> firework::PluginResult<()> {
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
    res: &mut Response,
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

    // Proxy to Vite dev server
    let vite_url = format!("{}{}", vite.dev_url(), req.uri.path);

    match reqwest::get(&vite_url).await {
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

                    *res = Response::new(status, body.to_vec());

                    // Copy headers
                    for (key, value) in headers.iter() {
                        if let Ok(v) = value.to_str() {
                            res.headers.insert(key.to_string(), v.to_string());
                        }
                    }

                    Flow::Stop(res.clone())
                }
                Err(_) => Flow::Continue,
            }
        }
        Err(_) => {
            // Vite dev server not ready yet
            Flow::Continue
        }
    }
}

/// Helper to serve Vite assets in production
pub async fn serve_vite_assets(
    req: &mut Request,
    res: &mut Response,
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

            *res = Response::new(StatusCode::Ok, contents);
            res.headers.insert("Content-Type".to_string(), content_type.to_string());
            
            Flow::Stop(res.clone())
        }
        Err(_) => Flow::Continue,
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
