use std::collections::{HashMap, HashSet};

use crate::{PluginFactory, RouteInfo, ScopeMiddleware, WsRouteInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
struct Diagnostic {
    code: &'static str,
    severity: Severity,
    message: String,
    tip: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuardMode {
    Strict,
    Warn,
    Off,
}

pub fn enforce(
    routes: &[RouteInfo],
    ws_routes: &[WsRouteInfo],
    scope_middlewares: &[ScopeMiddleware],
    plugin_factories: &[PluginFactory],
) -> Result<(), String> {
    let mode = guard_mode();
    if mode == GuardMode::Off {
        eprintln!("[LIGHT GUARD] Impure mode enabled; skipping strict validation.");
        return Ok(());
    }

    let middleware_names: HashSet<&str> = scope_middlewares.iter().map(|m| m.name).collect();
    let mut diagnostics = Vec::new();

    if routes.is_empty() {
        diagnostics.push(Diagnostic {
            code: "LG001",
            severity: Severity::Error,
            message: "no routes were registered".to_string(),
            tip: Some("Define at least one route with #[get], #[post], etc."),
        });
    }

    let mut seen = HashMap::new();
    for route in routes {
        validate_route(route, &middleware_names, &mut diagnostics);
        let key = format!("{} {}", route.method, normalize_path(route.path));
        if let Some(prev) = seen.insert(key.clone(), route.path) {
            diagnostics.push(Diagnostic {
                code: "LG002",
                severity: Severity::Error,
                message: format!(
                    "duplicate route detected for '{}': '{}' conflicts with '{}'",
                    key, prev, route.path
                ),
                tip: Some("Keep a single canonical route per METHOD+PATH."),
            });
        }
    }

    detect_ambiguous_param_shapes(routes, &mut diagnostics);
    detect_param_static_overlaps(routes, &mut diagnostics);
    detect_ws_collisions(ws_routes, &mut diagnostics);
    detect_ws_http_overlaps(routes, ws_routes, &mut diagnostics);
    detect_plugin_factory_issues(plugin_factories, &mut diagnostics);

    let has_errors = diagnostics.iter().any(|d| d.severity == Severity::Error);
    match mode {
        GuardMode::Strict if has_errors => Err(format_light_guard_failure(&diagnostics)),
        GuardMode::Strict => {
            emit_warnings(&diagnostics);
            Ok(())
        }
        GuardMode::Warn => {
            emit_all_as_warnings(&diagnostics);
            Ok(())
        }
        GuardMode::Off => Ok(()),
    }
}

fn validate_route(
    route: &RouteInfo,
    middleware_names: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !is_supported_method(route.method) {
        diagnostics.push(Diagnostic {
            code: "LG003",
            severity: Severity::Error,
            message: format!(
                "route '{}' '{}' uses unsupported HTTP method",
                route.method, route.path
            ),
            tip: Some("Use GET, POST, PUT, PATCH, DELETE, OPTIONS, or HEAD."),
        });
    }

    if !route.path.starts_with('/') {
        diagnostics.push(Diagnostic {
            code: "LG004",
            severity: Severity::Error,
            message: format!(
                "route '{}' '{}' must start with '/'",
                route.method, route.path
            ),
            tip: Some("Prefix all route paths with '/'."),
        });
    }

    if route.path.contains("//") {
        diagnostics.push(Diagnostic {
            code: "LG005",
            severity: Severity::Error,
            message: format!(
                "route '{}' '{}' contains duplicate '/' segments",
                route.method, route.path
            ),
            tip: Some("Normalize and remove duplicated path separators."),
        });
    }

    if route.path.len() > 1 && route.path.ends_with('/') {
        diagnostics.push(Diagnostic {
            code: "LG006",
            severity: Severity::Warning,
            message: format!(
                "route '{}' '{}' should not end with '/' (except root path)",
                route.method, route.path
            ),
            tip: Some("Use canonical paths without trailing '/'."),
        });
    }

    let parsed_static = !route.path.split('/').any(|s| s.starts_with(':'));
    if route.is_static_path != parsed_static {
        diagnostics.push(Diagnostic {
            code: "LG007",
            severity: Severity::Error,
            message: format!(
                "route metadata mismatch on '{}' '{}': is_static_path={} but parsed={}",
                route.method, route.path, route.is_static_path, parsed_static
            ),
            tip: Some("Regenerate route metadata through Firework macros."),
        });
    }

    if route.is_static_path && route.precomputed_hash == 0 {
        diagnostics.push(Diagnostic {
            code: "LG008",
            severity: Severity::Error,
            message: format!(
                "static route '{}' '{}' has zero precomputed hash",
                route.method, route.path
            ),
            tip: Some("Route metadata appears stale; rebuild and ensure macros are used."),
        });
    }

    let mut seen_params = HashSet::new();
    for segment in route.path.split('/') {
        if !segment.starts_with(':') {
            continue;
        }
        if segment.len() <= 1 {
            diagnostics.push(Diagnostic {
                code: "LG009",
                severity: Severity::Error,
                message: format!(
                    "route '{}' '{}' contains empty path parameter",
                    route.method, route.path
                ),
                tip: Some("Use named params like ':id'."),
            });
            continue;
        }
        let param = &segment[1..];
        if !is_valid_param_name(param) {
            diagnostics.push(Diagnostic {
                code: "LG010",
                severity: Severity::Error,
                message: format!(
                    "route '{}' '{}' has invalid parameter name ':{}'",
                    route.method, route.path, param
                ),
                tip: Some("Parameter names must match [A-Za-z_][A-Za-z0-9_]*."),
            });
        }
        if !seen_params.insert(param.to_string()) {
            diagnostics.push(Diagnostic {
                code: "LG011",
                severity: Severity::Warning,
                message: format!(
                    "route '{}' '{}' repeats parameter ':{}'",
                    route.method, route.path, param
                ),
                tip: Some("Repeated param names can shadow values; prefer unique names."),
            });
        }
    }

    if route.path.contains('{') || route.path.contains('}') {
        diagnostics.push(Diagnostic {
            code: "LG012",
            severity: Severity::Error,
            message: format!(
                "route '{}' '{}' uses unsupported '{{}}' path params; use ':param' syntax",
                route.method, route.path
            ),
            tip: Some("Replace '/users/{id}' with '/users/:id'."),
        });
    }

    // Scope sanity signal: middleware registry exists but typo/missing names can be caught early.
    if middleware_names.is_empty() && route.path.contains("/api/") {
        diagnostics.push(Diagnostic {
            code: "LG013",
            severity: Severity::Warning,
            message: format!(
                "no scope middlewares registered while using API-like path '{}'",
                route.path
            ),
            tip: Some("If this is intentional ignore this warning; otherwise add scope middleware."),
        });
    }
}

fn is_valid_param_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
    let mut normalized = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    if normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }
    normalized
}

fn is_impure_mode() -> bool {
    truthy_env("FIREWORK_IMPURE")
}

fn format_light_guard_failure(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::from("Firework refuses to compile due Light Guard violations:\n");
    for diagnostic in diagnostics.iter().filter(|d| d.severity == Severity::Error) {
        out.push_str("  - [");
        out.push_str(diagnostic.code);
        out.push_str("] ");
        out.push_str(&diagnostic.message);
        if let Some(tip) = diagnostic.tip {
            out.push_str(" (tip: ");
            out.push_str(tip);
            out.push(')');
        }
        out.push('\n');
    }
    let warning_count = diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
    if warning_count > 0 {
        out.push('\n');
        out.push_str("Warnings also detected:\n");
        for diagnostic in diagnostics.iter().filter(|d| d.severity == Severity::Warning) {
            out.push_str("  - [");
            out.push_str(diagnostic.code);
            out.push_str("] ");
            out.push_str(&diagnostic.message);
            out.push('\n');
        }
    }
    out.push_str("\nTip: fix these issues, or use `--impure` (FIREWORK_IMPURE=1), or set FIREWORK_LIGHT_GUARD=warn/off.");
    out
}

fn emit_warnings(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics.iter().filter(|d| d.severity == Severity::Warning) {
        eprintln!(
            "[LIGHT GUARD][{}][warning] {}",
            diagnostic.code, diagnostic.message
        );
        if let Some(tip) = diagnostic.tip {
            eprintln!("  tip: {tip}");
        }
    }
}

fn emit_all_as_warnings(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!(
            "[LIGHT GUARD][{}][warning] {}",
            diagnostic.code, diagnostic.message
        );
        if let Some(tip) = diagnostic.tip {
            eprintln!("  tip: {tip}");
        }
    }
}

fn detect_ambiguous_param_shapes(routes: &[RouteInfo], diagnostics: &mut Vec<Diagnostic>) {
    let mut by_method_shape: HashMap<(String, String), String> = HashMap::new();
    for route in routes {
        if route.is_static_path {
            continue;
        }
        let key = (route.method.to_string(), route_shape(route.path));
        if let Some(existing) = by_method_shape.get(&key) {
            if existing != route.path {
                diagnostics.push(Diagnostic {
                    code: "LG101",
                    severity: Severity::Warning,
                    message: format!(
                        "ambiguous param route shape for method '{}': '{}' vs '{}'",
                        route.method, existing, route.path
                    ),
                    tip: Some("Prefer a single canonical param shape per endpoint."),
                });
            }
        } else {
            by_method_shape.insert(key, route.path.to_string());
        }
    }
}

fn detect_param_static_overlaps(routes: &[RouteInfo], diagnostics: &mut Vec<Diagnostic>) {
    let mut static_by_method: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut param_by_method: HashMap<&str, Vec<&str>> = HashMap::new();
    for route in routes {
        if route.is_static_path {
            static_by_method.entry(route.method).or_default().push(route.path);
        } else {
            param_by_method.entry(route.method).or_default().push(route.path);
        }
    }

    for (method, static_paths) in &static_by_method {
        let Some(param_paths) = param_by_method.get(method) else {
            continue;
        };
        for static_path in static_paths {
            for param_path in param_paths {
                if path_matches_param_shape(static_path, param_path) {
                    diagnostics.push(Diagnostic {
                        code: "LG102",
                        severity: Severity::Warning,
                        message: format!(
                            "static route '{}' overlaps dynamic route '{}' for method '{}'",
                            static_path, param_path, method
                        ),
                        tip: Some("This is valid but can be confusing; document precedence explicitly."),
                    });
                }
            }
        }
    }
}

fn detect_ws_collisions(ws_routes: &[WsRouteInfo], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = HashMap::new();
    for route in ws_routes {
        let key = normalize_path(route.path);
        if let Some(prev) = seen.insert(key.clone(), route.path) {
            diagnostics.push(Diagnostic {
                code: "LG201",
                severity: Severity::Error,
                message: format!("duplicate websocket route '{}' conflicts with '{}'", prev, route.path),
                tip: Some("Keep only one websocket handler per path."),
            });
        }
        if !route.path.starts_with('/') {
            diagnostics.push(Diagnostic {
                code: "LG202",
                severity: Severity::Error,
                message: format!("websocket route '{}' must start with '/'", route.path),
                tip: Some("Prefix websocket paths with '/'."),
            });
        }
    }
}

fn detect_ws_http_overlaps(
    routes: &[RouteInfo],
    ws_routes: &[WsRouteInfo],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let http_paths: HashSet<String> = routes.iter().map(|r| normalize_path(r.path)).collect();
    for ws in ws_routes {
        let ws_path = normalize_path(ws.path);
        if http_paths.contains(&ws_path) {
            diagnostics.push(Diagnostic {
                code: "LG203",
                severity: Severity::Warning,
                message: format!(
                    "websocket path '{}' overlaps an HTTP route path; upgrade handling order matters",
                    ws.path
                ),
                tip: Some("Ensure this overlap is intentional and documented."),
            });
        }
    }
}

fn detect_plugin_factory_issues(
    plugin_factories: &[PluginFactory],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = HashSet::new();
    for factory in plugin_factories {
        let name = factory.name.trim();
        if name.is_empty() {
            diagnostics.push(Diagnostic {
                code: "LG301",
                severity: Severity::Error,
                message: "plugin factory has empty name".to_string(),
                tip: Some("Provide a non-empty plugin name in #[plugin(...)] registration."),
            });
            continue;
        }
        if !seen.insert(name.to_string()) {
            diagnostics.push(Diagnostic {
                code: "LG302",
                severity: Severity::Warning,
                message: format!("duplicate plugin factory name '{}'", name),
                tip: Some("Duplicate names can make plugin diagnostics confusing."),
            });
        }
        if name.contains(' ') {
            diagnostics.push(Diagnostic {
                code: "LG303",
                severity: Severity::Warning,
                message: format!("plugin factory name '{}' contains spaces", name),
                tip: Some("Prefer stable identifier-like names for better UX."),
            });
        }
    }
}

fn path_matches_param_shape(static_path: &str, param_path: &str) -> bool {
    let s_parts: Vec<&str> = static_path.split('/').filter(|s| !s.is_empty()).collect();
    let p_parts: Vec<&str> = param_path.split('/').filter(|s| !s.is_empty()).collect();
    if s_parts.len() != p_parts.len() {
        return false;
    }
    for (s, p) in s_parts.iter().zip(p_parts.iter()) {
        if p.starts_with(':') {
            continue;
        }
        if s != p {
            return false;
        }
    }
    true
}

fn route_shape(path: &str) -> String {
    let mut out = String::new();
    for segment in path.split('/') {
        if segment.is_empty() {
            continue;
        }
        out.push('/');
        if segment.starts_with(':') {
            out.push(':');
        } else {
            out.push_str(segment);
        }
    }
    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}

fn is_supported_method(method: &str) -> bool {
    matches!(method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS" | "HEAD")
}

fn truthy_env(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            let s = v.trim().to_ascii_lowercase();
            s == "1" || s == "true" || s == "yes" || s == "on"
        })
        .unwrap_or(false)
}

fn guard_mode() -> GuardMode {
    if is_impure_mode() {
        return GuardMode::Off;
    }
    match std::env::var("FIREWORK_LIGHT_GUARD")
        .unwrap_or_else(|_| "strict".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "off" => GuardMode::Off,
        "warn" => GuardMode::Warn,
        _ => GuardMode::Strict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_shape_normalization() {
        assert_eq!(route_shape("/users/:id"), "/users/:");
        assert_eq!(route_shape("/"), "/");
    }

    #[test]
    fn test_param_shape_match() {
        assert!(path_matches_param_shape("/users/42", "/users/:id"));
        assert!(!path_matches_param_shape("/users/42/posts", "/users/:id"));
    }

    #[test]
    fn test_truthy_env_parser() {
        std::env::set_var("FWK_TEST_TRUTHY", "true");
        assert!(truthy_env("FWK_TEST_TRUTHY"));
        std::env::remove_var("FWK_TEST_TRUTHY");
        assert!(!truthy_env("FWK_TEST_TRUTHY"));
    }
}
