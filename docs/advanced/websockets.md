# 🌐 WebSockets

Real-time bidirectional communication with WebSockets in Firework.

---

## Basic WebSocket

```rust
use firework::prelude::*;

#[ws("/ws")]
async fn websocket_handler(mut ws: WebSocket) {
    println!("Client connected!");
    
    while let Some(msg) = ws.recv().await {
        match msg {
            Message::Text(text) => {
                println!("Received: {}", text);
                ws.send(Message::Text(text)).await.ok();
            }
            Message::Close => {
                println!("Client disconnected");
                break;
            }
            _ => {}
        }
    }
}
```

---

## Chat Room Example

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref ROOM: Arc<WebSocketRoom> = Arc::new(WebSocketRoom::new());
}

#[ws("/chat")]
async fn chat_handler(mut ws: WebSocket) {
    // Add to room
    let ws_ref = ROOM.add(ws).await;
    
    // Listen for messages
    loop {
        let msg = {
            let mut socket = ws_ref.lock().await;
            socket.recv().await
        };
        
        match msg {
            Some(Message::Text(text)) => {
                // Broadcast to all
                ROOM.broadcast(Message::Text(text)).await;
            }
            Some(Message::Close) | None => {
                ROOM.remove(&ws_ref).await;
                break;
            }
            _ => {}
        }
    }
}
```

---

## Message Types

```rust
// Text message
ws.send(Message::Text("Hello".into())).await?;

// Binary message  
ws.send(Message::Binary(vec![1, 2, 3])).await?;

// Ping/Pong
ws.send(Message::Ping(vec![])).await?;
ws.send(Message::Pong(vec![])).await?;

// Close connection
ws.send(Message::Close).await?;
ws.close().await?;
```

---

## Testing

```javascript
// JavaScript client
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    console.log('Connected');
    ws.send('Hello Server!');
};

ws.onmessage = (event) => {
    console.log('Received:', event.data);
};

ws.onclose = () => {
    console.log('Disconnected');
};
```
