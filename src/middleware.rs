// src/middleware.rs
use crate::request::Request;
use crate::response::Response;
use std::sync::Arc;

/// Trait para definir middleware.
pub trait MiddlewareHandler: Fn(&mut Request, &mut Response) + Send + Sync + 'static {}
impl<T: Fn(&mut Request, &mut Response) + Send + Sync + 'static> MiddlewareHandler for T {}

/// Wrapper clonable para un middleware.
#[derive(Clone)]
pub struct MiddlewareCloneWrapper(pub Arc<dyn MiddlewareHandler>);

impl MiddlewareCloneWrapper {
    pub fn new<F>(f: F) -> Self
    where
        F: MiddlewareHandler + 'static,  // Se eliminó el bound Clone
    {
        MiddlewareCloneWrapper(Arc::new(f))
    }
}
