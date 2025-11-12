use crate::{Method, Request, Response, Router, Server, Uri, Version, Flow, Middleware};
use std::collections::HashMap;
use std::sync::Arc;

/// Test client for making requests to the application
pub struct TestClient {
    router: Arc<Router>,
    middlewares: Vec<Middleware>,
}

impl TestClient {
    /// Create a new test client from a server instance
    pub fn new(server: Server) -> Self {
        Self {
            router: Arc::new(server.router),
            middlewares: server.middlewares,
        }
    }

    /// Create a GET request
    pub fn get(&self, path: &str) -> TestRequest<'_> {
        TestRequest::new(self, Method::GET, path)
    }

    /// Create a POST request
    pub fn post(&self, path: &str) -> TestRequest<'_> {
        TestRequest::new(self, Method::POST, path)
    }

    /// Create a PUT request
    pub fn put(&self, path: &str) -> TestRequest<'_> {
        TestRequest::new(self, Method::PUT, path)
    }

    /// Create a PATCH request
    pub fn patch(&self, path: &str) -> TestRequest<'_> {
        TestRequest::new(self, Method::PATCH, path)
    }

    /// Create a DELETE request
    pub fn delete(&self, path: &str) -> TestRequest<'_> {
        TestRequest::new(self, Method::DELETE, path)
    }

    /// Execute a request and return the response
    async fn execute(&self, mut request: Request) -> TestResponse {
        let mut response = Response::default();
        let mut stopped = false;

        // Apply middlewares
        for mw in &self.middlewares {
            match mw(request.clone(), response) {
                Flow::Stop(final_res) => {
                    response = final_res;
                    stopped = true;
                    break;
                }
                Flow::Next(r, s) => {
                    request = r;
                    response = s;
                }
            }
        }

        // If middleware stopped, return that response
        if stopped {
            return TestResponse::new(response);
        }

        // Find route handler
        if let Some((handler, params)) = self.router.find(&request.method, &request.uri.path) {
            request.params = params;
            response = handler.call(request, response).await;
            TestResponse::new(response)
        } else {
            TestResponse::new(Response::new(crate::response::StatusCode::NotFound, b"Not Found"))
        }
    }
}

/// Builder for creating test requests
pub struct TestRequest<'a> {
    client: &'a TestClient,
    method: Method,
    path: String,
    headers: HashMap<String, Vec<String>>,
    body: Vec<u8>,
    query: HashMap<String, String>,
}

impl<'a> TestRequest<'a> {
    fn new(client: &'a TestClient, method: Method, path: &str) -> Self {
        Self {
            client,
            method,
            path: path.to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            query: HashMap::new(),
        }
    }

    /// Add a header to the request
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(value.to_string());
        self
    }

    /// Set JSON body (automatically sets Content-Type)
    pub fn json(mut self, json: &str) -> Self {
        self.body = json.as_bytes().to_vec();
        self.headers
            .entry("Content-Type".to_string())
            .or_insert_with(Vec::new)
            .push("application/json".to_string());
        self
    }

    /// Set text body
    pub fn body(mut self, body: &str) -> Self {
        self.body = body.as_bytes().to_vec();
        self
    }

    /// Set raw bytes body
    pub fn bytes(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Add a query parameter
    pub fn query(mut self, key: &str, value: &str) -> Self {
        self.query.insert(key.to_string(), value.to_string());
        self
    }

    /// Execute the request and return the response
    pub async fn send(self) -> TestResponse {
        let uri = Uri::new(&self.path, if self.query.is_empty() { None } else { Some(self.query) });
        
        let request = Request::new(
            self.method,
            uri,
            Version::Http11,
            self.headers,
            self.body,
            None,
        );

        self.client.execute(request).await
    }
}

/// Test response wrapper with assertion methods
pub struct TestResponse {
    response: Response,
}

impl TestResponse {
    fn new(response: Response) -> Self {
        Self { response }
    }

    /// Get the status code
    pub fn status(&self) -> &crate::response::StatusCode {
        &self.response.status
    }

    /// Get response body as bytes
    pub fn body(&self) -> &[u8] {
        match &self.response.body {
            crate::response::ResponseBody::Static(bytes) => bytes,
            crate::response::ResponseBody::Stream(_) => panic!("Cannot get body from streaming response"),
        }
    }

    /// Get response body as string
    pub fn text(&self) -> String {
        String::from_utf8_lossy(self.body()).to_string()
    }

    /// Parse response body as JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.body())
    }

    /// Get a header value
    pub fn header(&self, key: &str) -> Option<&String> {
        self.response.headers.get(key)
    }

    /// Get all headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.response.headers
    }

    /// Assert status code
    pub fn assert_status(&self, expected: crate::response::StatusCode) -> &Self {
        assert_eq!(
            self.response.status, expected,
            "Expected status {:?}, got {:?}",
            expected, self.response.status
        );
        self
    }

    /// Assert status is OK (200)
    pub fn assert_ok(&self) -> &Self {
        self.assert_status(crate::response::StatusCode::Ok)
    }

    /// Assert status is Created (201)
    pub fn assert_created(&self) -> &Self {
        self.assert_status(crate::response::StatusCode::Created)
    }

    /// Assert status is NotFound (404)
    pub fn assert_not_found(&self) -> &Self {
        self.assert_status(crate::response::StatusCode::NotFound)
    }

    /// Assert status is BadRequest (400)
    pub fn assert_bad_request(&self) -> &Self {
        self.assert_status(crate::response::StatusCode::BadRequest)
    }

    /// Assert status is Unauthorized (401)
    pub fn assert_unauthorized(&self) -> &Self {
        self.assert_status(crate::response::StatusCode::Unauthorized)
    }

    /// Assert status is Forbidden (403)
    pub fn assert_forbidden(&self) -> &Self {
        self.assert_status(crate::response::StatusCode::Forbidden)
    }

    /// Assert status is InternalServerError (500)
    pub fn assert_server_error(&self) -> &Self {
        self.assert_status(crate::response::StatusCode::InternalServerError)
    }

    /// Assert body contains text
    pub fn assert_body_contains(&self, text: &str) -> &Self {
        let body = self.text();
        assert!(
            body.contains(text),
            "Expected body to contain '{}', got '{}'",
            text, body
        );
        self
    }

    /// Assert exact body text
    pub fn assert_body_eq(&self, expected: &str) -> &Self {
        let body = self.text();
        assert_eq!(body, expected, "Body mismatch");
        self
    }

    /// Assert header exists
    pub fn assert_header(&self, key: &str) -> &Self {
        assert!(
            self.response.headers.contains_key(key),
            "Expected header '{}' not found",
            key
        );
        self
    }

    /// Assert header has specific value
    pub fn assert_header_eq(&self, key: &str, value: &str) -> &Self {
        let header_value = self.header(key)
            .unwrap_or_else(|| panic!("Header '{}' not found", key));
        assert_eq!(
            header_value, value,
            "Expected header '{}' to be '{}', got '{}'",
            key, value, header_value
        );
        self
    }

    /// Get the underlying response
    pub fn into_response(self) -> Response {
        self.response
    }
}

/// Extension trait for Server to create test clients
pub trait TestExt {
    fn test(self) -> TestClient;
}

impl TestExt for Server {
    fn test(self) -> TestClient {
        TestClient::new(self)
    }
}

/// Macro for creating test server with routes
#[macro_export]
macro_rules! test_server {
    ($($tt:tt)*) => {{
        $crate::Server::new()$($tt)*
    }};
}

/// Macro for asserting JSON response
#[macro_export]
macro_rules! assert_json {
    ($response:expr, $expected:expr) => {{
        let body = $response.text();
        let actual: serde_json::Value = serde_json::from_str(&body)
            .expect("Failed to parse response as JSON");
        let expected: serde_json::Value = serde_json::from_str($expected)
            .expect("Failed to parse expected JSON");
        assert_eq!(actual, expected, "JSON mismatch");
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;

    async fn hello_handler(_req: Request, mut res: Response) -> Response {
        res.set_body(b"Hello, World!".to_vec());
        res
    }

    async fn json_handler(_req: Request, res: Response) -> Response {
        res.json(serde_json::json!({"message": "Hello"}))
    }

    async fn echo_handler(req: Request, mut res: Response) -> Response {
        res.set_body(req.body.clone());
        res
    }

    async fn param_handler(req: Request, mut res: Response) -> Response {
        let name = req.param("name").map(|s| s.as_str()).unwrap_or("guest");
        res.set_body(format!("Hello, {}!", name).into_bytes());
        res
    }

    #[tokio::test]
    async fn test_basic_get_request() {
        let server = Server::new()
            .get("/hello", hello_handler);
        
        let client = server.test();
        let response = client.get("/hello").send().await;
        
        response
            .assert_ok()
            .assert_body_eq("Hello, World!");
    }

    #[tokio::test]
    async fn test_json_response() {
        let server = Server::new()
            .get("/json", json_handler);
        
        let client = server.test();
        let response = client.get("/json").send().await;
        
        response.assert_ok();
        assert_json!(response, r#"{"message": "Hello"}"#);
    }

    #[tokio::test]
    async fn test_post_request() {
        let server = Server::new()
            .post("/echo", echo_handler);
        
        let client = server.test();
        let response = client.post("/echo")
            .body("test data")
            .send()
            .await;
        
        response
            .assert_ok()
            .assert_body_eq("test data");
    }

    #[tokio::test]
    async fn test_json_request() {
        async fn json_echo(req: Request, res: Response) -> Response {
            let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
            res.json(body)
        }

        let server = Server::new()
            .post("/echo", json_echo);
        
        let client = server.test();
        let response = client.post("/echo")
            .json(r#"{"key": "value"}"#)
            .send()
            .await;
        
        response
            .assert_ok()
            .assert_header("Content-Type");
    }

    #[tokio::test]
    async fn test_not_found() {
        let server = Server::new();
        let client = server.test();
        let response = client.get("/nonexistent").send().await;
        
        response.assert_not_found();
    }

    #[tokio::test]
    async fn test_with_headers() {
        let server = Server::new()
            .get("/hello", hello_handler);
        
        let client = server.test();
        let response = client.get("/hello")
            .header("Authorization", "Bearer token")
            .header("X-Custom", "value")
            .send()
            .await;
        
        response.assert_ok();
    }

    #[tokio::test]
    async fn test_route_params() {
        let server = Server::new()
            .get("/hello/:name", param_handler);
        
        let client = server.test();
        let response = client.get("/hello/John").send().await;
        
        response
            .assert_ok()
            .assert_body_eq("Hello, John!");
    }

    #[tokio::test]
    async fn test_with_middleware() {
        fn add_header(req: Request, mut res: Response) -> Flow {
            res.headers.insert("X-Middleware".to_string(), "active".to_string());
            Flow::Next(req, res)
        }

        let server = Server::new()
            .middleware(add_header)
            .get("/hello", hello_handler);
        
        let client = server.test();
        let response = client.get("/hello").send().await;
        
        response
            .assert_ok()
            .assert_header_eq("X-Middleware", "active");
    }

    #[tokio::test]
    async fn test_query_params() {
        async fn query_handler(req: Request, mut res: Response) -> Response {
            let name = req.query("name").map(|s| s.as_str()).unwrap_or("guest");
            res.set_body(format!("Hello, {}!", name).into_bytes());
            res
        }

        let server = Server::new()
            .get("/hello", query_handler);
        
        let client = server.test();
        let response = client.get("/hello")
            .query("name", "Alice")
            .send()
            .await;
        
        response
            .assert_ok()
            .assert_body_eq("Hello, Alice!");
    }
}
