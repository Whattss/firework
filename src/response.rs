use crate::Version;
use std::collections::HashMap;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusCode {
    Ok,
    Created,
    NoContent,
    Found,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    Custom(u16, String),
}

impl StatusCode {
    pub fn as_str(&self) -> String {
        match self {
            StatusCode::Ok => "200 OK".to_string(),
            StatusCode::Created => "201 Created".to_string(),
            StatusCode::NoContent => "204 No Content".to_string(),
            StatusCode::Found => "302 Found".to_string(),
            StatusCode::BadRequest => "400 Bad Request".to_string(),
            StatusCode::Unauthorized => "401 Unauthorized".to_string(),
            StatusCode::Forbidden => "403 Forbidden".to_string(),
            StatusCode::NotFound => "404 Not Found".to_string(),
            StatusCode::InternalServerError => "500 Internal Server Error".to_string(),
            StatusCode::Custom(code, text) => format!("{code} {text}"),
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::Created => 201,
            StatusCode::NoContent => 204,
            StatusCode::Found => 302,
            StatusCode::BadRequest => 400,
            StatusCode::Unauthorized => 401,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::InternalServerError => 500,
            StatusCode::Custom(code, _) => *code,
        }
    }
}

pub enum ResponseBody {
    Static(Vec<u8>),
    Stream(Pin<Box<dyn AsyncRead + Send>>),
}

impl ResponseBody {
    pub fn len(&self) -> Option<usize> {
        match self {
            ResponseBody::Static(data) => Some(data.len()),
            ResponseBody::Stream(_) => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ResponseBody::Static(data) => data.is_empty(),
            ResponseBody::Stream(_) => false,
        }
    }
}

impl std::fmt::Debug for ResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseBody::Static(data) => write!(f, "Static({} bytes)", data.len()),
            ResponseBody::Stream(_) => write!(f, "Stream"),
        }
    }
}

pub struct Response {
    pub version: Version,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: ResponseBody,
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("version", &self.version)
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .finish()
    }
}

impl Response {
    pub fn new(status: StatusCode, body: impl Into<Vec<u8>>) -> Self {
        let body = body.into();
        let mut headers = HashMap::new();
        headers.insert("Content-Length".into(), body.len().to_string());
        headers.insert("Connection".into(), "close".into());

        Self {
            version: Version::Http11,
            status,
            headers,
            body: ResponseBody::Static(body),
        }
    }

    pub fn stream<R>(status: StatusCode, reader: R) -> Self
    where
        R: AsyncRead + Send + 'static,
    {
        let mut headers = HashMap::new();
        headers.insert("Transfer-Encoding".into(), "chunked".into());
        headers.insert("Connection".into(), "close".into());

        Self {
            version: Version::Http11,
            status,
            headers,
            body: ResponseBody::Stream(Box::pin(reader)),
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = format!("HTTP/1.1 {}\r\n", self.status.as_str());

        for (k, v) in &self.headers {
            response.push_str(&format!("{k}: {v}\r\n"));
        }

        response.push_str("\r\n");
        let mut bytes = response.into_bytes();
        
        if let ResponseBody::Static(body) = &self.body {
            bytes.extend_from_slice(body);
        }
        
        bytes
    }

    pub fn is_streaming(&self) -> bool {
        matches!(self.body, ResponseBody::Stream(_))
    }

    pub async fn write_stream_to<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: tokio::io::AsyncWrite + Unpin,
    {
        use tokio::io::AsyncWriteExt;

        if let ResponseBody::Stream(reader) = &mut self.body {
            let mut buffer = vec![0u8; 8192];
            
            loop {
                let n = reader.read(&mut buffer).await?;
                if n == 0 {
                    // End of stream
                    writer.write_all(b"0\r\n\r\n").await?;
                    break;
                }
                
                // Write chunk size in hex
                let chunk_size = format!("{:X}\r\n", n);
                writer.write_all(chunk_size.as_bytes()).await?;
                
                // Write chunk data
                writer.write_all(&buffer[..n]).await?;
                
                // Write trailing CRLF
                writer.write_all(b"\r\n").await?;
            }
        }
        
        Ok(())
    }

    /// Set response body (helper for tests)
    pub fn set_body(&mut self, body: Vec<u8>) -> &mut Self {
        self.body = ResponseBody::Static(body);
        self
    }

    /// Set plain text body
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let text_str = text.into();
        self.body = ResponseBody::Static(text_str.into_bytes());
        self.headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        self
    }

    /// Set JSON body (helper for tests)
    pub fn json(mut self, data: impl serde::Serialize) -> Self {
        match serde_json::to_vec(&data) {
            Ok(body) => {
                self.body = ResponseBody::Static(body);
                self.headers.insert("Content-Type".to_string(), "application/json".to_string());
                self
            }
            Err(_) => {
                let body = b"{\"error\":\"Failed to serialize JSON\"}".to_vec();
                self.body = ResponseBody::Static(body);
                self.status = StatusCode::InternalServerError;
                self.headers.insert("Content-Type".to_string(), "application/json".to_string());
                self
            }
        }
    }

    /// Set response status
    pub fn status(&mut self, status: StatusCode) -> &mut Self {
        self.status = status;
        self
    }
    
    /// Builder method to set header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

impl Default for Response {
    fn default() -> Self {
        Response::new(StatusCode::Ok, b"")
    }
}
