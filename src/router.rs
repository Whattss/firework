use crate::route::{Route, Method};
use crate::request::Request;
use crate::response::Response;
use std::collections::HashMap;
use std::sync::Arc;

/// Nodo del árbol radix que se utiliza para la gestión de rutas.
#[derive(Clone)]
pub struct RadixNode {
    pub segment: String,
    pub handler: Option<Arc<dyn Fn(Request, &mut Response) + Send + Sync>>,
    pub children: HashMap<String, RadixNode>,
    pub route: Option<Route>,
}

impl RadixNode {
    pub fn new(segment: &str) -> Self {
        Self {
            segment: segment.to_string(),
            handler: None,
            children: HashMap::new(),
            route: None,
        }
    }

    /// Inserta una ruta en el árbol, teniendo en cuenta segmentos dinámicos (prefijados con `:`).
    pub fn insert(&mut self, path: &str, method: Method, handler: Arc<dyn Fn(Request, &mut Response) + Send + Sync>) {
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        let mut current = self;

        for (i, part) in parts.iter().enumerate() {
            let is_param = part.starts_with(':');

            current = current.children.entry(part.to_string()).or_insert_with(|| {
                let mut node = RadixNode::new(part);
                if is_param && i == parts.len() - 1 {
                    // Solo asignamos la ruta si es un nodo terminal
                    node.route = Some(Route::new(method.clone(), path.to_string()));
                }
                node
            });
        }

        // Asigna el handler solo en el último nodo
        current.handler = Some(handler);
    }

    /// Busca una ruta en el árbol. Si no encuentra una coincidencia exacta, busca un nodo dinámico.
    pub fn find(&self, path: &str) -> Option<(&Route, &Arc<dyn Fn(Request, &mut Response) + Send + Sync>, HashMap<String, String>)> {
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        let mut current = self;
        let mut params = HashMap::new();

        for part in parts {
            if let Some(next) = current.children.get(part) {
                current = next;
            } else {
                // Busca un nodo dinámico (segmentos que comienzan con `:`)
                if let Some((key, next)) = current.children.iter().find(|(k, _)| k.starts_with(':')) {
                    current = next;
                    let param_name = key.trim_start_matches(':').to_string();
                    params.insert(param_name, part.to_string());
                } else {
                    return None;
                }
            }
        }

        // Verificamos si `current` tiene un `handler` antes de retornar
        if let (Some(route), Some(handler)) = (&current.route, &current.handler) {
            Some((route, handler, params))
        } else {
            None
        }
    }
}
