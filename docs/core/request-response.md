# 📨 Request & Response

Deep dive into Request and Response types.

---

## Request

### Structure

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

### Accessing Data

```rust
#[get("/example")]
async fn example(req: Request) -> String {
    // Method
    let method = &req.method; // Method::GET
    
    // Path
    let path = &req.uri.path; // "/example"
    
    // Query params
    if let Some(query) = &req.uri.query {
        let page = query.get("page");
    }
    
    // Headers
    let auth = req.header("Authorization");
    let all_auth = req.header_all("Accept-Language");
    
    // Body as string
    let body_str = req.body_str().ok();
    
    // Route params
    let id = req.param("id");
    let id_typed: Option<u32> = req.param_as("id");
    
    // Remote address
    let ip = req.remote_addr.map(|addr| addr.ip());
    
    format!("Request processed")
}
```

### Context Storage

```rust
// Store data
req.set_context(User { id: 1, name: "John".into() });

// Retrieve data
let user = req.get_context::<User>();
```

---

## Response

### Structure

```rust
pub struct Response {
    pub version: Version,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: ResponseBody,
}

pub enum ResponseBody {
    Static(Vec<u8>),
    Stream(Pin<Box<dyn AsyncRead + Send>>),
}
```

### Creating Responses

```rust
// Basic response
Response::new(StatusCode::Ok, b"Hello")

// Text response
Response::default().text("Hello, World!")

// JSON response
Response::default().json(my_data)

// With headers
Response::new(StatusCode::Ok, b"data")
    .with_header("X-Custom", "Value")
    .with_header("Cache-Control", "no-cache")

// Streaming response
let file = tokio::fs::File::open("video.mp4").await?;
Response::stream(StatusCode::Ok, file)
```

### Status Codes

```rust
StatusCode::Ok                    // 200
StatusCode::Created               // 201
StatusCode::NoContent             // 204
StatusCode::Found                 // 302
StatusCode::BadRequest            // 400
StatusCode::Unauthorized          // 401
StatusCode::Forbidden             // 403
StatusCode::NotFound              // 404
StatusCode::InternalServerError   // 500
StatusCode::Custom(451, "Unavailable".into())
```

---

## Methods

### HTTP Methods

```rust
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    Unknown(String),
}
```

---

## URI

```rust
pub struct Uri {
    pub path: String,
    pub query: Option<HashMap<String, String>>,
}
```

Parse query string:
```rust
// URL: /search?q=rust&page=2
let q = req.uri.query.as_ref()?.get("q");      // Some("rust")
let page = req.uri.query.as_ref()?.get("page"); // Some("2")
```

---

## Complete Example

```rust
#[post("/api/items/:id")]
async fn complex_handler(req: Request, mut res: Response) -> Response {
    // 1. Extract path param
    let id: u32 = match req.param_as("id") {
        Some(id) => id,
        None => {
            res.status = StatusCode::BadRequest;
            res.set_body(b"Invalid ID".to_vec());
            return res;
        }
    };
    
    // 2. Check authentication
    let token = match req.header("Authorization") {
        Some(token) => token,
        None => {
            res.status = StatusCode::Unauthorized;
            return res.json(serde_json::json!({
                "error": "Authentication required"
            }));
        }
    };
    
    // 3. Parse JSON body
    let data: MyData = match serde_json::from_slice(&req.body) {
        Ok(data) => data,
        Err(e) => {
            res.status = StatusCode::BadRequest;
            return res.json(serde_json::json!({
                "error": format!("Invalid JSON: {}", e)
            }));
        }
    };
    
    // 4. Process...
    
    // 5. Return response
    res.status = StatusCode::Ok;
    res.headers.insert("X-Processed".into(), "true".into());
    res.json(serde_json::json!({
        "id": id,
        "status": "success"
    }))
}
```
