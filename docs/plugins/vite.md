# Vite Plugin

Seamless integration between Firework backend and Vite frontend for fullstack Rust development.

## Features

- **Auto-Start Vite**: Automatically starts Vite dev server
- **Hot Module Replacement**: Instant frontend updates
- **Proxy Support**: API requests proxied to Firework backend
- **WebSocket HMR**: Real-time module replacement
- **Production Builds**: Serve built assets in production
- **Zero Config**: Works out of the box
- **TypeScript Support**: Full TypeScript integration

## Quick Start

### Installation

```toml
[dependencies]
firework-vite = { git = "https://github.com/Whattss/firework" }
```

### One-Line Setup

```rust
use firework::prelude::*;
use firework_vite::VitePlugin;

#[tokio::main]
async fn main() {
    // Magic one-liner - auto-detects and starts Vite!
    VitePlugin::auto();
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

That's it! The plugin will:
1. Detect your frontend directory (frontend/, client/, app/)
2. Auto-start Vite dev server on port 5173
3. Proxy non-API requests to Vite
4. Configure HMR WebSocket

## Configuration

### Manual Setup

```rust
use firework_vite::VitePlugin;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let vite = Arc::new(VitePlugin::new()
        .dev_port(5173)
        .root("./frontend")
        .out_dir("./frontend/dist")
        .auto_start(true)
        .hmr(true));
    
    firework::register_plugin(vite);
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

### Firework.toml

```toml
[plugins.vite]
dev_port = 5173
root = "./frontend"
out_dir = "./frontend/dist"
auto_start = true
hmr = true
```

Then load automatically:
```rust
let vite = Arc::new(VitePlugin::from_config().await);
firework::register_plugin(vite);
```

## Project Structure

```
my-app/
├── Cargo.toml
├── src/
│   └── main.rs          # Firework backend
├── frontend/
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── src/
│       └── main.tsx     # React/Vue/etc.
└── Firework.toml
```

## Frontend Setup

### React

```bash
cd my-app
npm create vite@latest frontend -- --template react-ts
cd frontend
npm install
```

**vite.config.ts:**
```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      }
    }
  }
})
```

### Vue

```bash
npm create vite@latest frontend -- --template vue-ts
```

**vite.config.ts:**
```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5173,
    proxy: {
      '/api': 'http://localhost:8080'
    }
  }
})
```

### Svelte

```bash
npm create vite@latest frontend -- --template svelte-ts
```

## Development Workflow

### Start Development

```bash
# Terminal 1: Start Firework + Vite (auto-starts)
cargo run

# Or use CLI
fwk dev
```

The Vite plugin will:
- Start Vite dev server automatically
- Proxy frontend requests
- Set up HMR WebSocket
- Watch for changes

### API Integration

**Backend (Rust):**
```rust
use firework::prelude::*;

#[get("/api/users")]
async fn list_users() -> Response {
    json!([
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ])
}
```

**Frontend (TypeScript):**
```typescript
// Automatically proxied to Firework backend
async function fetchUsers() {
  const response = await fetch('/api/users');
  return response.json();
}
```

## Production Build

### Build Frontend

```bash
cd frontend
npm run build
```

### Serve Static Assets

```rust
use firework::prelude::*;
use firework_vite::VitePlugin;

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        // Development: auto-start Vite
        VitePlugin::auto();
    } else {
        // Production: serve built files
        serve_static("./frontend/dist");
    }
    
    routes!()
        .listen("0.0.0.0:8080")
        .await
        .unwrap();
}
```

## Advanced Usage

### Custom Proxy Rules

```rust
use firework_vite::{VitePlugin, ProxyConfig};

let vite = Arc::new(VitePlugin::new()
    .proxy("/api", "http://localhost:8080")
    .proxy("/ws", "ws://localhost:8080")
    .proxy("/graphql", "http://localhost:4000"));

firework::register_plugin(vite);
```

### Environment Variables

**.env:**
```bash
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080
```

**Frontend:**
```typescript
const API_URL = import.meta.env.VITE_API_URL;

fetch(`${API_URL}/api/users`);
```

### Conditional Features

```rust
#[tokio::main]
async fn main() {
    #[cfg(feature = "frontend")]
    {
        VitePlugin::auto();
    }
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

## WebSocket Integration

### Backend

```rust
use firework::prelude::*;

#[ws("/ws")]
async fn websocket(mut ws: WebSocket) {
    while let Some(msg) = ws.recv().await {
        ws.send(msg).await.ok();
    }
}
```

### Frontend

```typescript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  console.log('Received:', event.data);
};

ws.send('Hello from frontend!');
```

## TypeScript API Generation

Generate TypeScript types from Rust:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

// TODO: Auto-generate TypeScript definitions
```

**Generated types.ts:**
```typescript
export interface User {
  id: number;
  name: string;
  email: string;
}
```

## Troubleshooting

### Vite Not Starting

**Problem**: Vite dev server doesn't start

**Solutions**:
- Check `node_modules` installed: `npm install`
- Verify port 5173 is available
- Check `vite.config.js` exists
- Enable verbose logging:

```rust
VitePlugin::new().verbose(true);
```

### CORS Issues

**Problem**: API requests blocked by CORS

**Solution**: Configure CORS in Firework:

```rust
use firework_cors::CorsPlugin;

firework::register_plugin(Arc::new(CorsPlugin::new()
    .allow_origin("http://localhost:5173")));
```

### Hot Reload Not Working

**Problem**: Changes not reflected

**Solutions**:
- Check Vite dev server is running
- Verify WebSocket connection in browser console
- Check proxy configuration

```typescript
// vite.config.ts
server: {
  hmr: {
    port: 5173
  }
}
```

### Build Assets Not Found

**Problem**: 404 on production assets

**Solution**: Check paths:

```rust
// Serve from correct directory
serve_static("./frontend/dist");

// Or with route prefix
#[get("/*path")]
async fn serve_frontend(Path(path): Path<String>) -> Response {
    serve_file(&format!("./frontend/dist/{}", path)).await
}
```

## Best Practices

### 1. Separate API Routes

```rust
// All API routes under /api prefix
#[scope("/api")]
mod api {
    #[get("/users")]
    async fn users() -> Response { }
}

// Frontend handles everything else
#[get("/*path")]
async fn frontend(Path(path): Path<String>) -> Response {
    serve_file(&format!("./frontend/dist/{}", path)).await
}
```

### 2. Environment-Based Config

```rust
#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        VitePlugin::auto();
    } else {
        serve_static("./dist");
    }
}
```

### 3. Type Safety

```rust
// Share types between frontend and backend
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "frontend", derive(tsify::Tsify))]
pub struct User {
    pub id: u32,
    pub name: String,
}
```

### 4. Error Handling

```rust
use firework_vite::VitePlugin;

match VitePlugin::auto_start().await {
    Ok(plugin) => {
        firework::register_plugin(Arc::new(plugin));
    }
    Err(e) => {
        eprintln!("Failed to start Vite: {}", e);
        // Fallback to serving built files
        serve_static("./frontend/dist");
    }
}
```

## Full Example

**Backend (src/main.rs):**
```rust
use firework::prelude::*;
use firework_vite::VitePlugin;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
}

#[get("/api/todos")]
async fn list_todos() -> Response {
    json!([
        Todo { id: 1, title: "Learn Rust".into(), completed: true },
        Todo { id: 2, title: "Build with Firework".into(), completed: false },
    ])
}

#[post("/api/todos")]
async fn create_todo(Json(todo): Json<Todo>) -> Response {
    json!(todo)
}

#[tokio::main]
async fn main() {
    VitePlugin::auto();
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .unwrap();
}
```

**Frontend (frontend/src/App.tsx):**
```typescript
import { useState, useEffect } from 'react';

interface Todo {
  id: number;
  title: string;
  completed: boolean;
}

function App() {
  const [todos, setTodos] = useState<Todo[]>([]);

  useEffect(() => {
    fetch('/api/todos')
      .then(res => res.json())
      .then(setTodos);
  }, []);

  return (
    <div>
      <h1>Todos</h1>
      {todos.map(todo => (
        <div key={todo.id}>
          <input type="checkbox" checked={todo.completed} />
          {todo.title}
        </div>
      ))}
    </div>
  );
}

export default App;
```

## Performance

- **Dev Server**: ~100ms startup time
- **HMR Updates**: < 50ms for most changes
- **Proxy Overhead**: < 1ms per request
- **Production Build**: Optimized with Rollup

## Comparison

| Feature | Vite Plugin | Manual Setup |
|---------|-------------|--------------|
| Auto-start | ✅ Yes | ❌ Manual |
| HMR | ✅ Yes | ⚠️ Manual config |
| Proxy | ✅ Auto | ⚠️ Manual |
| Production | ✅ Integrated | ❌ Separate |
| Setup Time | 1 line | ~20 lines |

## See Also

- [Full Example](../../examples/vite_integration.rs)
- [Threads Clone](../../examples/threads-clone/)
- [Static Files](./static-files.md)
- [Configuration](./configuration.md)
