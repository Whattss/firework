// src/request.rs
use std::collections::HashMap;

/// Representa una solicitud HTTP.
#[derive(Clone, Debug)]
pub struct Request {
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub params: HashMap<String, String>,
    pub extra: HashMap<String, String>
}

impl Request {
    pub fn new(path: String, headers: HashMap<String, String>, body: String) -> Self {
        Request {
            path,
            headers,
            body,
            params: HashMap::new(),
            extra: HashMap::new()
        }
    }
}
