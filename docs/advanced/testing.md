# Testing

Comprehensive guide to testing Firework applications with the built-in test utilities.

## Overview

Firework provides powerful testing utilities that make it easy to write integration tests for your APIs without spinning up a real server.

## Features

- **No Server Required**: Test handlers without HTTP overhead
- **Async/Await Support**: Natural async test syntax
- **Request Builder**: Fluent API for building test requests
- **Full HTTP Simulation**: Headers, cookies, JSON, forms, etc.
- **Type-Safe**: Strongly typed assertions
- **Fast**: No network latency

## Quick Start

### Enable Testing Feature

```toml
[dependencies]
firework = { git = "...", features = ["testing"] }
```

### Basic Test

```rust
use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello, World!"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[firework_test]
    async fn test_index() {
        let client = TestClient::new(routes!());
        
        let response = client.get("/").send().await;
        
        assert_eq!(response.status(), StatusCode::Ok);
        assert_eq!(response.text(), "Hello, World!");
    }
}
```

## TestClient API

### Creating a Test Client

```rust
use firework::TestClient;

// From routes macro
let client = TestClient::new(routes!());

// With state
let state = AppState::new();
let client = TestClient::new(routes!())
    .with_state(state);

// With middleware
let client = TestClient::new(routes!())
    .with_middleware(auth_middleware);
```

### Making Requests

```rust
// GET request
let response = client.get("/users").send().await;

// POST request
let response = client.post("/users")
    .json(&user_data)
    .send()
    .await;

// PUT request
let response = client.put("/users/1")
    .json(&update_data)
    .send()
    .await;

// DELETE request
let response = client.delete("/users/1").send().await;

// PATCH request
let response = client.patch("/users/1")
    .json(&partial_update)
    .send()
    .await;
```

## Request Building

### Headers

```rust
let response = client.get("/api/users")
    .header("Authorization", "Bearer token123")
    .header("X-Custom-Header", "value")
    .send()
    .await;
```

### JSON Body

```rust
use serde_json::json;

let response = client.post("/users")
    .json(&json!({
        "username": "john",
        "email": "john@example.com"
    }))
    .send()
    .await;
```

### Form Data

```rust
let response = client.post("/login")
    .form(&[
        ("username", "john"),
        ("password", "secret")
    ])
    .send()
    .await;
```

### Query Parameters

```rust
let response = client.get("/search")
    .query(&[
        ("q", "rust"),
        ("page", "1")
    ])
    .send()
    .await;
```

### Cookies

```rust
let response = client.get("/profile")
    .cookie("session_id", "abc123")
    .send()
    .await;
```

### Body as Text

```rust
let response = client.post("/echo")
    .body("Hello, World!")
    .send()
    .await;
```

## Response Assertions

### Status Code

```rust
let response = client.get("/").send().await;

assert_eq!(response.status(), StatusCode::Ok);
assert!(response.status().is_success());
assert!(response.status().is_client_error());
assert!(response.status().is_server_error());
```

### Body Content

```rust
// As text
let text = response.text();
assert_eq!(text, "Hello, World!");

// As JSON
let json: serde_json::Value = response.json();
assert_eq!(json["username"], "john");

// As typed
#[derive(Deserialize)]
struct User {
    id: u32,
    username: String,
}

let user: User = response.json();
assert_eq!(user.username, "john");
```

### Headers

```rust
let response = client.get("/").send().await;

assert_eq!(response.header("Content-Type"), Some("text/plain"));
assert!(response.has_header("Set-Cookie"));
```

### Cookies

```rust
let response = client.post("/login")
    .json(&credentials)
    .send()
    .await;

assert!(response.has_cookie("session_id"));
let session = response.cookie("session_id").unwrap();
assert!(session.http_only);
```

## Testing CRUD Operations

```rust
use firework::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: Option<u32>,
    username: String,
    email: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[firework_test]
    async fn test_user_crud() {
        let client = TestClient::new(routes!());
        
        // CREATE
        let response = client.post("/users")
            .json(&json!({
                "username": "john",
                "email": "john@example.com"
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::Created);
        let user: User = response.json();
        assert_eq!(user.username, "john");
        let user_id = user.id.unwrap();
        
        // READ
        let response = client.get(&format!("/users/{}", user_id))
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::Ok);
        let fetched: User = response.json();
        assert_eq!(fetched.username, "john");
        
        // UPDATE
        let response = client.put(&format!("/users/{}", user_id))
            .json(&json!({
                "username": "john_updated",
                "email": "john@example.com"
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::Ok);
        let updated: User = response.json();
        assert_eq!(updated.username, "john_updated");
        
        // DELETE
        let response = client.delete(&format!("/users/{}", user_id))
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::NoContent);
        
        // Verify deletion
        let response = client.get(&format!("/users/{}", user_id))
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::NotFound);
    }
}
```

## Testing Authentication

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[firework_test]
    async fn test_protected_route_without_auth() {
        let client = TestClient::new(routes!());
        
        let response = client.get("/admin/dashboard").send().await;
        
        assert_eq!(response.status(), StatusCode::Unauthorized);
    }
    
    #[firework_test]
    async fn test_protected_route_with_auth() {
        let client = TestClient::new(routes!());
        
        // Login first
        let login_response = client.post("/login")
            .json(&json!({
                "username": "admin",
                "password": "secret"
            }))
            .send()
            .await;
        
        let token = login_response.json::<LoginResponse>().token;
        
        // Access protected route
        let response = client.get("/admin/dashboard")
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::Ok);
    }
}
```

## Testing Middleware

```rust
#[middleware]
fn add_header(req: &mut Request, res: &mut Response) -> Flow {
    res.headers.insert("X-Custom".to_string(), "test".to_string());
    Flow::Continue
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[firework_test]
    async fn test_middleware() {
        let client = TestClient::new(routes!())
            .with_middleware(add_header);
        
        let response = client.get("/").send().await;
        
        assert_eq!(response.header("X-Custom"), Some("test"));
    }
}
```

## Testing WebSockets

```rust
#[ws("/ws")]
async fn echo_ws(mut ws: WebSocket) {
    while let Some(msg) = ws.recv().await {
        ws.send(msg).await.ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[firework_test]
    async fn test_websocket() {
        let client = TestClient::new(routes!());
        
        let mut ws = client.ws("/ws").await;
        
        ws.send("Hello").await;
        let msg = ws.recv().await.unwrap();
        assert_eq!(msg.to_text(), Some("Hello"));
    }
}
```

## Testing File Uploads

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[firework_test]
    async fn test_file_upload() {
        let client = TestClient::new(routes!());
        
        let response = client.post("/upload")
            .multipart()
            .file("file", "test.txt", b"Hello, World!")
            .field("description", "Test file")
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::Ok);
    }
}
```

## Testing with Database

```rust
use firework_seaorm::{SeaOrmPlugin, DbConn};

#[cfg(test)]
mod tests {
    use super::*;
    
    async fn setup_test_db() -> TestClient {
        let db = Arc::new(SeaOrmPlugin::new("sqlite::memory:"));
        firework::register_plugin(db);
        
        TestClient::new(routes!())
    }
    
    #[firework_test]
    async fn test_with_database() {
        let client = setup_test_db().await;
        
        // Run migrations
        // Seed test data
        
        let response = client.get("/users").send().await;
        assert_eq!(response.status(), StatusCode::Ok);
    }
}
```

## Testing Error Handling

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[firework_test]
    async fn test_validation_error() {
        let client = TestClient::new(routes!());
        
        let response = client.post("/users")
            .json(&json!({
                "username": "a",  // Too short
                "email": "invalid" // Invalid email
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::BadRequest);
        
        let error: ErrorResponse = response.json();
        assert!(error.message.contains("validation"));
    }
    
    #[firework_test]
    async fn test_not_found() {
        let client = TestClient::new(routes!());
        
        let response = client.get("/nonexistent").send().await;
        
        assert_eq!(response.status(), StatusCode::NotFound);
    }
}
```

## Mocking External Services

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

struct MockEmailService {
    sent_emails: Arc<Mutex<Vec<Email>>>,
}

impl MockEmailService {
    fn new() -> Self {
        Self {
            sent_emails: Arc::new(Mutex::new(vec![])),
        }
    }
    
    async fn send(&self, email: Email) {
        self.sent_emails.lock().await.push(email);
    }
    
    async fn emails_sent(&self) -> usize {
        self.sent_emails.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[firework_test]
    async fn test_email_sending() {
        let email_service = Arc::new(MockEmailService::new());
        let client = TestClient::new(routes!())
            .with_state(email_service.clone());
        
        let response = client.post("/register")
            .json(&json!({
                "email": "user@example.com"
            }))
            .send()
            .await;
        
        assert_eq!(response.status(), StatusCode::Created);
        assert_eq!(email_service.emails_sent().await, 1);
    }
}
```

## Test Helpers

### Custom Assertions

```rust
trait ResponseExt {
    fn assert_ok(&self) -> &Self;
    fn assert_json<T: DeserializeOwned>(&self) -> T;
}

impl ResponseExt for TestResponse {
    fn assert_ok(&self) -> &Self {
        assert_eq!(self.status(), StatusCode::Ok);
        self
    }
    
    fn assert_json<T: DeserializeOwned>(&self) -> T {
        self.json()
    }
}

// Usage
let user: User = client.get("/users/1")
    .send()
    .await
    .assert_ok()
    .assert_json();
```

### Test Fixtures

```rust
struct TestFixtures {
    client: TestClient,
    test_user: User,
}

impl TestFixtures {
    async fn new() -> Self {
        let client = TestClient::new(routes!());
        
        let response = client.post("/users")
            .json(&json!({
                "username": "test",
                "email": "test@example.com"
            }))
            .send()
            .await;
        
        let test_user = response.json();
        
        Self { client, test_user }
    }
}

#[firework_test]
async fn test_with_fixtures() {
    let fixtures = TestFixtures::new().await;
    
    let response = fixtures.client
        .get(&format!("/users/{}", fixtures.test_user.id))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::Ok);
}
```

## Best Practices

### 1. Use Test Database

```rust
// Use in-memory database for tests
let db = SeaOrmPlugin::new("sqlite::memory:");
```

### 2. Clean Up Between Tests

```rust
#[firework_test]
async fn test_something() {
    let client = setup_test_client().await;
    
    // Test logic
    
    cleanup_test_data().await;
}
```

### 3. Test Edge Cases

```rust
#[firework_test]
async fn test_empty_list() {
    // Test with no data
}

#[firework_test]
async fn test_large_payload() {
    // Test with large data
}

#[firework_test]
async fn test_concurrent_requests() {
    // Test race conditions
}
```

### 4. Use Descriptive Test Names

```rust
// ❌ Bad
#[firework_test]
async fn test1() { }

// ✅ Good
#[firework_test]
async fn test_user_creation_with_valid_email() { }
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_user_crud

# Run with output
cargo test -- --nocapture

# Run in parallel
cargo test -- --test-threads=4
```

## Examples

See complete examples in:
- [examples/testing_example.rs](../../examples/testing_example.rs)
- [tests/integration_tests.rs](../../tests/integration_tests.rs)

## See Also

- [Best Practices](../examples/best-practices.md)
- [Error Handling](../core/errors.md)
- [Middleware](../core/middleware.md)
