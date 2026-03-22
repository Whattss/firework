use std::collections::HashMap;
use std::sync::Arc;
use ahash::AHashMap;

use crate::AsyncHandler;
use crate::Method;

type HandlerBox = Arc<dyn AsyncHandler>;

/// Segment of a route pattern
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    /// Static path segment (e.g., "users", "api")
    Static(String),
    /// Dynamic parameter (e.g., ":id", ":name")
    Param { name: String },
}

/// Route pattern with segments and precomputed hash
#[derive(Debug, Clone)]
pub struct RoutePattern {
    /// Original path string
    pub path: String,
    /// Parsed segments
    pub segments: Vec<Segment>,
    /// Number of static segments (for specificity sorting)
    pub specificity: usize,
}

impl RoutePattern {
    /// Parse a path into a RoutePattern
    pub fn parse(path: &str) -> Self {
        let segments: Vec<Segment> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|part| {
                if part.starts_with(':') {
                    Segment::Param {
                        name: part[1..].to_string(),
                    }
                } else {
                    Segment::Static(part.to_string())
                }
            })
            .collect();

        let specificity = segments
            .iter()
            .filter(|s| matches!(s, Segment::Static(_)))
            .count();

        RoutePattern {
            path: path.to_string(),
            segments,
            specificity,
        }
    }

    /// Check if this pattern matches the given path parts
    /// Returns Some(params) if match, None otherwise
    pub fn matches(&self, parts: &[&str]) -> Option<AHashMap<String, String>> {
        if parts.len() != self.segments.len() {
            return None;
        }

        let mut params = AHashMap::new();

        for (segment, part) in self.segments.iter().zip(parts.iter()) {
            match segment {
                Segment::Static(expected) => {
                    if expected != part {
                        return None;
                    }
                }
                Segment::Param { name } => {
                    params.insert(name.clone(), part.to_string());
                }
            }
        }

        Some(params)
    }

    /// Check if path is fully static (no parameters)
    pub fn is_static(&self) -> bool {
        self.segments
            .iter()
            .all(|s| matches!(s, Segment::Static(_)))
    }
}

/// Route entry for static routes (no parameters)
struct StaticRoute {
    method: String,
    path: String,
    handler: HandlerBox,
}

struct StaticPerfectTable {
    seed: u64,
    slots: Vec<Option<usize>>,
}

/// Route entry for parameterized routes
struct ParamRoute {
    pattern: RoutePattern,
    methods: HashMap<String, HandlerBox>,
}

/// Perfect hash router - O(1) for static routes, O(n) for param routes
pub struct PerfectHashRouter {
    /// Static routes stored in a collision-free table.
    static_routes: Vec<StaticRoute>,
    static_table: Option<StaticPerfectTable>,

    /// Parameterized routes sorted by specificity (most specific first)
    /// O(n) search but n is typically small and ordered by likelihood
    param_routes: Vec<ParamRoute>,

    /// Total route count
    route_count: usize,
}

impl PerfectHashRouter {
    pub fn new() -> Self {
        PerfectHashRouter {
            static_routes: Vec::new(),
            static_table: None,
            param_routes: Vec::new(),
            route_count: 0,
        }
    }

    /// Add a route to the router
    pub fn add_route(&mut self, method: &str, path: &str, handler: Box<dyn AsyncHandler>) {
        let normalized_path = normalize_path(path);
        let pattern = RoutePattern::parse(&normalized_path);
        let handler_arc = Arc::from(handler);
        let method_upper = method.to_uppercase();
        let mut is_new_mapping = false;

        if pattern.is_static() {
            // Static route → perfect hash
            is_new_mapping = self.insert_static_route(method_upper, pattern.path.clone(), handler_arc);
        } else {
            // Parameterized route → add to param_routes
            // Exact same path updates existing entry (method overwrite semantics).
            if let Some(existing) = self
                .param_routes
                .iter_mut()
                .find(|r| r.pattern.path == pattern.path)
            {
                if existing.methods.insert(method_upper, handler_arc).is_none() {
                    is_new_mapping = true;
                }
            } else {
                if let Some(conflict) = self
                    .param_routes
                    .iter()
                    .find(|r| same_route_shape(&r.pattern, &pattern) && r.methods.contains_key(&method_upper))
                {
                    eprintln!(
                        "[ROUTER] Ambiguous param route shape detected for method {}: '{}' and '{}'. \
Resolution is deterministic (specificity desc, path lexical asc).",
                        method_upper, conflict.pattern.path, pattern.path
                    );
                }

                let mut methods = HashMap::new();
                methods.insert(method_upper, handler_arc);

                self.param_routes.push(ParamRoute { pattern, methods });

                self.sort_param_routes();
                is_new_mapping = true;
            }
        }

        if is_new_mapping {
            self.route_count += 1;
        }
    }

    /// Add a route coming from compile-time metadata (linkme distributed slice).
    pub fn add_route_info(&mut self, route: &crate::RouteInfo) {
        let method_upper = route.method.to_uppercase();
        let normalized_path = normalize_path(route.path);
        let actual_is_static = is_static_path_runtime(&normalized_path);

        if actual_is_static {
            let expected_hash = Self::hash_static_route(&method_upper, &normalized_path);
            let hash_matches_metadata = route.precomputed_hash != 0
                && route.precomputed_hash == expected_hash
                && route.is_static_path
                && normalized_path == route.path;
            if !hash_matches_metadata && route.precomputed_hash != 0 {
                eprintln!(
                    "[ROUTER] Static route metadata mismatch for {} {}. Falling back to runtime canonical hash.",
                    route.method, route.path
                );
            }

            let handler = Arc::from(Box::new(route.handler) as Box<dyn AsyncHandler>);
            if self.insert_static_route(method_upper, normalized_path, handler) {
                self.route_count += 1;
            }
        } else {
            // Metadata can lie (or be stale). Dynamic routes are always rebuilt from path shape.
            self.add_route(
                route.method,
                route.path,
                Box::new(route.handler) as Box<dyn AsyncHandler>,
            );
        }
    }

    /// Find a route handler for the given method and path
    pub fn find(
        &self,
        method: &Method,
        path: &str,
    ) -> Option<(HandlerBox, AHashMap<String, String>)> {
        let method_str = method_to_str(method);
        let normalized_path = normalize_path(path);

        // If a generated PHF table exists, use it to short-circuit static misses.
        let has_phf_map = crate::phf_routes::has_static_route_map();
        let phf_has_route = crate::phf_routes::static_route_path(method_str, &normalized_path).is_some();
        if !has_phf_map || phf_has_route {
            // Try static routes first (O(1) on perfect table)
            if let Some(table) = &self.static_table {
                let slot = static_slot(method_str, &normalized_path, table.seed, table.slots.len());
                if let Some(index) = table.slots[slot] {
                    if let Some(route) = self.static_routes.get(index) {
                        if route.method == method_str && route.path == normalized_path {
                            return Some((Arc::clone(&route.handler), AHashMap::new()));
                        }
                    }
                }
            } else {
                for route in &self.static_routes {
                    if route.method == method_str && route.path == normalized_path {
                        return Some((Arc::clone(&route.handler), AHashMap::new()));
                    }
                }
            }
        }

        // Try parameterized routes (O(n), but sorted by specificity)
        let parts: Vec<&str> = normalized_path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let parts_len = parts.len();

        for param_route in &self.param_routes {
            if param_route.pattern.segments.len() != parts_len {
                continue;
            }
            if let Some(params) = param_route.pattern.matches(&parts) {
                if let Some(handler) = param_route.methods.get(method_str) {
                    return Some((Arc::clone(handler), params));
                }
            }
        }

        None
    }

    /// Compute perfect hash for static route
    #[inline]
    fn hash_static_route(method: &str, path: &str) -> u64 {
        hash_route_key(method, path)
    }

    /// Get statistics about the router
    pub fn stats(&self) -> RouterStats {
        RouterStats {
            total_routes: self.route_count,
            static_routes: self.static_routes.len(),
            param_routes: self.param_routes.len(),
        }
    }

    fn insert_static_route(
        &mut self,
        method_upper: String,
        normalized_path: String,
        handler_arc: HandlerBox,
    ) -> bool {
        if let Some(route) = self
            .static_routes
            .iter_mut()
            .find(|r| r.method == method_upper && r.path == normalized_path)
        {
            route.handler = handler_arc;
            false
        } else {
            self.static_routes.push(StaticRoute {
                method: method_upper,
                path: normalized_path,
                handler: handler_arc,
            });
            self.rebuild_static_table();
            true
        }
    }

    fn rebuild_static_table(&mut self) {
        if self.static_routes.is_empty() {
            self.static_table = None;
            return;
        }

        let mut table_size = self.static_routes.len().next_power_of_two().max(1);
        for _ in 0..10 {
            if let Some(table) = Self::build_static_table(&self.static_routes, table_size, 4096) {
                self.static_table = Some(table);
                return;
            }
            table_size <<= 1;
        }

        self.static_table = None;
        eprintln!("[ROUTER] Could not build perfect table for static routes, using linear fallback.");
    }

    fn build_static_table(
        routes: &[StaticRoute],
        table_size: usize,
        max_seed_attempts: u64,
    ) -> Option<StaticPerfectTable> {
        for seed in 0..max_seed_attempts {
            let mut slots = vec![None; table_size];
            let mut collision = false;

            for (idx, route) in routes.iter().enumerate() {
                let slot = static_slot(&route.method, &route.path, seed, table_size);
                if slots[slot].is_some() {
                    collision = true;
                    break;
                }
                slots[slot] = Some(idx);
            }

            if !collision {
                return Some(StaticPerfectTable { seed, slots });
            }
        }

        None
    }

    fn sort_param_routes(&mut self) {
        self.param_routes.sort_by(|a, b| {
            b.pattern
                .specificity
                .cmp(&a.pattern.specificity)
                .then_with(|| a.pattern.path.cmp(&b.pattern.path))
        });
    }
}

impl Default for PerfectHashRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Router statistics
#[derive(Debug, Clone)]
pub struct RouterStats {
    pub total_routes: usize,
    pub static_routes: usize,
    pub param_routes: usize,
}

#[inline]
fn method_to_str(method: &Method) -> &'static str {
    match method {
        Method::GET => "GET",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::DELETE => "DELETE",
        Method::HEAD => "HEAD",
        Method::OPTIONS => "OPTIONS",
        Method::PATCH => "PATCH",
        Method::Unknown(_) => "UNKNOWN",
    }
}

fn normalize_path(path: &str) -> String {
    let normalized_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if normalized_segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", normalized_segments.join("/"))
    }
}

#[inline]
fn is_static_path_runtime(path: &str) -> bool {
    !path.as_bytes().contains(&b':')
}

#[inline]
fn hash_route_key(method: &str, path: &str) -> u64 {
    hash_route_key_with_seed(method, path, 0)
}

#[inline]
fn hash_route_key_with_seed(method: &str, path: &str, seed: u64) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET ^ seed;
    for b in method.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash ^= b'|' as u64;
    hash = hash.wrapping_mul(FNV_PRIME);
    for b in path.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[inline]
fn static_slot(method: &str, path: &str, seed: u64, table_len: usize) -> usize {
    (hash_route_key_with_seed(method, path, seed) as usize) % table_len
}

fn same_route_shape(a: &RoutePattern, b: &RoutePattern) -> bool {
    if a.segments.len() != b.segments.len() {
        return false;
    }

    for (left, right) in a.segments.iter().zip(b.segments.iter()) {
        match (left, right) {
            (Segment::Static(lhs), Segment::Static(rhs)) if lhs == rhs => {}
            (Segment::Param { .. }, Segment::Param { .. }) => {}
            _ => return false,
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;
    use std::pin::Pin;
    use std::future::Future;

    // Mock handler for testing
    struct MockHandler;
    impl AsyncHandler for MockHandler {
        fn call(
            &self,
            _req: crate::Request,
            _res: crate::Response,
        ) -> Pin<Box<dyn Future<Output = crate::Response> + Send>> {
            Box::pin(async { Response::new(crate::StatusCode::Ok, vec![]) })
        }
    }

    #[test]
    fn test_route_pattern_parse() {
        let pattern = RoutePattern::parse("/users/:id/posts/:pid");
        assert_eq!(pattern.segments.len(), 4);
        assert_eq!(pattern.specificity, 2); // "users" and "posts"
        assert!(!pattern.is_static());
    }

    #[test]
    fn test_route_pattern_static() {
        let pattern = RoutePattern::parse("/api/health");
        assert_eq!(pattern.segments.len(), 2);
        assert_eq!(pattern.specificity, 2);
        assert!(pattern.is_static());
    }

    #[test]
    fn test_pattern_matching() {
        let pattern = RoutePattern::parse("/users/:id/posts");
        let parts = vec!["users", "123", "posts"];
        let params = pattern.matches(&parts).unwrap();

        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_pattern_no_match() {
        let pattern = RoutePattern::parse("/users/:id/posts");
        let parts = vec!["users", "123"]; // Too few segments
        assert!(pattern.matches(&parts).is_none());
    }

    #[test]
    fn test_static_route_lookup() {
        let mut router = PerfectHashRouter::new();
        router.add_route("GET", "/api/health", Box::new(MockHandler));

        let result = router.find(&Method::GET, "/api/health");
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert!(params.is_empty()); // No params for static route
    }

    #[test]
    fn test_param_route_lookup() {
        let mut router = PerfectHashRouter::new();
        router.add_route("GET", "/users/:id", Box::new(MockHandler));

        let result = router.find(&Method::GET, "/users/123");
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_multiple_params() {
        let mut router = PerfectHashRouter::new();
        router.add_route("GET", "/users/:id/posts/:pid", Box::new(MockHandler));

        let result = router.find(&Method::GET, "/users/42/posts/99");
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert_eq!(params.get("id"), Some(&"42".to_string()));
        assert_eq!(params.get("pid"), Some(&"99".to_string()));
    }

    #[test]
    fn test_specificity_ordering() {
        let mut router = PerfectHashRouter::new();

        // Add less specific first
        router.add_route("GET", "/users/:id", Box::new(MockHandler));
        // Add more specific second
        router.add_route("GET", "/users/:id/posts/:pid", Box::new(MockHandler));

        // More specific should be checked first
        assert_eq!(router.param_routes[0].pattern.specificity, 2);
        assert_eq!(router.param_routes[1].pattern.specificity, 1);
    }

    #[test]
    fn test_method_not_found() {
        let mut router = PerfectHashRouter::new();
        router.add_route("GET", "/users", Box::new(MockHandler));

        let result = router.find(&Method::POST, "/users");
        assert!(result.is_none()); // POST not registered
    }

    #[test]
    fn test_router_stats() {
        let mut router = PerfectHashRouter::new();
        router.add_route("GET", "/static", Box::new(MockHandler));
        router.add_route("GET", "/users/:id", Box::new(MockHandler));

        let stats = router.stats();
        assert_eq!(stats.total_routes, 2);
        assert_eq!(stats.static_routes, 1);
        assert_eq!(stats.param_routes, 1);
    }

    #[test]
    fn test_route_shape_prefers_deterministic_lexical_winner() {
        let mut router = PerfectHashRouter::new();
        router.add_route("GET", "/users/:user_id", Box::new(MockHandler));
        router.add_route("GET", "/users/:id", Box::new(MockHandler));

        let stats = router.stats();
        assert_eq!(stats.total_routes, 2);
        assert_eq!(stats.param_routes, 2);

        let (_, params) = router.find(&Method::GET, "/users/42").expect("route must match");
        // Lexical order decides winner consistently: "/users/:id" < "/users/:user_id"
        assert_eq!(params.get("id"), Some(&"42".to_string()));
        assert!(params.get("user_id").is_none());
    }

    #[test]
    fn test_route_shape_ambiguity_resolution_is_deterministic() {
        let mut router_a = PerfectHashRouter::new();
        router_a.add_route("GET", "/users/:id", Box::new(MockHandler));
        router_a.add_route("GET", "/users/:user_id", Box::new(MockHandler));

        let mut router_b = PerfectHashRouter::new();
        router_b.add_route("GET", "/users/:user_id", Box::new(MockHandler));
        router_b.add_route("GET", "/users/:id", Box::new(MockHandler));

        let (_, params_a) = router_a.find(&Method::GET, "/users/7").expect("route must match");
        let (_, params_b) = router_b.find(&Method::GET, "/users/7").expect("route must match");

        // Lexical order decides winner consistently: "/users/:id" < "/users/:user_id"
        assert_eq!(params_a.get("id"), Some(&"7".to_string()));
        assert_eq!(params_b.get("id"), Some(&"7".to_string()));
    }

    #[test]
    fn test_path_normalization_trailing_slash() {
        let mut router = PerfectHashRouter::new();
        router.add_route("GET", "/health", Box::new(MockHandler));

        assert!(router.find(&Method::GET, "/health").is_some());
        assert!(router.find(&Method::GET, "/health/").is_some());
        assert!(router.find(&Method::GET, "health").is_some());
    }

    #[test]
    fn test_path_normalization_on_insert() {
        let mut router = PerfectHashRouter::new();
        router.add_route("GET", "health", Box::new(MockHandler));

        assert!(router.find(&Method::GET, "/health").is_some());
        assert!(router.find(&Method::GET, "health").is_some());
    }

    fn route_info_handler(
        _req: crate::Request,
        _res: crate::Response,
    ) -> Pin<Box<dyn Future<Output = crate::Response> + Send>> {
        Box::pin(async { Response::new(crate::StatusCode::Ok, vec![]) })
    }

    #[test]
    fn test_route_info_ignores_bad_static_metadata_for_param_path() {
        let mut router = PerfectHashRouter::new();
        let bad = crate::RouteInfo {
            method: "GET",
            path: "/users/:id",
            handler: route_info_handler,
            precomputed_hash: 42,
            is_static_path: true,
        };

        router.add_route_info(&bad);
        let (_, params) = router.find(&Method::GET, "/users/99").expect("must match");
        assert_eq!(params.get("id"), Some(&"99".to_string()));

        let stats = router.stats();
        assert_eq!(stats.static_routes, 0);
        assert_eq!(stats.param_routes, 1);
    }

    #[test]
    fn test_route_info_ignores_incorrect_precomputed_hash() {
        let mut router = PerfectHashRouter::new();
        let bad = crate::RouteInfo {
            method: "GET",
            path: "/health",
            handler: route_info_handler,
            precomputed_hash: 12345, // intentionally wrong
            is_static_path: true,
        };

        router.add_route_info(&bad);
        assert!(router.find(&Method::GET, "/health").is_some());
        assert!(router.find(&Method::GET, "/health/").is_some());
    }

    #[test]
    fn test_route_info_ignores_non_canonical_static_hash() {
        let mut router = PerfectHashRouter::new();
        let bad = crate::RouteInfo {
            method: "GET",
            path: "/health/",
            handler: route_info_handler,
            precomputed_hash: hash_route_key("GET", "/health/"),
            is_static_path: true,
        };

        router.add_route_info(&bad);
        assert!(router.find(&Method::GET, "/health").is_some());
        assert!(router.find(&Method::GET, "/health/").is_some());
    }

    #[test]
    fn test_static_table_is_collision_free() {
        let mut router = PerfectHashRouter::new();
        for i in 0..256 {
            router.add_route("GET", &format!("/s/{i}"), Box::new(MockHandler));
        }

        let table = router.static_table.as_ref().expect("perfect table should exist");
        let filled = table.slots.iter().filter(|x| x.is_some()).count();
        assert_eq!(filled, 256);

        for i in 0..256 {
            assert!(router.find(&Method::GET, &format!("/s/{i}")).is_some());
        }
    }
}
