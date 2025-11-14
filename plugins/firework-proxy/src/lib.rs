use firework::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancing {
    RoundRobin,
    LeastConnections,
    IpHash,
    Random,
}

/// Backend configuration
#[derive(Debug, Clone)]
pub struct Backend {
    pub url: String,
    pub weight: u32,
}

impl Backend {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            weight: 1,
        }
    }
}

/// Proxy route
pub struct ProxyRoute {
    pub pattern: Regex,
}

impl ProxyRoute {
    pub fn new(pattern: &str) -> Result<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| Error::BadRequest(format!("Invalid pattern: {}", e)))?;
        Ok(Self { pattern: regex })
    }

    pub fn matches(&self, path: &str) -> bool {
        self.pattern.is_match(path)
    }
}

/// Proxy router
pub struct ProxyRouter {
    routes: Vec<ProxyRoute>,
}

impl ProxyRouter {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn route(mut self, route: ProxyRoute) -> Self {
        self.routes.push(route);
        self
    }
}

impl Default for ProxyRouter {
    fn default() -> Self {
        Self::new()
    }
}
