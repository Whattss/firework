# 🔥 Firework Vite - Fullstack Rust Development

**The first truly fullstack Rust web framework.**

Seamlessly integrates Vite frontend tooling with Firework backend, giving you the best of both worlds:
- ⚡ Lightning-fast HMR with Vite
- 🦀 Type-safe Rust backend
- 🎯 Zero configuration
- 🔄 Automatic proxying
- 📦 Production-ready builds

## ✨ Features

### Development Mode
- **Hot Module Replacement** - Instant updates without page reload
- **Automatic Vite Proxy** - Backend proxies to Vite dev server
- **API Routes** - `/api/*` automatically go to Rust backend
- **Asset Serving** - Vite serves all frontend assets
- **Error Overlay** - Vite's beautiful error UI

### Production Mode
- **Optimized Builds** - Vite builds are automatically served
- **SPA Routing** - HTML5 history mode support
- **Asset Optimization** - Minification, tree-shaking, code-splitting
- **Static Serving** - Efficient static file serving

## 🚀 Quick Start

### 1. Create Project Structure

```bash
my-app/
├── backend/         # Rust backend
│   ├── src/
│   │   └── main.rs
│   └── Cargo.toml
└── frontend/        # Vite frontend
    ├── src/
    │   ├── main.ts
    │   └── App.vue
    ├── package.json
    └── vite.config.ts
```

### 2. Backend Setup

```rust
// backend/src/main.rs
use firework::prelude::*;
use firework_vite::VitePlugin;
use std::sync::Arc;

#[get("/api/hello")]
async fn hello() -> &'static str {
    "Hello from Rust!"
}

#[tokio::main]
async fn main() {
    // Create Vite plugin
    let vite = Arc::new(VitePlugin::new());
    
    // Development mode (auto-starts Vite)
    // let vite = Arc::new(VitePlugin::new().development());
    
    // Production mode (serves dist/)
    // let vite = Arc::new(VitePlugin::new().production());

    let server = Server::new()
        // Vite middleware (must be first!)
        .middleware(move |req, res| {
            let vite = vite.clone();
            async move {
                if vite.is_production() {
                    firework_vite::serve_vite_assets(req, res, &vite).await
                } else {
                    firework_vite::vite_middleware(req, res, &vite).await
                }
            }
        })
        // API routes
        .get("/api/hello", hello);

    println!("🔥 Firework server running on http://localhost:8080");
    server.listen("0.0.0.0:8080").await.unwrap();
}
```

### 3. Frontend Setup

```typescript
// frontend/vite.config.ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue' // or react, etc.

export default defineConfig({
  plugins: [vue()],
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      }
    }
  }
})
```

```json
// frontend/package.json
{
  "name": "my-app-frontend",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "dependencies": {
    "vue": "^3.0.0"
  },
  "devDependencies": {
    "vite": "^5.0.0",
    "@vitejs/plugin-vue": "^5.0.0"
  }
}
```

### 4. Run!

```bash
# Development (single command!)
cd backend && cargo run

# Vite starts automatically on port 5173
# Backend proxies frontend requests to Vite
# Visit http://localhost:8080
```

## 📖 Usage Examples

### Basic Setup
```rust
use firework_vite::{VitePlugin, ViteConfig};

let vite = VitePlugin::new(); // Default config
```

### Custom Configuration
```rust
let config = ViteConfig {
    dev_port: 3000,
    root: PathBuf::from("./frontend"),
    out_dir: PathBuf::from("./dist"),
    hmr: true,
    auto_start: true,
    ..Default::default()
};

let vite = VitePlugin::with_config(config);
```

### Environment-Based
```rust
let vite = if cfg!(debug_assertions) {
    VitePlugin::new().development()
} else {
    VitePlugin::new().production()
};
```

### With API Routes
```rust
#[get("/api/users")]
async fn get_users() -> Json<Vec<User>> {
    // Your logic
    Json(users)
}

let server = Server::new()
    .middleware(vite_middleware)  // Frontend
    .get("/api/users", get_users);  // Backend API
```

## 🎯 How It Works

### Development Flow

```
User Request → Firework (port 8080)
                ↓
    Is it /api/*? 
       ↓        ↓
      Yes       No
       ↓        ↓
   Backend    Proxy to Vite (port 5173)
               ↓
           Vite serves with HMR
```

### Production Flow

```
User Request → Firework (port 8080)
                ↓
    Is it /api/*?
       ↓        ↓
      Yes       No
       ↓        ↓
   Backend    Serve from dist/
               ↓
           Optimized assets
```

## 🔧 Configuration Options

```rust
pub struct ViteConfig {
    /// Vite dev server port (default: 5173)
    pub dev_port: u16,
    
    /// Vite config file path
    pub config_file: String,
    
    /// Frontend root directory
    pub root: PathBuf,
    
    /// Build output directory
    pub out_dir: PathBuf,
    
    /// Enable HMR proxy
    pub hmr: bool,
    
    /// Auto-start Vite dev server
    pub auto_start: bool,
}
```

## 📦 Build for Production

```bash
# 1. Build frontend
cd frontend && npm run build

# 2. Run backend in production mode
cd backend && cargo run --release
```

Or automate with a script:

```bash
#!/bin/bash
# build.sh

echo "Building frontend..."
cd frontend && npm run build

echo "Building backend..."
cd backend && cargo build --release

echo "Done! Run: ./backend/target/release/my-app"
```

## 🎨 Framework Examples

### Vue 3
```typescript
// frontend/src/main.ts
import { createApp } from 'vue'
import App from './App.vue'

createApp(App).mount('#app')
```

### React
```typescript
// frontend/src/main.tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
```

### Svelte
```typescript
// frontend/src/main.ts
import App from './App.svelte'

const app = new App({
  target: document.getElementById('app')!,
})

export default app
```

## 🔌 Integration with Other Plugins

### With SeaORM
```rust
use firework_vite::VitePlugin;
use firework_seaorm::SeaOrmPlugin;

let vite = Arc::new(VitePlugin::new());
let db = Arc::new(SeaOrmPlugin::new("sqlite::memory:"));

let server = Server::new()
    .middleware(vite_middleware)
    .get("/api/users", get_users);  // Uses DB
```

### With Auth
```rust
use firework_auth::AuthPlugin;

let auth = Arc::new(AuthPlugin::new());

let server = Server::new()
    .middleware(vite_middleware)
    .middleware(auth_middleware)
    .get("/api/protected", protected_route);
```

## 🚨 Common Patterns

### API Prefix
```rust
// All API routes under /api
scope("/api")
    .get("/users", get_users)
    .post("/users", create_user)
    .get("/posts", get_posts)
```

### SPA Fallback
```rust
// Automatically handled! 
// Unknown routes serve index.html in production
// Proxied to Vite in development
```

### Environment Variables
```rust
// backend
std::env::var("NODE_ENV")
    .unwrap_or_else(|_| "development".to_string())
```

```typescript
// frontend (Vite)
import.meta.env.VITE_API_URL
```

## 🎯 Best Practices

1. **Always prefix API routes with `/api`**
   ```rust
   #[get("/api/users")]  // ✅
   #[get("/users")]      // ❌ Will try Vite first
   ```

2. **Vite middleware first**
   ```rust
   Server::new()
       .middleware(vite_middleware)  // ✅ First!
       .middleware(auth)
       .middleware(logging)
   ```

3. **Environment detection**
   ```rust
   let vite = if cfg!(debug_assertions) {
       VitePlugin::new().development()
   } else {
       VitePlugin::new().production()
   };
   ```

4. **Build script**
   ```bash
   npm run build && cargo build --release
   ```

## 🔜 Coming Soon

- [ ] **TypeScript API Generation** - Auto-generate TS types from Rust
- [ ] **SSR Support** - Server-side rendering with Rust
- [ ] **WebSocket Integration** - HMR over WebSocket
- [ ] **Asset Preprocessing** - Image optimization, etc.
- [ ] **Template CLI** - `firework new fullstack my-app`

## 🎓 Example Projects

See `examples/fullstack/` for complete examples:
- Vue 3 + Firework
- React + Firework
- Svelte + Firework

## 📝 License

MIT

---

**Firework + Vite = The Future of Fullstack Rust** 🚀
