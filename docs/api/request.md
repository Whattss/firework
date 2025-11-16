# Request API Reference

Complete API documentation for the `Request` struct.

---

## Request

```rust
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Vec<u8>,
    pub remote_addr: Option<SocketAddr>,
    pub params: HashMap<String, String>,
    pub context: Context,
}
```

### Methods

#### `param(&self, name: &str) -> Option<&String>`

Get a route parameter by name.

```rust
let id = req.param("id");
```

#### `param_as<T>(&self, name: &str) -> Option<T>`

Get a route parameter as a specific type.

```rust
let id: u32 = req.param_as("id").unwrap();
```

#### `header(&self, name: &str) -> Option<&str>`

Get a header value.

```rust
let auth = req.header("Authorization");
```

#### `query(&self, name: &str) -> Option<&String>`

Get a query parameter.

```rust
let page = req.query("page");
```

#### `body_string(self) -> Result<String>`

Get the request body as a UTF-8 string.

```rust
let body = req.body_string()?;
```

#### `body_str(&self) -> Result<&str>`

Get the request body as a str slice.

#### `set_context<T>(&mut self, value: T)`

Store a value in the request context.

```rust
req.set_context(user);
```

#### `get_context<T>(&self) -> Option<T>`

Retrieve a value from the request context.

```rust
let user = req.get_context::<User>();
```
