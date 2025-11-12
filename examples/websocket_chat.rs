use firework::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use lazy_static::lazy_static;

type BoxError = Box<dyn std::error::Error>;

// Simple chat room example
lazy_static! {
    static ref CHAT_ROOM: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
}

#[get("/")]
async fn index(_req: Request, res: Response) -> Response {
    res.with_header("Content-Type", "text/html; charset=utf-8")
        .text(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Firework WebSocket Chat</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        #messages { border: 1px solid #ccc; height: 400px; overflow-y: scroll; padding: 10px; margin-bottom: 10px; }
        #input { width: 70%; padding: 10px; }
        #send { padding: 10px 20px; }
        .message { margin: 5px 0; padding: 5px; background: #f0f0f0; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>🔥 Firework WebSocket Chat</h1>
    <div id="messages"></div>
    <input type="text" id="input" placeholder="Type a message...">
    <button id="send">Send</button>
    
    <script>
        const ws = new WebSocket('ws://' + window.location.host + '/ws');
        const messages = document.getElementById('messages');
        const input = document.getElementById('input');
        const sendBtn = document.getElementById('send');
        
        ws.onmessage = function(event) {
            const div = document.createElement('div');
            div.className = 'message';
            div.textContent = event.data;
            messages.appendChild(div);
            messages.scrollTop = messages.scrollHeight;
        };
        
        ws.onopen = function() {
            console.log('Connected to WebSocket server');
            const div = document.createElement('div');
            div.className = 'message';
            div.textContent = '✅ Connected to chat!';
            div.style.background = '#d4edda';
            messages.appendChild(div);
        };
        
        ws.onclose = function() {
            console.log('Disconnected from WebSocket server');
            const div = document.createElement('div');
            div.className = 'message';
            div.textContent = '❌ Disconnected from chat';
            div.style.background = '#f8d7da';
            messages.appendChild(div);
        };
        
        function sendMessage() {
            const text = input.value.trim();
            if (text && ws.readyState === WebSocket.OPEN) {
                ws.send(text);
                input.value = '';
            }
        }
        
        sendBtn.onclick = sendMessage;
        input.onkeypress = function(e) {
            if (e.key === 'Enter') sendMessage();
        };
    </script>
</body>
</html>
    "#)
}

#[ws("/ws")]
async fn chat_websocket(mut ws: WebSocket) {
    println!("🔌 New WebSocket connection established");
    
    // Send welcome message
    let _ = ws.send_text("Welcome to Firework Chat! 🔥").await;
    
    // Broadcast message history
    {
        let history = CHAT_ROOM.read().await;
        for msg in history.iter() {
            let _ = ws.send_text(msg).await;
        }
    }
    
    // Handle incoming messages
    while let Some(msg) = ws.recv().await {
        match msg {
            WebSocketMessage::Text(text) => {
                println!("📩 Received: {}", text);
                
                // Broadcast to all (in a real app, you'd use WebSocketRoom)
                let broadcast_msg = format!("User: {}", text);
                
                // Save to history
                {
                    let mut history = CHAT_ROOM.write().await;
                    history.push(broadcast_msg.clone());
                    if history.len() > 100 {
                        history.remove(0);
                    }
                }
                
                // Echo back (in a real app, broadcast to all connected clients)
                let _ = ws.send_text(&broadcast_msg).await;
            }
            WebSocketMessage::Binary(data) => {
                println!("📦 Received binary data: {} bytes", data.len());
                // Echo binary data back
                let _ = ws.send(WebSocketMessage::Binary(data)).await;
            }
            WebSocketMessage::Ping(data) => {
                println!("🏓 Received ping");
                let _ = ws.send(WebSocketMessage::Pong(data)).await;
            }
            WebSocketMessage::Pong(_) => {
                println!("🏓 Received pong");
            }
            WebSocketMessage::Close => {
                println!("👋 WebSocket connection closed");
                break;
            }
        }
    }
    
    println!("🔌 WebSocket connection ended");
}

#[tokio::main]
async fn main() -> std::result::Result<(), BoxError> {
    let server = routes!();
    
    println!("🚀 Server starting on http://127.0.0.1:8080");
    println!("📡 WebSocket endpoint: ws://127.0.0.1:8080/ws");
    println!("🌐 Open http://127.0.0.1:8080 in your browser");
    
    server.listen("127.0.0.1:8080").await?;
    Ok(())
}
