# Firework CLI (`fwk`)

**The official command-line tool for Firework** - Build blazing fast web applications with Rust.

## Installation

```bash
# From source
cd firework-cli
cargo install --path .

# Or add to your project
cargo install fwk
```

## Commands

### 🆕 Create a New Project

```bash
fwk new my-app
fwk new my-api --template api
fwk new my-fullstack --template fullstack
```

**Templates**:
- `basic` (default) - Minimal web server
- `api` - REST API with JSON
- `fullstack` - Full web app with static files

### 🚀 Development Server

```bash
# Start dev server with hot reload
fwk dev

# Or using run command
fwk run dev --hot-reload
```

Auto-reloads on file changes!

### 📋 List Routes

```bash
# List all routes
fwk routes

# Show detailed information
fwk routes --verbose

# Filter routes
fwk routes --filter "api"
fwk routes --filter "upload"
```

Example output:
```
🔍 Scanning for routes...

  ────────────────────────────────────────────────────────────────
  GET     /api/users                               list_users
  GET     /api/users/:id                           get_user
  POST    /api/users                               create_user
  DELETE  /api/users/:id                           delete_user
  ────────────────────────────────────────────────────────────────

✓ 4 routes registered
```

See [FWK_ROUTES.md](./FWK_ROUTES.md) for detailed documentation.

### ⚙️ Configuration

```bash
# Create Firework.toml
fwk create config
```

### 🏗️ Build & Release

```bash
# Build for development
fwk run build

# Build optimized release
fwk run release
```

### 📜 Run Custom Scripts

```bash
# Run script from Firework.toml
fwk run script migrate
fwk run script seed
```

## Complete Command Reference

```bash
fwk [COMMAND] [OPTIONS]

Commands:
  new       Create a new Firework project
  dev       Run in development mode with hot reload
  routes    List all registered routes
  create    Create configuration file
  run       Run various tasks
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Project Structure

After running `fwk new my-app`:

```
my-app/
├── Cargo.toml
├── Firework.toml
└── src/
    └── main.rs
```

## Firework.toml Example

```toml
[project]
name = "my-app"
version = "0.1.0"

[server]
host = "127.0.0.1"
port = 8080

[dev]
hot_reload = true
watch_paths = ["src", "static"]

[scripts]
migrate = "cargo run --bin migrate"
seed = "cargo run --bin seed"
test = "cargo test"
```

## Features

### ✨ Hot Reload

Automatically restarts your server when files change:

```bash
fwk dev
```

Watches:
- `src/**/*.rs` - Rust source files
- `Cargo.toml` - Dependencies
- `static/**/*` - Static assets (if configured)

### 🎨 Colored Output

Beautiful terminal output with:
- Color-coded HTTP methods
- Progress indicators
- Clear error messages
- Helpful tips

### 🔍 Route Discovery

Automatically scans your codebase for routes:
- Detects `#[get]`, `#[post]`, etc. macros
- Shows handler names and locations
- Supports filtering and verbose mode

### ⚡ Fast Builds

Optimized build pipeline:
- Incremental compilation
- Parallel builds
- Smart caching

## Examples

### Create and run a new API

```bash
fwk new my-api --template api
cd my-api
fwk dev
```

### List all authentication routes

```bash
fwk routes --filter auth --verbose
```

### Build for production

```bash
fwk run release
./target/release/my-app
```

### Run database migrations

Add to `Firework.toml`:
```toml
[scripts]
migrate = "sea-orm-cli migrate up"
```

Then run:
```bash
fwk run script migrate
```

## Development

### Building the CLI

```bash
cd firework-cli
cargo build --release
```

### Testing

```bash
cargo test
```

### Installing locally

```bash
cargo install --path .
```

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Roadmap

- [x] Project scaffolding
- [x] Hot reload
- [x] Route listing
- [ ] Code generation (handlers, models)
- [ ] Database migrations
- [ ] Testing helpers
- [ ] Deployment tools
- [ ] Docker support
- [ ] OpenAPI/Swagger generation

## Documentation

- [FWK_ROUTES.md](./FWK_ROUTES.md) - Route listing documentation
- [Firework Docs](../docs/) - Main framework documentation

## License

MIT

## Support

- GitHub Issues: Report bugs and request features
- Discussions: Ask questions and share ideas
- Discord: Join the community (coming soon)

---

**Part of the Firework Framework** 🔥
