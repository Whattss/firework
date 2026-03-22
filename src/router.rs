use std::sync::Arc;
use ahash::AHashMap;

use crate::perfect_hash_router::{PerfectHashRouter, RouterStats};
use crate::AsyncHandler;
use crate::Method;

type HandlerBox = Arc<dyn AsyncHandler>;

pub struct Router {
    inner: PerfectHashRouter,
}

impl Router {
    pub fn new() -> Self {
        Self {
            inner: PerfectHashRouter::new(),
        }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: Box<dyn AsyncHandler>) {
        self.inner.add_route(method, path, handler);
    }

    pub fn add_route_info(&mut self, route: &crate::RouteInfo) {
        self.inner.add_route_info(route);
    }

    pub fn add_routes_info_sorted(&mut self, routes: &[crate::RouteInfo]) {
        let mut sorted: Vec<&crate::RouteInfo> = routes.iter().collect();
        sorted.sort_by(|a, b| {
            a.method
                .cmp(b.method)
                .then_with(|| a.path.cmp(b.path))
                .then_with(|| a.precomputed_hash.cmp(&b.precomputed_hash))
        });

        for route in sorted {
            self.inner.add_route_info(route);
        }
    }

    pub fn find(&self, method: &Method, path: &str) -> Option<(HandlerBox, AHashMap<String, String>)> {
        self.inner.find(method, path)
    }

    pub fn stats(&self) -> RouterStats {
        self.inner.stats()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
