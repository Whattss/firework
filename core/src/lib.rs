mod config;
mod error;
mod extract;
mod macros;
mod plugin;
mod request;
mod response;
mod router;
mod serve;
mod server;

#[cfg(feature = "websockets")]
pub mod websocket;

#[cfg(feature = "hot-reload")]
pub mod hot_reload;

#[cfg(feature = "hot-reload")]
pub mod hot_reload_state;

#[cfg(any(test, feature = "testing"))]
pub mod test;

pub use config::{Config, ServerConfig, PluginConfig, config, init_config, get_config, load_plugin_config, load_plugin_config_as};
pub use error::{Error, Result};
pub use extract::{FromRequest, PluginExtractor, Extract, IntoResponse, Json, Path, Query, Body, Header};
pub use plugin::{Plugin, PluginRegistry, PluginError, PluginResult, PluginMetadata, register_plugin, register_plugin_async, auto_register_plugins, registry as plugin_registry, get_plugin};
pub use request::{Method, Request, Uri, Version};
pub use response::{Response, ResponseBody, StatusCode};
pub use router::Router;
pub use serve::{serve_file, serve_dir, serve_static};
pub use server::Server;

#[cfg(feature = "websockets")]
pub use websocket::{WebSocket, Message as WebSocketMessage, WebSocketHandler, WebSocketRoom, is_websocket_upgrade, websocket_upgrade};

#[cfg(feature = "hot-reload")]
pub use hot_reload::HotReload;

#[cfg(any(test, feature = "testing"))]
pub use test::{TestClient, TestRequest, TestResponse, TestExt};

// Re-export macros
#[cfg(feature = "websockets")]
pub use firework_macros::{
    get, post, put, patch, delete, ws, middleware, routes, scope, 
    plugin, plugin_builder, firework_test,
    on_init, on_start, on_shutdown, on_reload, on_request, on_response, on_stream_accept,
    depends_on, priority
};

#[cfg(not(feature = "websockets"))]
pub use firework_macros::{
    get, post, put, patch, delete, middleware, routes, scope, 
    plugin, plugin_builder, firework_test,
    on_init, on_start, on_shutdown, on_reload, on_request, on_response, on_stream_accept,
    depends_on, priority
};

use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub enum Flow {
    Stop(Response),
    Next(Request, Response),
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

pub type Middleware = fn(Request, Response) -> Flow;
pub type AsyncMiddleware = fn(Request, Response) -> Pin<Box<dyn Future<Output = Flow> + Send>>;

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

#[cfg(feature = "websockets")]
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
}

#[cfg(feature = "websockets")]
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
    pub use linkme;
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
    
    #[cfg(feature = "websockets")]
    pub use crate::{WebSocket, WebSocketMessage, WebSocketHandler, WebSocketRoom, is_websocket_upgrade, websocket_upgrade};
    
    #[cfg(feature = "hot-reload")]
    pub use crate::HotReload;
    
    #[cfg(feature = "websockets")]
    pub use firework_macros::{
        get, post, put, patch, delete, ws,
        middleware, routes, scope, 
        plugin, plugin_builder, firework_test,
        on_init, on_start, on_shutdown, on_reload, on_request, on_response, on_stream_accept,
        depends_on, priority
    };
    
    #[cfg(not(feature = "websockets"))]
    pub use firework_macros::{
        get, post, put, patch, delete,
        middleware, routes, scope, 
        plugin, plugin_builder, firework_test,
        on_init, on_start, on_shutdown, on_reload, on_request, on_response, on_stream_accept,
        depends_on, priority
    };
    
    #[cfg(any(test, feature = "testing"))]
    pub use crate::{TestClient, TestRequest, TestResponse, TestExt};
    
    pub use crate::{ROUTES, MIDDLEWARES, SCOPE_MIDDLEWARES};
    
    #[derive(Debug)]
    pub enum MiddlewareResult {
        Continue,
        Stop(Response),
    }
}
