// 🔥 FIREWORK REVERSE PROXY - PRODUCTION GRADE
// High-performance reverse proxy with Hyper, connection pooling, and circuit breaker

use firework::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

/// Configuration for a proxy target
#[derive(Clone, Debug)]
pub struct ProxyTarget {
    pub path_prefix: String,
    pub backend_url: String,
    pub strip_prefix: bool,
    pub timeout: Duration,
    pub max_connections: usize,
}

impl ProxyTarget {
    pub fn new(path_prefix: &str, backend_url: &str) -> Self {
        Self {
            path_prefix: path_prefix.to_string(),
            backend_url: backend_url.to_string(),
            strip_prefix: false,
            timeout: Duration::from_secs(30),
            max_connections: 100,
        }
    }

    pub fn strip_prefix(mut self) -> Self {
        self.strip_prefix = true;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

/// Circuit breaker for preventing cascade failures
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
    failures: Arc<Mutex<usize>>,
    successes: Arc<Mutex<usize>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_threshold,
            success_threshold: 2,
            timeout,
            failures: Arc::new(Mutex::new(0)),
            successes: Arc::new(Mutex::new(0)),
        }
    }
}

/// Connection pool for reusing HTTP connections
pub struct ConnectionPool {
    semaphore: Arc<Semaphore>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections)),
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(60))),
        }
    }
}

/// Context marker for proxied responses
#[derive(Clone)]
pub struct ProxiedResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl ProxiedResponse {
    pub fn into_response(self) -> Response {
        let mut response = Response::new(self.status, self.body);
        response.headers = self.headers;
        response
    }
}

/// Context marker for proxy failures
#[derive(Clone)]
pub struct ProxyFailed(pub String);

/// Helper to check if request was proxied
pub fn is_proxied(req: &Request) -> bool {
    req.get_context::<ProxiedResponse>().is_some()
}

/// Get proxied response if available
pub fn get_proxied_response(req: &Request) -> Option<Response> {
    req.get_context::<ProxiedResponse>().map(|p| p.into_response())
}

/// Check if proxy failed
pub fn proxy_failed(req: &Request) -> Option<String> {
    req.get_context::<ProxyFailed>().map(|p| p.0.clone())
}
