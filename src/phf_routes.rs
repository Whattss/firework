#[allow(clippy::all)]
#[allow(dead_code)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/firework_phf_routes.rs"));
}

pub fn static_route_path(method: &str, path: &str) -> Option<&'static str> {
    let key = format!("{method}|{path}");
    generated::STATIC_ROUTE_METHOD_PATH
        .as_ref()
        .and_then(|m| m.get(key.as_str()).copied())
}

pub fn has_static_route_map() -> bool {
    generated::STATIC_ROUTE_METHOD_PATH.is_some()
}
