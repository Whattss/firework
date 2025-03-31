// src/error.rs
use std::fmt;

/// Error centralizado para el servidor.
#[derive(Debug)]
pub enum ServerError {
    InvalidRequest,
    NotFound,
    InternalServerError(String), // Puede contener detalles adicionales
    BadRequest(String),
    ConnectionClosed
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::InvalidRequest => write!(f, "Invalid request"),
            ServerError::NotFound => write!(f, "Not found"),
            ServerError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ServerError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ServerError::ConnectionClosed => write!(f, "Connection Closed"),
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::InternalServerError(err.to_string())
    }
}
