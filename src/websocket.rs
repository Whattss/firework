/// WebSocket support for Firework
/// 
/// Provides ergonomic WebSocket handling with automatic upgrade from HTTP requests.
/// 
/// # Example
/// 
/// ```rust
/// use firework::prelude::*;
/// 
/// #[websocket("/ws")]
/// async fn chat_handler(mut ws: WebSocket) {
///     while let Some(msg) = ws.recv().await {
///         match msg {
///             Message::Text(text) => {
///                 println!("Received: {}", text);
///                 ws.send(Message::Text(text)).await.ok();
///             }
///             Message::Close => break,
///             _ => {}
///         }
///     }
/// }
/// ```

use crate::{Request, Response, StatusCode};
use std::pin::Pin;
use std::future::Future;
use tokio::net::TcpStream;

use tokio_tungstenite::{
    tungstenite::{
        protocol::{Message as WsMessage, Role},
        Error as WsError,
    },
    WebSocketStream,
};

use futures_util::{SinkExt, StreamExt};

/// WebSocket message types
#[derive(Debug, Clone)]
pub enum Message {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Vec<u8>),
    /// Ping message
    Ping(Vec<u8>),
    /// Pong message
    Pong(Vec<u8>),
    /// Close message
    Close,
}

impl From<WsMessage> for Message {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Text(text) => Message::Text(text),
            WsMessage::Binary(data) => Message::Binary(data),
            WsMessage::Ping(data) => Message::Ping(data),
            WsMessage::Pong(data) => Message::Pong(data),
            WsMessage::Close(_) => Message::Close,
            WsMessage::Frame(_) => Message::Close, // Shouldn't happen
        }
    }
}

impl From<Message> for WsMessage {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(text) => WsMessage::Text(text),
            Message::Binary(data) => WsMessage::Binary(data),
            Message::Ping(data) => WsMessage::Ping(data),
            Message::Pong(data) => WsMessage::Pong(data),
            Message::Close => WsMessage::Close(None),
        }
    }
}

/// WebSocket connection
pub struct WebSocket {
    stream: WebSocketStream<TcpStream>,
}

impl WebSocket {
    /// Create a new WebSocket from a TCP stream
    pub(crate) async fn new(stream: TcpStream) -> Self {
        Self {
            stream: WebSocketStream::from_raw_socket(stream, Role::Server, None).await,
        }
    }


    /// Receive a message from the WebSocket
    pub async fn recv(&mut self) -> Option<Message> {
        match self.stream.next().await {
            Some(Ok(msg)) => Some(msg.into()),
            _ => None,
        }
    }


    /// Send a message to the WebSocket
    pub async fn send(&mut self, msg: Message) -> Result<(), WsError> {
        self.stream.send(msg.into()).await
    }


    /// Close the WebSocket connection
    pub async fn close(&mut self) -> Result<(), WsError> {
        self.stream.close(None).await
    }

    /// Send text message (convenience method)
    pub async fn send_text(&mut self, text: impl Into<String>) -> Result<(), Box<dyn std::error::Error>> {
        self.send(Message::Text(text.into())).await.map_err(|e| e.into())
    }

    /// Send binary message (convenience method)
    pub async fn send_binary(&mut self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        self.send(Message::Binary(data)).await.map_err(|e| e.into())
    }

    /// Broadcast to multiple WebSockets
    pub async fn broadcast(sockets: &mut [WebSocket], msg: Message) {
        for socket in sockets {
            let _ = socket.send(msg.clone()).await;
        }
    }
}

/// WebSocket handler trait
pub trait WebSocketHandler: Send + Sync {
    fn call(&self, ws: WebSocket) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl<F, Fut> WebSocketHandler for F
where
    F: Fn(WebSocket) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn call(&self, ws: WebSocket) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(self(ws))
    }
}

/// Check if a request is a WebSocket upgrade request
pub fn is_websocket_upgrade(req: &Request) -> bool {
    let upgrade = req.header("Upgrade").unwrap_or("");
    let connection = req.header("Connection").unwrap_or("");
    
    upgrade.eq_ignore_ascii_case("websocket") 
        && connection.to_lowercase().contains("upgrade")
}

/// Perform WebSocket handshake and return upgrade response
pub fn websocket_upgrade(req: &Request) -> Option<Response> {
    if !is_websocket_upgrade(req) {
        return None;
    }

    let key = req.header("Sec-WebSocket-Key")?;
    let accept_key = generate_accept_key(key);

    let mut response = Response::new(StatusCode::Custom(101, "Switching Protocols".into()), b"");
    response.headers.insert("Upgrade".to_string(), "websocket".to_string());
    response.headers.insert("Connection".to_string(), "Upgrade".to_string());
    response.headers.insert("Sec-WebSocket-Accept".to_string(), accept_key);
    
    Some(response)
}

/// Generate WebSocket accept key from client key
fn generate_accept_key(key: &str) -> String {
    use sha1::{Digest, Sha1};
    use base64::Engine;
    
    const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WEBSOCKET_GUID.as_bytes());
    let hash = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(&hash)
}


/// WebSocket room for managing multiple connections (channel-based for better performance)
pub struct WebSocketRoom {
    broadcast_tx: tokio::sync::broadcast::Sender<Message>,
    connection_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl WebSocketRoom {
    /// Create a new WebSocket room
    pub fn new() -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        Self {
            broadcast_tx: tx,
            connection_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Create with custom channel capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel(capacity);
        Self {
            broadcast_tx: tx,
            connection_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Subscribe to room broadcasts - returns a receiver for this connection
    /// 
    /// Use this in a loop to receive broadcasts:
    /// ```ignore
    /// let mut rx = room.subscribe();
    /// while let Ok(msg) = rx.recv().await {
    ///     ws.send(msg).await.ok();
    /// }
    /// ```
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Message> {
        self.connection_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.broadcast_tx.subscribe()
    }

    /// Unsubscribe (call when connection closes)
    pub fn unsubscribe(&self) {
        self.connection_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Broadcast a message to all connections in the room (non-blocking!)
    /// 
    /// Returns the number of receivers the message was sent to
    pub fn broadcast(&self, msg: Message) -> usize {
        self.broadcast_tx.send(msg).unwrap_or(0)
    }

    /// Get the number of active connections
    pub fn len(&self) -> usize {
        self.connection_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Check if the room is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for WebSocketRoom {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy WebSocketRoom with mutex-based connections (deprecated)
/// Use the new channel-based WebSocketRoom instead for better performance
#[deprecated(note = "Use the new channel-based WebSocketRoom instead")]
pub struct LegacyWebSocketRoom {
    connections: std::sync::Arc<tokio::sync::RwLock<Vec<std::sync::Arc<tokio::sync::Mutex<WebSocket>>>>>,
}

#[allow(deprecated)]
impl LegacyWebSocketRoom {
    pub fn new() -> Self {
        Self {
            connections: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn add(&self, ws: WebSocket) -> std::sync::Arc<tokio::sync::Mutex<WebSocket>> {
        let ws = std::sync::Arc::new(tokio::sync::Mutex::new(ws));
        self.connections.write().await.push(ws.clone());
        ws
    }

    pub async fn remove(&self, ws: &std::sync::Arc<tokio::sync::Mutex<WebSocket>>) {
        let mut conns = self.connections.write().await;
        if let Some(pos) = conns.iter().position(|c| std::sync::Arc::ptr_eq(c, ws)) {
            conns.remove(pos);
        }
    }

    pub async fn broadcast(&self, msg: Message) {
        let conns = self.connections.read().await;
        for conn in conns.iter() {
            if let Ok(mut ws) = conn.try_lock() {
                let _ = ws.send(msg.clone()).await;
            }
        }
    }

    pub async fn len(&self) -> usize {
        self.connections.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.connections.read().await.is_empty()
    }
}

#[allow(deprecated)]
impl Default for LegacyWebSocketRoom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_conversion() {
        let text_msg = Message::Text("hello".to_string());
        assert!(matches!(text_msg, Message::Text(_)));

        let binary_msg = Message::Binary(vec![1, 2, 3]);
        assert!(matches!(binary_msg, Message::Binary(_)));
    }

    #[test]
    fn test_websocket_room() {
        let room = WebSocketRoom::new();
        assert_eq!(room.len(), 0);
        assert!(room.is_empty());
        
        // Test subscribe
        let _rx1 = room.subscribe();
        assert_eq!(room.len(), 1);
        
        let _rx2 = room.subscribe();
        assert_eq!(room.len(), 2);
        
        room.unsubscribe();
        assert_eq!(room.len(), 1);
    }
}
