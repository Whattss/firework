# 📦 Installation

## Prerequisites

Before installing Firework, ensure you have:

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Cargo** - Comes with Rust
- **Git** - For cloning examples

Check your Rust version:
```bash
rustc --version
# Should be 1.70.0 or higher
```

---

## Installation Methods

### Method 1: Add to Existing Project (Recommended)

Add Firework to your `Cargo.toml`:

```toml
[dependencies]
firework = { git = "https://github.com/your-org/firework" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**With optional features:**

```toml
[dependencies]
firework = { git = "https://github.com/your-org/firework", features = ["hot-reload"] }
```

### Method 2: Using the CLI Tool

Install the Firework CLI globally:

```bash
git clone https://github.com/your-org/firework
cd firework
cargo install --path firework-cli
```

Or use the install script:

```bash
./install_fwk.sh
```

Verify installation:

```bash
fwk --version
```

### Method 3: From Source

Clone and build from source:

```bash
git clone https://github.com/your-org/firework
cd firework
cargo build --release
```

---

## Feature Flags

Firework supports the following feature flags:

| Feature | Description | Default |
|---------|-------------|---------|
| `hot-reload` | Enable hot-reload for development | ❌ |
| `testing` | Enable testing utilities | ❌ |

**Enable features:**

```toml
[dependencies]
firework = { git = "...", features = ["hot-reload", "testing"] }
```

---

## Optional Plugins

Install official plugins as needed:

```toml
[dependencies]
firework = { git = "..." }
firework-auth = { git = "...", path = "plugins/firework-auth" }
firework-seaorm = { git = "...", path = "plugins/firework-seaorm" }
firework-vite = { git = "...", path = "plugins/firework-vite" }
```

---

## Development Tools

### Hot Reload Binary

For development with auto-reload:

```bash
cargo install --path . --features hot-reload --bin firework-dev
```

Usage:
```bash
firework-dev                    # Watch main app
firework-dev my_example         # Watch specific example
```

---

## Verification

Create a simple test to verify installation:

**`src/main.rs`:**
```rust
use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Firework is working! 🔥"
}

#[tokio::main]
async fn main() {
    routes!().listen("127.0.0.1:8080").await.unwrap();
}
```

Run it:
```bash
cargo run
```

Visit http://127.0.0.1:8080 - you should see the message!

---

## Troubleshooting

### Issue: Compilation errors with linkme

**Solution:** Update Rust to latest version:
```bash
rustup update stable
```

### Issue: "could not find tokio"

**Solution:** Add tokio to dependencies:
```toml
tokio = { version = "1", features = ["full"] }
```

### Issue: Slow compilation

**Solution:** Use `cargo build --release` for optimized builds, or enable incremental compilation:

```toml
[profile.dev]
incremental = true
```

---

## Next Steps

✅ Firework is installed!

Now proceed to:
- [Quick Start Guide](./quickstart.md) - Build your first app
- [Project Structure](./project-structure.md) - Understand the layout
- [Core Concepts](../core/routing.md) - Learn the fundamentals

---

## System Requirements

**Minimum:**
- 2 CPU cores
- 1 GB RAM
- 100 MB disk space

**Recommended:**
- 4+ CPU cores
- 4 GB RAM
- SSD storage

**Operating Systems:**
- ✅ Linux (tested on Ubuntu 20.04+)
- ✅ macOS (tested on 11+)
- ✅ Windows (with WSL2 recommended)

---

## Need Help?

- 📖 [Documentation](../README.md)
- 💬 [Discord Community](https://discord.gg/firework)
- 🐛 [Report Issues](https://github.com/your-org/firework/issues)
