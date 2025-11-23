//! # Firework Compression Plugin
//! 
//! HTTP response compression middleware with Gzip and Brotli support.
//! 
//! ## Features
//! 
//! - Gzip compression (RFC 1952)
//! - Brotli compression (RFC 7932) - better compression ratio
//! - Auto-detection from Accept-Encoding header
//! - Configurable compression level
//! - Minimum size threshold
//! - Skip already compressed content types
//! - Firework.toml configuration support
//! 
//! ## Quick Start
//! 
//! ```rust
//! use firework::prelude::*;
//! use firework_compress::CompressionPlugin;
//! use std::sync::Arc;
//! 
//! #[tokio::main]
//! async fn main() {
//!     // Auto mode - uses best available compression
//!     firework::register_plugin(Arc::new(CompressionPlugin::auto()));
//!     
//!     routes!().listen("127.0.0.1:8080").await.expect("Failed to start");
//! }
//! ```

use firework::{Plugin, PluginResult, PluginMetadata, Request, Response, ResponseBody};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable gzip compression
    pub gzip: bool,
    
    /// Enable brotli compression (better ratio, slower)
    pub brotli: bool,
    
    /// Minimum response size to compress (in bytes)
    pub min_size: usize,
    
    /// Compression level (0-11 for brotli, 0-9 for gzip)
    pub level: u32,
    
    /// Skip compression for these content types
    pub skip_content_types: Vec<String>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            gzip: true,
            brotli: true,
            min_size: 1024, // 1KB
            level: 6,       // Balanced
            skip_content_types: vec![
                "image/".to_string(),
                "video/".to_string(),
                "audio/".to_string(),
                "application/zip".to_string(),
                "application/gzip".to_string(),
                "application/x-brotli".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Algorithm {
    Brotli,
    Gzip,
}

#[derive(Clone)]
pub struct CompressionPlugin {
    config: CompressionConfig,
}

impl CompressionPlugin {
    /// Create with default config (gzip + brotli enabled)
    pub fn new() -> Self {
        Self {
            config: CompressionConfig::default(),
        }
    }
    
    /// Create from Firework.toml
    /// 
    /// # Example Firework.toml
    /// 
    /// ```toml
    /// [plugins.compression]
    /// gzip = true
    /// brotli = true
    /// min_size = 1024
    /// level = 6
    /// skip_content_types = ["image/", "video/"]
    /// ```
    pub async fn from_config() -> Self {
        let config: CompressionConfig = firework::load_plugin_config_as("compression")
            .await
            .unwrap_or_default();
        Self { config }
    }
    
    /// Auto mode - enable both gzip and brotli
    pub fn auto() -> Self {
        Self::new()
    }
    
    /// Only gzip compression
    pub fn gzip_only() -> Self {
        Self {
            config: CompressionConfig {
                gzip: true,
                brotli: false,
                ..Default::default()
            },
        }
    }
    
    /// Only brotli compression (better ratio)
    pub fn brotli_only() -> Self {
        Self {
            config: CompressionConfig {
                gzip: false,
                brotli: true,
                ..Default::default()
            },
        }
    }
    
    /// Set minimum size threshold
    pub fn min_size(mut self, bytes: usize) -> Self {
        self.config.min_size = bytes;
        self
    }
    
    /// Set compression level (0-11 for brotli, 0-9 for gzip)
    pub fn level(mut self, level: u32) -> Self {
        self.config.level = level;
        self
    }
    
    /// Add content type to skip
    pub fn skip_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.config.skip_content_types.push(content_type.into());
        self
    }
    
    /// Choose best algorithm based on Accept-Encoding
    fn choose_algorithm(&self, accept_encoding: &str) -> Option<Algorithm> {
        // Prefer brotli if available (better compression)
        if self.config.brotli && accept_encoding.contains("br") {
            Some(Algorithm::Brotli)
        } else if self.config.gzip && (accept_encoding.contains("gzip") || accept_encoding.contains("*")) {
            Some(Algorithm::Gzip)
        } else {
            None
        }
    }
    
    /// Check if content type should be skipped
    fn should_skip_content_type(&self, content_type: &str) -> bool {
        self.config.skip_content_types
            .iter()
            .any(|skip| content_type.starts_with(skip))
    }
    
    /// Compress data with gzip
    fn compress_gzip(&self, data: &[u8]) -> Vec<u8> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.config.level));
        encoder.write_all(data).expect("Gzip compression failed");
        encoder.finish().expect("Gzip finish failed")
    }
    
    /// Compress data with brotli
    fn compress_brotli(&self, data: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();
        let params = brotli::enc::BrotliEncoderParams {
            quality: self.config.level as i32,
            ..Default::default()
        };
        
        brotli::BrotliCompress(&mut &data[..], &mut output, &params)
            .expect("Brotli compression failed");
        output
    }
}

impl Default for CompressionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Plugin for CompressionPlugin {
    fn name(&self) -> &'static str {
        "Compression"
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Compression",
            version: "1.0.0",
            author: "Firework Team",
            description: "HTTP response compression (gzip/brotli)",
        }
    }
    
    fn priority(&self) -> i32 {
        80 // Run after CORS and Security, before sending response
    }
    
    async fn on_init(&self) -> PluginResult<()> {
        println!("[Compression] Plugin initialized");
        
        let mut algos = Vec::new();
        if self.config.brotli {
            algos.push("brotli");
        }
        if self.config.gzip {
            algos.push("gzip");
        }
        
        println!("[Compression] Algorithms: {}", algos.join(", "));
        println!("[Compression] Min size: {} bytes", self.config.min_size);
        println!("[Compression] Level: {}", self.config.level);
        
        Ok(())
    }
    
    async fn on_response(&self, req: &Request, res: &mut Response) -> PluginResult<()> {
        // Skip if already compressed
        if res.headers.contains_key("content-encoding") {
            return Ok(());
        }
        
        // Only compress static bodies (not streams)
        let data = match &res.body {
            ResponseBody::Static(data) => data,
            ResponseBody::Stream(_) => return Ok(()), // Can't compress streams
        };
        
        // Skip if too small
        if data.len() < self.config.min_size {
            return Ok(());
        }
        
        // Skip if content type should not be compressed
        if let Some(content_type) = res.headers.get("content-type") {
            if self.should_skip_content_type(content_type) {
                return Ok(());
            }
        }
        
        // Get Accept-Encoding header
        let accept_encoding = req.headers
            .get("accept-encoding")
            .or_else(|| req.headers.get("Accept-Encoding"))
            .and_then(|v| v.first())
            .map(|s| s.as_str())
            .unwrap_or("");
        
        // Choose compression algorithm
        let Some(algorithm) = self.choose_algorithm(accept_encoding) else {
            return Ok(());
        };
        
        // Compress the data
        let compressed = match algorithm {
            Algorithm::Gzip => {
                let compressed = self.compress_gzip(data);
                res.headers.insert("Content-Encoding".to_string(), "gzip".to_string());
                compressed
            }
            Algorithm::Brotli => {
                let compressed = self.compress_brotli(data);
                res.headers.insert("Content-Encoding".to_string(), "br".to_string());
                compressed
            }
        };
        
        // Only use compression if it actually reduced size
        if compressed.len() < data.len() {
            res.body = ResponseBody::Static(compressed);
            res.headers.insert("Content-Length".to_string(), res.body.len().unwrap_or(0).to_string());
            res.headers.insert("Vary".to_string(), "Accept-Encoding".to_string());
        }
        
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Helper to create auto compression plugin
pub fn compress() -> CompressionPlugin {
    CompressionPlugin::auto()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let plugin = CompressionPlugin::default();
        assert!(plugin.config.gzip);
        assert!(plugin.config.brotli);
        assert_eq!(plugin.config.min_size, 1024);
    }
    
    #[test]
    fn test_gzip_only() {
        let plugin = CompressionPlugin::gzip_only();
        assert!(plugin.config.gzip);
        assert!(!plugin.config.brotli);
    }
    
    #[test]
    fn test_brotli_only() {
        let plugin = CompressionPlugin::brotli_only();
        assert!(!plugin.config.gzip);
        assert!(plugin.config.brotli);
    }
    
    #[test]
    fn test_algorithm_selection() {
        let plugin = CompressionPlugin::auto();
        
        // Prefer brotli
        assert_eq!(plugin.choose_algorithm("gzip, br"), Some(Algorithm::Brotli));
        
        // Fallback to gzip
        assert_eq!(plugin.choose_algorithm("gzip"), Some(Algorithm::Gzip));
        
        // None if not supported
        assert_eq!(plugin.choose_algorithm("deflate"), None);
    }
    
    #[test]
    fn test_skip_content_types() {
        let plugin = CompressionPlugin::auto();
        
        assert!(plugin.should_skip_content_type("image/png"));
        assert!(plugin.should_skip_content_type("video/mp4"));
        assert!(!plugin.should_skip_content_type("text/html"));
        assert!(!plugin.should_skip_content_type("application/json"));
    }
    
    #[test]
    fn test_compression() {
        let plugin = CompressionPlugin::auto();
        
        let data = b"Hello World! ".repeat(100); // Compressible data
        
        // Test gzip
        let compressed = plugin.compress_gzip(&data);
        assert!(compressed.len() < data.len());
        
        // Test brotli
        let compressed = plugin.compress_brotli(&data);
        assert!(compressed.len() < data.len());
    }
    
    #[test]
    fn test_builder_pattern() {
        let plugin = CompressionPlugin::new()
            .min_size(2048)
            .level(9)
            .skip_content_type("application/pdf");
        
        assert_eq!(plugin.config.min_size, 2048);
        assert_eq!(plugin.config.level, 9);
        assert!(plugin.config.skip_content_types.contains(&"application/pdf".to_string()));
    }
}
