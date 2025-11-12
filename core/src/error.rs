use std::fmt;

/// Tipo de resultado estándar para handlers
pub type Result<T> = std::result::Result<T, Error>;

/// Error del framework
#[derive(Debug)]
pub enum Error {
    /// Error de parsing
    ParseError(String),
    /// Error de IO
    IoError(std::io::Error),
    /// Error de serialización JSON
    JsonError(serde_json::Error),
    /// Error de validación
    ValidationError(String),
    /// Not Found
    NotFound(String),
    /// Unauthorized
    Unauthorized(String),
    /// Forbidden
    Forbidden(String),
    /// Bad Request
    BadRequest(String),
    /// Internal Server Error
    Internal(String),
    /// Custom error with status code
    Custom(String),
    /// Custom error with specific status code
    CustomWithCode(u16, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::IoError(err) => write!(f, "IO error: {}", err),
            Error::JsonError(err) => write!(f, "JSON error: {}", err),
            Error::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Error::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Error::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
            Error::Custom(msg) => write!(f, "{}", msg),
            Error::CustomWithCode(code, msg) => write!(f, "Error {}: {}", code, msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonError(err)
    }
}

impl Error {
    /// Convierte el error a una respuesta HTTP
    pub fn into_response(self) -> crate::Response {
        use crate::{Response, StatusCode};
        
        let (status, message) = match &self {
            Error::ParseError(msg) => (StatusCode::BadRequest, msg.clone()),
            Error::IoError(err) => (StatusCode::InternalServerError, err.to_string()),
            Error::JsonError(err) => (StatusCode::BadRequest, err.to_string()),
            Error::ValidationError(msg) => (StatusCode::BadRequest, msg.clone()),
            Error::NotFound(msg) => (StatusCode::NotFound, msg.clone()),
            Error::Unauthorized(msg) => (StatusCode::Custom(401, "Unauthorized".into()), msg.clone()),
            Error::Forbidden(msg) => (StatusCode::Forbidden, msg.clone()),
            Error::BadRequest(msg) => (StatusCode::BadRequest, msg.clone()),
            Error::Internal(msg) => (StatusCode::InternalServerError, msg.clone()),
            Error::Custom(msg) => (StatusCode::InternalServerError, msg.clone()),
            Error::CustomWithCode(code, msg) => (StatusCode::Custom(*code, "Custom".into()), msg.clone()),
        };
        
        let body = serde_json::json!({
            "error": message,
            "status": status.code(),
        });
        
        let mut response = Response::new(status, serde_json::to_vec(&body).unwrap_or_default());
        response.headers.insert("Content-Type".to_string(), "application/json".to_string());
        response
    }
}
