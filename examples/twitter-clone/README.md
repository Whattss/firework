# 🔥 Twitter Clone - Firework + Vite

Minimalist Twitter/X clone built with Firework backend and Vite frontend.

## Features

- ✅ Post tweets (280 chars)
- ✅ Like/Unlike tweets  
- ✅ Retweet/Unretweet
- ✅ Real-time updates
- ✅ Dark theme (pure black)
- ✅ Responsive design

## Quick Start

```bash
# Install frontend dependencies
cd frontend
npm install

# Run (from backend directory)
cd ../backend
cargo run

# Visit http://localhost:8080
```

## Tech Stack

- **Backend:** Firework (Rust) - 200k+ req/s
- **Frontend:** Vue 3 + Vite - HMR enabled
- **State:** In-memory
- **API:** REST

## API Endpoints

```
GET  /api/tweets
POST /api/tweets  
POST /api/tweets/:id/like
POST /api/tweets/:id/retweet
```

---

Built with ❤️ using Firework
