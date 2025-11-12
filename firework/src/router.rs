use std::collections::HashMap;
use std::sync::Arc;

use crate::AsyncHandler;
use crate::Method;

type HandlerBox = Arc<dyn AsyncHandler>;

#[derive(Clone)]
pub struct RadixNode {
    path: String,
    children: Vec<RadixNode>,
    handlers: HashMap<String, HandlerBox>, // method -> handler
    is_param: bool,
    param_name: Option<String>,
}

impl RadixNode {
    fn new() -> Self {
        RadixNode {
            path: String::new(),
            children: Vec::new(),
            handlers: HashMap::new(),
            is_param: false,
            param_name: None,
        }
    }

    pub fn insert(&mut self, method: &str, path: &str, handler: Box<dyn AsyncHandler>) {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        self.insert_parts(method, &parts, Arc::from(handler));
    }

    fn insert_parts(&mut self, method: &str, parts: &[&str], handler: HandlerBox) {
        if parts.is_empty() {
            self.handlers.insert(method.to_uppercase(), handler);
            return;
        }

        let part = parts[0];
        let remaining = &parts[1..];

        // Verificar si es un parámetro
        let (is_param, search_key, param_name) = if part.starts_with(':') {
            (true, ":param".to_string(), Some(part[1..].to_string()))
        } else {
            (false, part.to_string(), None)
        };

        // Buscar hijo existente
        for child in &mut self.children {
            if (child.is_param && is_param) || (!child.is_param && child.path == search_key) {
                child.insert_parts(method, remaining, handler);
                return;
            }
        }

        // Crear nuevo hijo
        let mut child = RadixNode::new();
        child.path = search_key;
        child.is_param = is_param;
        child.param_name = param_name;
        child.insert_parts(method, remaining, handler);
        self.children.push(child);
    }

    pub fn search(&self, method: &Method, path: &str) -> Option<(HandlerBox, HashMap<String, String>)> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let method_str = method_to_string(method);
        self.search_parts(&method_str, &parts, HashMap::new())
    }

    fn search_parts(&self, method: &str, parts: &[&str], mut params: HashMap<String, String>) -> Option<(HandlerBox, HashMap<String, String>)> {
        if parts.is_empty() {
            return self.handlers.get(method).map(|h| (Arc::clone(h), params));
        }

        let part = parts[0];
        let remaining = &parts[1..];

        // Primero intentar match exacto
        for child in &self.children {
            if !child.is_param && child.path == part {
                if let Some(result) = child.search_parts(method, remaining, params.clone()) {
                    return Some(result);
                }
            }
        }

        // Luego intentar match con parámetros
        for child in &self.children {
            if child.is_param {
                if let Some(param_name) = &child.param_name {
                    params.insert(param_name.clone(), part.to_string());
                }
                if let Some(result) = child.search_parts(method, remaining, params.clone()) {
                    return Some(result);
                }
            }
        }

        None
    }
}

#[inline]
fn method_to_string(method: &Method) -> String {
    match method {
        Method::GET => "GET".to_string(),
        Method::POST => "POST".to_string(),
        Method::PUT => "PUT".to_string(),
        Method::DELETE => "DELETE".to_string(),
        Method::HEAD => "HEAD".to_string(),
        Method::OPTIONS => "OPTIONS".to_string(),
        Method::PATCH => "PATCH".to_string(),
        Method::Unknown(s) => s.to_uppercase(),
    }
}

pub struct Router {
    root: RadixNode,
}

impl Router {
    pub fn new() -> Self {
        Router {
            root: RadixNode::new(),
        }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: Box<dyn AsyncHandler>) {
        self.root.insert(method, path, handler);
    }

    pub fn find(&self, method: &Method, path: &str) -> Option<(HandlerBox, HashMap<String, String>)> {
        self.root.search(method, path)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
