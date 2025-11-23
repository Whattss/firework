//! # Firework Security Headers Plugin
//! 
//! Adds essential security headers to protect against common web vulnerabilities.
//! 
//! ## Features
//! 
//! - X-Frame-Options (clickjacking protection)
//! - X-Content-Type-Options (MIME sniffing protection)
//! - X-XSS-Protection (XSS protection)
//! - Strict-Transport-Security (HSTS)
//! - Content-Security-Policy (CSP)
//! - Referrer-Policy
//! - Permissions-Policy
//! 
//! ## Quick Start
//! 
//! ```rust
//! use firework::prelude::*;
//! use firework_security::SecurityHeadersPlugin;
//! use std::sync::Arc;
//! 
//! #[tokio::main]
//! async fn main() {
//!     // Use defaults (recommended)
//!     firework::register_plugin(Arc::new(SecurityHeadersPlugin::default()));
//!     
//!     routes!().listen("127.0.0.1:8080").await.expect("Failed to start");
//! }
//! ```

use firework::{Plugin, PluginResult, PluginMetadata, Request, Response};
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub frame_options: Option<String>,
    pub content_type_nosniff: bool,
    pub xss_protection: bool,
    pub hsts_max_age: Option<u64>,
    pub csp: Option<String>,
    pub referrer_policy: Option<String>,
    pub permissions_policy: Option<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            frame_options: Some("DENY".to_string()),
            content_type_nosniff: true,
            xss_protection: true,
            hsts_max_age: Some(31536000),
            csp: Some("default-src 'self'".to_string()),
            referrer_policy: Some("no-referrer".to_string()),
            permissions_policy: None,
        }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersPlugin {
    config: SecurityConfig,
}

impl SecurityHeadersPlugin {
    pub fn new() -> Self {
        Self { config: SecurityConfig::default() }
    }
    
    pub async fn from_config() -> Self {
        let config: SecurityConfig = firework::load_plugin_config_as("security")
            .await
            .unwrap_or_default();
        Self { config }
    }
    
    pub fn with_config(config: SecurityConfig) -> Self {
        Self { config }
    }
    
    pub fn strict() -> Self {
        Self {
            config: SecurityConfig {
                frame_options: Some("DENY".to_string()),
                content_type_nosniff: true,
                xss_protection: true,
                hsts_max_age: Some(63072000),
                csp: Some("default-src 'none'; script-src 'self'; connect-src 'self'; img-src 'self'; style-src 'self'".to_string()),
                referrer_policy: Some("no-referrer".to_string()),
                permissions_policy: Some("geolocation=(), microphone=(), camera=()".to_string()),
            },
        }
    }
    
    pub fn relaxed() -> Self {
        Self {
            config: SecurityConfig {
                frame_options: Some("SAMEORIGIN".to_string()),
                content_type_nosniff: true,
                xss_protection: true,
                hsts_max_age: None,
                csp: Some("default-src 'self' 'unsafe-inline' 'unsafe-eval'".to_string()),
                referrer_policy: Some("no-referrer-when-downgrade".to_string()),
                permissions_policy: None,
            },
        }
    }
    
    pub fn frame_options(mut self, value: impl Into<String>) -> Self {
        self.config.frame_options = Some(value.into());
        self
    }
    
    pub fn no_frame_options(mut self) -> Self {
        self.config.frame_options = None;
        self
    }
    
    pub fn content_type_nosniff(mut self, enable: bool) -> Self {
        self.config.content_type_nosniff = enable;
        self
    }
    
    pub fn xss_protection(mut self, enable: bool) -> Self {
        self.config.xss_protection = enable;
        self
    }
    
    pub fn hsts(mut self, max_age_seconds: u64) -> Self {
        self.config.hsts_max_age = Some(max_age_seconds);
        self
    }
    
    pub fn no_hsts(mut self) -> Self {
        self.config.hsts_max_age = None;
        self
    }
    
    pub fn csp(mut self, policy: impl Into<String>) -> Self {
        self.config.csp = Some(policy.into());
        self
    }
    
    pub fn no_csp(mut self) -> Self {
        self.config.csp = None;
        self
    }
    
    pub fn referrer_policy(mut self, policy: impl Into<String>) -> Self {
        self.config.referrer_policy = Some(policy.into());
        self
    }
    
    pub fn permissions_policy(mut self, policy: impl Into<String>) -> Self {
        self.config.permissions_policy = Some(policy.into());
        self
    }
}

impl Default for SecurityHeadersPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Plugin for SecurityHeadersPlugin {
    fn name(&self) -> &'static str {
        "SecurityHeaders"
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "SecurityHeaders",
            version: "1.0.0",
            author: "Firework Team",
            description: "Security headers middleware for web protection",
        }
    }
    
    fn priority(&self) -> i32 {
        90
    }
    
    async fn on_init(&self) -> PluginResult<()> {
        println!("[Security] Plugin initialized");
        println!("[Security] Frame Options: {:?}", self.config.frame_options);
        println!("[Security] HSTS: {:?}", self.config.hsts_max_age);
        Ok(())
    }
    
    async fn on_response(&self, _req: &Request, res: &mut Response) -> PluginResult<()> {
        if let Some(ref frame_options) = self.config.frame_options {
            res.headers.insert("X-Frame-Options".to_string(), frame_options.clone());
        }
        
        if self.config.content_type_nosniff {
            res.headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        }
        
        if self.config.xss_protection {
            res.headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        }
        
        if let Some(max_age) = self.config.hsts_max_age {
            res.headers.insert("Strict-Transport-Security".to_string(), format!("max-age={}", max_age));
        }
        
        if let Some(ref csp) = self.config.csp {
            res.headers.insert("Content-Security-Policy".to_string(), csp.clone());
        }
        
        if let Some(ref referrer) = self.config.referrer_policy {
            res.headers.insert("Referrer-Policy".to_string(), referrer.clone());
        }
        
        if let Some(ref permissions) = self.config.permissions_policy {
            res.headers.insert("Permissions-Policy".to_string(), permissions.clone());
        }
        
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn security_headers() -> SecurityHeadersPlugin {
    SecurityHeadersPlugin::default()
}

pub fn strict_security() -> SecurityHeadersPlugin {
    SecurityHeadersPlugin::strict()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let plugin = SecurityHeadersPlugin::default();
        assert!(plugin.config.frame_options.is_some());
        assert!(plugin.config.content_type_nosniff);
    }
    
    #[test]
    fn test_strict_config() {
        let plugin = SecurityHeadersPlugin::strict();
        assert_eq!(plugin.config.frame_options, Some("DENY".to_string()));
    }
    
    #[test]
    fn test_builder_pattern() {
        let plugin = SecurityHeadersPlugin::new()
            .frame_options("SAMEORIGIN")
            .hsts(86400);
        
        assert_eq!(plugin.config.frame_options, Some("SAMEORIGIN".to_string()));
        assert_eq!(plugin.config.hsts_max_age, Some(86400));
    }
}
