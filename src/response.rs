// src/response.rs
use std::collections::HashMap;
use crate::error::ServerError;

/// Representa una respuesta HTTP.
pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status_code: 200,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    /// Define el header "Content-Type".
    pub fn set_content_type(&mut self, content_type: &str) {
        self.headers.insert("Content-Type".to_string(), content_type.to_string());
    }

    /// Genera una respuesta de error en base al tipo de error recibido.
    pub fn error_response(&mut self, error: ServerError) {
        match error {
            ServerError::NotFound => {
                self.status_code = 404;
                self.body = "404 Not Found: The requested resource could not be found.".to_string();
            }
            ServerError::InvalidRequest => {
                self.status_code = 400;
                self.body = "400 Bad Request: Invalid request format.".to_string();
            }
            ServerError::InternalServerError(msg) => {
                self.status_code = 500;
                self.body = format!("500 Internal Server Error: {}", msg);
            }
            ServerError::BadRequest(msg) => {
                self.status_code = 400;
                self.body = format!("400 Bad Request: {}", msg);
            }
            ServerError::ConnectionClosed => {
                self.status_code = 499;
                self.body = format!("499 Connection Closed by client")
            }
        }
        self.headers.insert("Content-Type".to_string(), "application/json; charset=utf-8".to_string());
    }

    /// Convierte la respuesta a una cadena formateada como respuesta HTTP.
    pub fn to_string(&self) -> String {
        let mut response = format!("HTTP/1.1 {} OK\r\n", self.status_code);
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");
        response.push_str(&self.body);
        response
    }
}
