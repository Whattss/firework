use crate::{Request, Response, Error, Result};
use std::marker::PhantomData;

/// Trait for extracting data from requests (V2 async-native)
#[async_trait::async_trait]
pub trait FromRequest: Sized {
    async fn from_request(req: &mut Request, res: &mut Response) -> Result<Self>;
}

/// Plugin extractor trait - allows plugins to be extracted in handlers
pub trait PluginExtractor: Send + Sync {
    type Plugin: crate::Plugin + Send + Sync + 'static;
    fn extract(plugin: &Self::Plugin) -> Self;
}

/// Extract wrapper for automatic plugin injection
pub struct Extract<P: PluginExtractor>(pub P);

#[async_trait::async_trait]
impl<P> FromRequest for Extract<P>
where
    P: PluginExtractor + Send + 'static,
{
    async fn from_request(_req: &mut Request, _res: &mut Response) -> Result<Self> {
        let registry = crate::plugin::registry().read().await;
        let plugin = registry.get::<P::Plugin>()
            .ok_or_else(|| Error::Internal("Plugin not registered".into()))?;
        
        let extracted = P::extract(plugin);
        Ok(Extract(extracted))
    }
}

/// Extract JSON body
pub struct Json<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for Json<T>
where
    T: serde::de::DeserializeOwned + Send,
{
    async fn from_request(req: &mut Request, _res: &mut Response) -> Result<Self> {
        let body = std::str::from_utf8(&req.body)
            .map_err(|_| Error::BadRequest("Invalid UTF-8 in body".into()))?;
        
        let value = serde_json::from_str(body)
            .map_err(|e| Error::BadRequest(format!("Failed to parse JSON: {}", e)))?;
        
        Ok(Json(value))
    }
}

/// Extract path parameter
pub struct Path<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for Path<T>
where
    T: std::str::FromStr + Send,
{
    async fn from_request(req: &mut Request, _res: &mut Response) -> Result<Self> {
        // For single parameter, we extract from params map
        // This is a simplified version - in real implementation we'd need
        // to know which param to extract based on the route definition
        let value = req.params.values().next()
            .ok_or_else(|| Error::BadRequest("Missing path parameter".into()))?;
        
        let parsed = value.parse()
            .map_err(|_| Error::BadRequest(format!("Failed to parse path parameter: {}", value)))?;
        
        Ok(Path(parsed))
    }
}

/// Extract query parameters
pub struct Query<T>(pub T);

#[async_trait::async_trait]
impl<T> FromRequest for Query<T>
where
    T: serde::de::DeserializeOwned + Send,
{
    async fn from_request(req: &mut Request, _res: &mut Response) -> Result<Self> {
        let query = req.uri.query.as_ref()
            .ok_or_else(|| Error::BadRequest("Missing query parameters".into()))?;
        
        let value = serde_json::from_value(serde_json::json!(query))
            .map_err(|e| Error::BadRequest(format!("Failed to parse query: {}", e)))?;
        
        Ok(Query(value))
    }
}

/// Extract request body as string
pub struct Body(pub String);

#[async_trait::async_trait]
impl FromRequest for Body {
    async fn from_request(req: &mut Request, _res: &mut Response) -> Result<Self> {
        let body = String::from_utf8(req.body.clone())
            .map_err(|_| Error::BadRequest("Invalid UTF-8 in body".into()))?;
        
        Ok(Body(body))
    }
}

/// Extract header value (placeholder - needs better implementation)
pub struct Header<T> {
    _phantom: PhantomData<T>,
}

/// Trait for converting handler return types to Response
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::new(crate::StatusCode::Ok, self.as_bytes())
            .with_header("Content-Type", "text/plain; charset=utf-8")
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::new(crate::StatusCode::Ok, self.into_bytes())
            .with_header("Content-Type", "text/plain; charset=utf-8")
    }
}

impl<T> IntoResponse for Json<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(body) => {
                Response::new(crate::StatusCode::Ok, body)
                    .with_header("Content-Type", "application/json")
            }
            Err(_) => {
                Response::new(
                    crate::StatusCode::InternalServerError,
                    b"{\"error\":\"Failed to serialize JSON\"}"
                )
                .with_header("Content-Type", "application/json")
            }
        }
    }
}

impl<T> IntoResponse for Result<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
