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
    /// Conflict (409)
    Conflict(String),
    /// Gone (410)
    Gone(String),
    /// Payload Too Large (413)
    PayloadTooLarge(String),
    /// URI Too Long (414)
    UriTooLong(String),
    /// Too Many Requests (429)
    TooManyRequests(String),
    /// Service Unavailable (503)
    ServiceUnavailable(String),
    /// Gateway Timeout (504)
    GatewayTimeout(String),
    /// Method Not Allowed (405)
    MethodNotAllowed(String),
    /// Not Acceptable (406)
    NotAcceptable(String),
    /// Request Timeout (408)
    RequestTimeout(String),
    /// Unprocessable Entity (422)
    UnprocessableEntity(String),
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
            Error::Conflict(msg) => write!(f, "Conflict: {}", msg),
            Error::Gone(msg) => write!(f, "Gone: {}", msg),
            Error::PayloadTooLarge(msg) => write!(f, "Payload too large: {}", msg),
            Error::UriTooLong(msg) => write!(f, "URI too long: {}", msg),
            Error::TooManyRequests(msg) => write!(f, "Too many requests: {}", msg),
            Error::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
            Error::GatewayTimeout(msg) => write!(f, "Gateway timeout: {}", msg),
            Error::MethodNotAllowed(msg) => write!(f, "Method not allowed: {}", msg),
            Error::NotAcceptable(msg) => write!(f, "Not acceptable: {}", msg),
            Error::RequestTimeout(msg) => write!(f, "Request timeout: {}", msg),
            Error::UnprocessableEntity(msg) => write!(f, "Unprocessable entity: {}", msg),
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
    /// Convierte el error a una respuesta HTTP (consume el error)
    pub fn into_response(self) -> crate::Response {
        use crate::{Response, StatusCode};
        
        let (status, message) = match self {
            Error::ParseError(msg) => (StatusCode::BadRequest, msg),
            Error::IoError(err) => (StatusCode::InternalServerError, err.to_string()),
            Error::JsonError(err) => (StatusCode::BadRequest, err.to_string()),
            Error::ValidationError(msg) => (StatusCode::BadRequest, msg),
            Error::NotFound(msg) => (StatusCode::NotFound, msg),
            Error::Unauthorized(msg) => (StatusCode::Unauthorized, msg),
            Error::Forbidden(msg) => (StatusCode::Forbidden, msg),
            Error::BadRequest(msg) => (StatusCode::BadRequest, msg),
            Error::Internal(msg) => (StatusCode::InternalServerError, msg),
            Error::Custom(msg) => (StatusCode::InternalServerError, msg),
            Error::CustomWithCode(code, msg) => (StatusCode::Custom(code, "Custom".into()), msg),
            Error::Conflict(msg) => (StatusCode::Custom(409, "Conflict".into()), msg),
            Error::Gone(msg) => (StatusCode::Custom(410, "Gone".into()), msg),
            Error::PayloadTooLarge(msg) => (StatusCode::Custom(413, "Payload Too Large".into()), msg),
            Error::UriTooLong(msg) => (StatusCode::Custom(414, "URI Too Long".into()), msg),
            Error::TooManyRequests(msg) => (StatusCode::Custom(429, "Too Many Requests".into()), msg),
            Error::ServiceUnavailable(msg) => (StatusCode::Custom(503, "Service Unavailable".into()), msg),
            Error::GatewayTimeout(msg) => (StatusCode::Custom(504, "Gateway Timeout".into()), msg),
            Error::MethodNotAllowed(msg) => (StatusCode::Custom(405, "Method Not Allowed".into()), msg),
            Error::NotAcceptable(msg) => (StatusCode::Custom(406, "Not Acceptable".into()), msg),
            Error::RequestTimeout(msg) => (StatusCode::Custom(408, "Request Timeout".into()), msg),
            Error::UnprocessableEntity(msg) => (StatusCode::Custom(422, "Unprocessable Entity".into()), msg),
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
