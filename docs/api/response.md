# Response API Reference

Complete API documentation for the `Response` struct.

---

## Response

```rust
pub struct Response {
    pub version: Version,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: ResponseBody,
}
```

### Methods

#### `Response::new(status: StatusCode, body: impl Into<Vec<u8>>) -> Self`

Create a new response.

```rust
let res = Response::new(StatusCode::Ok, b"Hello");
```

#### `text(self, text: impl Into<String>) -> Self`

Set plain text body.

```rust
let res = Response::default().text("Hello, World!");
```

#### `json(self, data: impl Serialize) -> Self`

Set JSON body.

```rust
let res = Response::default().json(user);
```

#### `with_header(self, key: &str, value: &str) -> Self`

Add a header.

```rust
let res = response.with_header("X-Custom", "Value");
```

#### `header(self, key: impl Into<String>, value: impl Into<String>) -> Self`

Builder method to set header.

#### `status(&mut self, status: StatusCode) -> &mut Self`

Set response status.

```rust
res.status(StatusCode::Created);
```

#### `stream<R>(status: StatusCode, reader: R) -> Self`

Create a streaming response.

```rust
let file = tokio::fs::File::open("video.mp4").await?;
let res = Response::stream(StatusCode::Ok, file);
```
