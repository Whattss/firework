// Use jemalloc as the global allocator for better performance
#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod config;
mod cookie;
mod error;
mod extract;
mod macros;
mod plugin;
mod request;
mod response;
mod router;
mod perfect_hash_router;
mod phf_routes;
mod light_guard;
mod serve;
mod server;
mod upload;
mod validation;

pub mod log;
pub mod websocket;

#[cfg(feature = "hot-reload")]
pub mod hot_reload;

#[cfg(feature = "hot-reload")]
pub mod hot_reload_state;

#[cfg(any(test, feature = "testing"))]
pub mod test;

pub use config::{Config, ServerConfig, PluginConfig, config, init_config, get_config, load_plugin_config, load_plugin_config_as};
pub use cookie::{Cookie, SameSite};
pub use error::{Error, Result};
pub use extract::{FromRequest, PluginExtractor, Extract, IntoResponse, Json, Path, Query, Body, Header};
pub use plugin::{Plugin, PluginRegistry, PluginError, PluginResult, PluginMetadata, register_plugin, register_plugin_async, auto_register_plugins, registry as plugin_registry, get_plugin};
pub use request::{Method, Request, Uri, Version};
pub use response::{Response, ResponseBody, StatusCode};
pub use router::Router;
pub use serve::{serve_file, serve_dir, serve_static};
pub use server::Server;
pub use upload::{FormData, UploadedFile, UploadConfig};
pub use validation::{Validated, ValidationError, validators};

pub use websocket::{WebSocket, Message as WebSocketMessage, WebSocketHandler, WebSocketRoom, is_websocket_upgrade, websocket_upgrade};

#[cfg(feature = "hot-reload")]
pub use hot_reload::HotReload;

#[cfg(any(test, feature = "testing"))]
pub use test::{TestClient, TestRequest, TestResponse, TestExt};

// Re-export validator::Validate for convenience
pub use validator::Validate;

// Re-export macros
pub use firework_macros::{
    get, post, put, patch, delete, ws, middleware, routes, run, scope, 
    plugin, plugin_builder, firework_test,
    on_init, on_start, on_shutdown, on_reload, on_request, on_response, on_stream_accept,
    depends_on, priority
};


use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub enum Flow {
    Stop(Response),
    Continue,
}

pub trait AsyncHandler: Send + Sync {
    fn call(&self, req: Request, res: Response) -> Pin<Box<dyn Future<Output = Response> + Send>>;
}

impl<F, Fut> AsyncHandler for F
where
    F: Fn(Request, Response) -> Fut + Send + Sync,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn call(&self, req: Request, res: Response) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(self(req, res))
    }
}

pub type Middleware = fn(&mut Request, &mut Response) -> Flow;
pub type AsyncMiddleware = for<'a> fn(&'a mut Request, &'a mut Response) -> Pin<Box<dyn Future<Output = Flow> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiddlewarePhase {
    Pre,   // Ejecutar antes del handler
    Post,  // Ejecutar después del handler
}

pub enum MiddlewareHandler {
    Sync(Middleware),
    Async(AsyncMiddleware),
}

pub struct ScopeMiddleware {
    pub name: &'static str,
    pub handler: MiddlewareHandler,
    pub phase: MiddlewarePhase,
}

// Distributed slices para auto-registro
#[linkme::distributed_slice]
pub static ROUTES: [RouteInfo];

#[linkme::distributed_slice]
pub static WS_ROUTES: [WsRouteInfo];

#[linkme::distributed_slice]
pub static MIDDLEWARES: [fn(Request, Response) -> Flow];

#[linkme::distributed_slice]
pub static SCOPE_MIDDLEWARES: [ScopeMiddleware];

pub struct RouteInfo {
    pub method: &'static str,
    pub path: &'static str,
    pub handler: fn(Request, Response) -> Pin<Box<dyn Future<Output = Response> + Send>>,
    pub precomputed_hash: u64,
    pub is_static_path: bool,
}

pub struct WsRouteInfo {
    pub path: &'static str,
    pub handler: fn(WebSocket) -> Pin<Box<dyn Future<Output = ()> + Send>>,
}

/// Plugin factory for auto-registration
pub struct PluginFactory {
    pub name: &'static str,
    pub create: fn() -> std::sync::Arc<dyn Plugin>,
}

#[linkme::distributed_slice]
pub static PLUGIN_FACTORIES: [PluginFactory];

// Re-export linkme for use by macros (users don't need to add linkme dependency manually)
pub use linkme;

#[doc(hidden)]
pub mod __private {
    pub use crate::{
        const_hash_route,
        const_is_static_path,
        enforce_light_guard,
        update_routes_manifest,
        update_routes_manifest_for_source_root,
    };
    pub use linkme;
    pub use linkme::distributed_slice;
}

pub const fn const_is_static_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            return false;
        }
        i += 1;
    }
    true
}

pub const fn const_hash_route(method: &str, path: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    let method_bytes = method.as_bytes();
    let mut i = 0;
    while i < method_bytes.len() {
        hash ^= method_bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }

    hash ^= b'|' as u64;
    hash = hash.wrapping_mul(FNV_PRIME);

    let path_bytes = path.as_bytes();
    i = 0;
    while i < path_bytes.len() {
        hash ^= path_bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }

    hash
}

#[doc(hidden)]
pub fn enforce_light_guard(
    routes: &[RouteInfo],
    ws_routes: &[WsRouteInfo],
    scope_middlewares: &[ScopeMiddleware],
    plugin_factories: &[PluginFactory],
) -> std::result::Result<(), String> {
    light_guard::enforce(routes, ws_routes, scope_middlewares, plugin_factories)
}

#[doc(hidden)]
pub fn update_routes_manifest(routes: &[RouteInfo]) -> std::io::Result<()> {
    let source_root = std::env::var("FIREWORK_SOURCE_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    update_routes_manifest_inner(routes, &source_root)
}

#[doc(hidden)]
pub fn update_routes_manifest_for_source_root(
    routes: &[RouteInfo],
    source_root: &str,
) -> std::io::Result<()> {
    let source_root = std::path::Path::new(source_root);
    update_routes_manifest_inner(routes, source_root)
}

fn update_routes_manifest_inner(
    routes: &[RouteInfo],
    source_root: &std::path::Path,
) -> std::io::Result<()> {
    let path = std::env::var("FIREWORK_ROUTES_MANIFEST")
        .unwrap_or_else(|_| "target/firework/routes.manifest".to_string());
    let source_root = source_root
        .canonicalize()
        .unwrap_or_else(|_| source_root.to_path_buf());
    let source_hash = compute_source_hash(&source_root)?;

    let mut route_lines = Vec::with_capacity(routes.len());
    for route in routes {
        route_lines.push(format!(
            "route\t{}\t{}\t{}",
            route.method,
            route.path,
            if route.is_static_path { "1" } else { "0" }
        ));
    }

    route_lines.sort_unstable();
    let mut lines = Vec::with_capacity(route_lines.len() + 2);
    lines.push(format!("meta\tsource_root\t{}", source_root.display()));
    lines.push(format!("meta\tsource_hash\t{source_hash:016x}"));
    lines.extend(route_lines);
    let serialized = lines.join("\n");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if existing == serialized {
        return Ok(());
    }

    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serialized)
}

fn should_skip_source_dir(path: &std::path::Path) -> bool {
    matches!(
        path.file_name().and_then(|s| s.to_str()),
        Some("target") | Some(".git") | Some("node_modules")
    )
}

fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    if should_skip_source_dir(dir) {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        entries.push(entry?.path());
    }
    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_rs_files(&path, out)?;
            continue;
        }
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "rs")
        {
            out.push(path);
        }
    }

    Ok(())
}

fn fnv1a_update(mut hash: u64, bytes: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x100000001b3;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn compute_source_hash(source_root: &std::path::Path) -> std::io::Result<u64> {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    let mut files = Vec::new();
    collect_rs_files(source_root, &mut files)?;
    files.sort();

    let mut hash = FNV_OFFSET;
    for file in files {
        let rel = file.strip_prefix(source_root).unwrap_or(&file);
        hash = fnv1a_update(hash, rel.to_string_lossy().as_bytes());
        hash = fnv1a_update(hash, b"\0");

        let content = std::fs::read(&file)?;
        hash = fnv1a_update(hash, &content);
        hash = fnv1a_update(hash, b"\n");
    }

    Ok(hash)
}

pub mod prelude {
    pub use crate::{
        Server, Router, Request, Response, StatusCode, Method, Version, Uri,
        Error, Result, Flow, MiddlewarePhase,
        FromRequest, IntoResponse, Json, Path, Query, Body, Header,
        PluginExtractor, Extract,
        Plugin, PluginRegistry, PluginError, PluginResult, PluginMetadata,
        register_plugin, register_plugin_async, plugin_registry, get_plugin,
        Config, ServerConfig, PluginConfig, config, get_config,
        serve_file, serve_dir, serve_static,
        ResponseBody,
    };
    
    pub use crate::{WebSocket, WebSocketMessage, WebSocketHandler, WebSocketRoom, is_websocket_upgrade, websocket_upgrade};
    
    #[cfg(feature = "hot-reload")]
    pub use crate::HotReload;
    
    pub use firework_macros::{
        get, post, put, patch, delete, ws,
        middleware, routes, run, scope, 
        plugin, plugin_builder, firework_test,
        on_init, on_start, on_shutdown, on_reload, on_request, on_response, on_stream_accept,
        depends_on, priority
    };
    
    // Re-export common macros
    pub use crate::{json, html, text, redirect, stream};
    
    // Re-export serde for user convenience
    pub use serde::{Deserialize, Serialize};
    pub use serde_json;
    pub use validator::Validate;
    
    
    #[cfg(any(test, feature = "testing"))]
    pub use crate::{TestClient, TestRequest, TestResponse, TestExt};
    
    pub use crate::{ROUTES, MIDDLEWARES, SCOPE_MIDDLEWARES};
    
    #[derive(Debug)]
    pub enum MiddlewareResult {
        Continue,
        Stop(Response),
    }
}
