use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Version {
    Http10,
    Http11,
    Http2,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct Uri {
    pub path: String,
    pub query: Option<HashMap<String, String>>,
}

impl Uri {
    pub fn new(path: &str, query: Option<HashMap<String, String>>) -> Self {
        Uri {
            path: String::from(path),
            query,
        }
    }
}

#[derive(Clone)]
pub struct Context {
    data: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn insert<T: Any + Send + Sync + Clone>(&mut self, value: T) {
        if let Ok(mut data) = self.data.write() {
            data.insert(TypeId::of::<T>(), Box::new(value));
        }
    }
    
    pub fn get<T: Any + Send + Sync + Clone>(&self) -> Option<T> {
        if let Ok(data) = self.data.read() {
            data.get(&TypeId::of::<T>())
                .and_then(|boxed| boxed.downcast_ref::<T>())
                .cloned()
        } else {
            None
        }
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(data) = self.data.read() {
            f.debug_struct("Context")
                .field("items", &data.len())
                .finish()
        } else {
            f.debug_struct("Context")
                .field("items", &"<locked>")
                .finish()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Vec<u8>,
    pub remote_addr: Option<std::net::SocketAddr>,
    pub params: HashMap<String, String>,
    pub context: Context,
}

impl Request {
    pub fn new(
        method: Method,
        uri: Uri,
        version: Version,
        headers: HashMap<String, Vec<String>>,
        body: Vec<u8>,
        remote_addr: Option<std::net::SocketAddr>,
    ) -> Self {
        Request {
            method,
            uri,
            version,
            headers,
            body,
            remote_addr,
            params: HashMap::new(),
            context: Context::new(),
        }
    }
    
    /// Insert a value into the request context
    pub fn set_context<T: Any + Send + Sync + Clone>(&mut self, value: T) {
        self.context.insert(value);
    }
    
    /// Get a value from the request context
    pub fn get_context<T: Any + Send + Sync + Clone>(&self) -> Option<T> {
        self.context.get()
    }
    
    /// Get a route parameter by name
    pub fn param(&self, name: &str) -> Option<&String> {
        self.params.get(name)
    }
    
    /// Get a route parameter as a specific type
    pub fn param_as<T>(&self, name: &str) -> Option<T>
    where
        T: std::str::FromStr,
    {
        self.params.get(name)?.parse().ok()
    }
    
    /// Get a header value by name (returns first value if multiple exist)
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)?.first().map(|s| s.as_str())
    }
    
    /// Get all header values by name
    pub fn header_all(&self, name: &str) -> Option<&Vec<String>> {
        self.headers.get(name)
    }
    
    /// Get query parameter from URI (if query is parsed)
    pub fn query(&self, name: &str) -> Option<&String> {
        self.uri.query.as_ref()?.get(name)
    }
    
    /// Get the request body as a UTF-8 string
    pub fn body_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }
    
    /// Get the request body as a str slice
    pub fn body_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }
}
