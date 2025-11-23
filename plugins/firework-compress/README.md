# Firework Compression Plugin

HTTP response compression middleware with Gzip and Brotli support.

## Features

- ✅ **Gzip compression** (RFC 1952) - widely supported
- ✅ **Brotli compression** (RFC 7932) - better compression ratio
- ✅ **Auto-detection** from Accept-Encoding header
- ✅ **Configurable compression level** (0-11)
- ✅ **Minimum size threshold** (don't compress tiny responses)
- ✅ **Smart content-type filtering** (skip images, videos, etc.)
- ✅ **Firework.toml configuration support**

## Quick Start

```rust
use firework::prelude::*;
use firework_compress::CompressionPlugin;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Auto mode - uses best available compression
    firework::register_plugin(Arc::new(CompressionPlugin::auto()));
    
    routes!().listen("127.0.0.1:8080").await.expect("Failed to start");
}
```

## Configuration

### From Code

```rust
use firework_compress::CompressionPlugin;

// Auto (both gzip + brotli)
let compress = CompressionPlugin::auto();

// Gzip only
let compress = CompressionPlugin::gzip_only();

// Brotli only (better ratio)
let compress = CompressionPlugin::brotli_only();

// Custom config
let compress = CompressionPlugin::new()
    .min_size(2048)      // Only compress > 2KB
    .level(9)            // Max compression
    .skip_content_type("application/pdf");
```

### From Firework.toml

```toml
[plugins.compression]
gzip = true
brotli = true
min_size = 1024     # 1KB minimum
level = 6           # Balanced (0-11)
skip_content_types = ["image/", "video/", "audio/"]
```

```rust
let compress = CompressionPlugin::from_config().await;
firework::register_plugin(Arc::new(compress));
```

## Compression Levels

- **0-3:** Fast, lower compression
- **4-6:** Balanced (default: 6)
- **7-9:** Slower, better compression
- **10-11:** Maximum compression (brotli only, very slow)

## Performance

### Compression Ratios (typical)

| Content Type | Gzip | Brotli |
|--------------|------|--------|
| HTML         | 70%  | 75%    |
| JSON         | 80%  | 85%    |
| JavaScript   | 65%  | 70%    |
| CSS          | 75%  | 80%    |
| Plain Text   | 60%  | 65%    |

### Bandwidth Savings

For a 100KB HTML response:
- **Uncompressed:** 100KB
- **Gzip:** ~30KB (70% reduction)
- **Brotli:** ~25KB (75% reduction)

## Content Types

### Compressed by Default

- `text/html`
- `text/css`
- `text/javascript`
- `application/javascript`
- `application/json`
- `text/plain`
- `text/xml`
- `application/xml`

### Skipped by Default

- `image/*` (already compressed)
- `video/*` (already compressed)
- `audio/*` (already compressed)
- `application/zip`
- `application/gzip`

## Examples

### Basic Usage

```rust
use firework::prelude::*;
use firework_compress::CompressionPlugin;
use std::sync::Arc;

#[get("/")]
async fn index() -> Response {
    // Large HTML will be auto-compressed
    html!("<html>...</html>")
}

#[tokio::main]
async fn main() {
    firework::register_plugin(Arc::new(CompressionPlugin::auto()));
    routes!().listen("127.0.0.1:8080").await.unwrap();
}
```

### Custom Configuration

```rust
let compress = CompressionPlugin::new()
    .min_size(500)           // Compress anything > 500 bytes
    .level(8)                // High compression
    .skip_content_type("text/csv");  // Don't compress CSV
    
firework::register_plugin(Arc::new(compress));
```

### API with Large JSON Responses

```rust
#[get("/api/data")]
async fn get_data() -> Response {
    let data = vec![/* large dataset */];
    json!(data)  // Will be compressed automatically
}
```

## How It Works

1. **Request arrives** with `Accept-Encoding: gzip, br` header
2. **Plugin checks:**
   - Response size > min_size?
   - Content-Type compressible?
   - Not already compressed?
3. **Choose algorithm:**
   - Prefer Brotli (better ratio)
   - Fallback to Gzip (more compatible)
4. **Compress response**
5. **Set headers:**
   - `Content-Encoding: br` or `gzip`
   - `Vary: Accept-Encoding`
   - Update `Content-Length`

## Browser Support

| Browser | Gzip | Brotli |
|---------|------|--------|
| Chrome  | ✅   | ✅     |
| Firefox | ✅   | ✅     |
| Safari  | ✅   | ✅     |
| Edge    | ✅   | ✅     |
| IE 11   | ✅   | ❌     |

## Production Tips

### 1. Enable Both Algorithms

```rust
CompressionPlugin::auto()  // Enables gzip + brotli
```

Brotli for modern browsers, Gzip as fallback.

### 2. Set Appropriate Level

```rust
.level(6)  // Balanced - good for production
.level(9)  // Maximum - use for static assets
```

### 3. Configure Min Size

```rust
.min_size(1024)  // Don't waste CPU on tiny responses
```

### 4. Skip Already Compressed

The plugin automatically skips:
- Images (JPEG, PNG, WebP)
- Videos (MP4, WebM)
- Archives (ZIP, GZIP)

### 5. Monitor Compression Ratio

```bash
# Before compression
curl -H "Accept-Encoding: identity" http://localhost:8080/api/data

# With gzip
curl -H "Accept-Encoding: gzip" http://localhost:8080/api/data

# With brotli
curl -H "Accept-Encoding: br" http://localhost:8080/api/data
```

## Troubleshooting

### Response not compressed?

Check:
1. Response size > `min_size`?
2. Client sent `Accept-Encoding` header?
3. Content-Type not in skip list?
4. Not already compressed?

### Content-Encoding header present?

```bash
curl -I -H "Accept-Encoding: gzip" http://localhost:8080
```

Should see:
```
Content-Encoding: gzip
Vary: Accept-Encoding
```

## Benchmarks

On a 100KB JSON response:

| Config | Size | Time |
|--------|------|------|
| No compression | 100KB | 1ms |
| Gzip level 1 | 35KB | 2ms |
| Gzip level 6 | 30KB | 5ms |
| Gzip level 9 | 28KB | 12ms |
| Brotli level 6 | 25KB | 8ms |
| Brotli level 11 | 22KB | 50ms |

**Recommendation:** Level 6 offers best speed/size balance.

## License

MIT
