# Server API Reference

Complete API documentation for the `Server` struct.

---

## Server

```rust
pub struct Server {
    pub(crate) router: Router,
    pub(crate) middlewares: Vec<Middleware>,
    pub(crate) async_middlewares: Vec<AsyncMiddleware>,
    // ...
}
```

### Methods

#### `Server::new() -> Self`

Create a new server instance.

```rust
let server = Server::new();
```

#### `listen(self, addr: &str) -> Result<()>`

Start the server on the specified address.

```rust
server.listen("127.0.0.1:8080").await?;
```

#### `listen_with_config(self) -> Result<()>`

Start server using configuration from `Firework.toml`.

```rust
server.listen_with_config().await?;
```

#### `get<H>(self, path: &str, handler: H) -> Self`

Register a GET route.

```rust
server = server.get("/", index_handler);
```

#### `post<H>(self, path: &str, handler: H) -> Self`

Register a POST route.

#### `put<H>(self, path: &str, handler: H) -> Self`

Register a PUT route.

#### `patch<H>(self, path: &str, handler: H) -> Self`

Register a PATCH route.

#### `delete<H>(self, path: &str, handler: H) -> Self`

Register a DELETE route.

#### `middleware(self, mw: Middleware) -> Self`

Add a synchronous middleware.

```rust
server = server.middleware(my_middleware);
```

#### `async_middleware(self, mw: AsyncMiddleware) -> Self`

Add an asynchronous middleware.

#### `websocket<H>(self, path: &str, handler: H) -> Self`

Register a WebSocket route.

```rust
server = server.websocket("/ws", ws_handler);
```

#### `prefix(self, prefix: &str) -> Self`

Set a global prefix for all routes.

```rust
server = server.prefix("/api/v1");
```

#### `scope<F>(self, prefix: &str, configurator: F) -> Self`

Create a route scope.

```rust
server = server.scope("/admin", |scope| {
    scope.get("/dashboard", dashboard_handler)
});
```
