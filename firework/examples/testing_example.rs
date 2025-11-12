use firework::{
    firework_test, get, middleware, routes, scope, test::TestExt, Flow, Request, Response,
};

#[get("/")]
async fn index(_req: Request, mut res: Response) -> Response {
    res.set_body(b"Hello, World!".to_vec());
    res
}

#[get("/users/:id")]
async fn get_user(req: Request, res: Response) -> Response {
    let user_id = req.param("id").map(|s| s.as_str()).unwrap_or("unknown");
    res.json(serde_json::json!({
        "id": user_id,
        "name": "John Doe"
    }))
}

#[get("/query")]
async fn query_test(req: Request, mut res: Response) -> Response {
    let name = req.query("name").map(|s| s.as_str()).unwrap_or("guest");
    res.set_body(format!("Hello, {}!", name).into_bytes());
    res
}

#[derive(Clone)]
struct UserId(String);

#[middleware]
fn auth_middleware(mut req: Request, mut res: Response) -> Flow {
    if let Some(auth) = req.headers.get("Authorization") {
        if auth.first().map(|s| s.as_str()) == Some("Bearer valid_token") {
            req.set_context(UserId("123".to_string()));
            return Flow::Next(req, res);
        }
    }
    res.status(firework::StatusCode::Unauthorized);
    res.set_body(b"Unauthorized".to_vec());
    Flow::Stop(res)
}

#[scope("/api", middleware = [auth_middleware])]
mod api {
    use super::*;

    #[get("/protected")]
    async fn protected(req: Request, res: Response) -> Response {
        let user_id = req
            .get_context::<UserId>()
            .map(|id| id.0.clone())
            .unwrap_or_else(|| "unknown".to_string());
        res.json(serde_json::json!({
            "message": "Protected resource",
            "user_id": user_id
        }))
    }

    #[get("/users")]
    async fn list_users(_req: Request, res: Response) -> Response {
        res.json(serde_json::json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]))
    }
}

#[firework_test]
async fn test_basic_route() {
    let server = routes!();
    let client = server.test();

    let response = client.get("/").send().await;
    response.assert_ok().assert_body_eq("Hello, World!");
}

#[firework_test]
async fn test_route_with_params() {
    let server = routes!();
    let client = server.test();

    let response = client.get("/users/42").send().await;
    response.assert_ok();

    let json: serde_json::Value = response.json().expect("Invalid JSON");
    assert_eq!(json["id"], "42");
    assert_eq!(json["name"], "John Doe");
}

#[firework_test]
async fn test_query_params() {
    let server = routes!();
    let client = server.test();

    let response = client.get("/query").query("name", "Alice").send().await;
    response.assert_ok().assert_body_eq("Hello, Alice!");
}

#[firework_test]
async fn test_not_found() {
    let server = routes!();
    let client = server.test();

    let response = client.get("/nonexistent").send().await;
    response.assert_not_found();
}

#[firework_test]
async fn test_protected_route_without_auth() {
    let server = routes!();
    let client = server.test();

    let response = client.get("/api/protected").send().await;
    response.assert_unauthorized();
}

#[firework_test]
async fn test_protected_route_with_auth() {
    let server = routes!();
    let client = server.test();

    let response = client
        .get("/api/protected")
        .header("Authorization", "Bearer valid_token")
        .send()
        .await;

    response.assert_ok();
    let json: serde_json::Value = response.json().expect("Invalid JSON");
    assert_eq!(json["user_id"], "123");
}

#[firework_test]
async fn test_api_list_users() {
    let server = routes!();
    let client = server.test();

    let response = client
        .get("/api/users")
        .header("Authorization", "Bearer valid_token")
        .send()
        .await;

    response.assert_ok();
    let json: serde_json::Value = response.json().expect("Invalid JSON");
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[firework_test]
async fn test_multiple_requests() {
    let server = routes!();
    let client = server.test();

    let response1 = client.get("/").send().await;
    response1.assert_ok();

    let response2 = client.get("/users/1").send().await;
    response2.assert_ok();

    assert_eq!(response1.text(), "Hello, World!");
    assert!(response2.text().contains("John Doe"));
}

#[firework_test]
async fn test_post_request() {
    use firework::post;

    #[post("/echo")]
    async fn echo(req: Request, mut res: Response) -> Response {
        res.set_body(req.body.clone());
        res
    }

    let server = firework::Server::new().post("/echo", echo);
    let client = server.test();

    let response = client.post("/echo").body("test data").send().await;

    response.assert_ok().assert_body_eq("test data");
}

#[firework_test]
async fn test_json_request_and_response() {
    use firework::post;

    #[post("/json")]
    async fn handle_json(req: Request, res: Response) -> Response {
        let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
        res.json(serde_json::json!({
            "received": body,
            "processed": true
        }))
    }

    let server = firework::Server::new().post("/json", handle_json);
    let client = server.test();

    let response = client
        .post("/json")
        .json(r#"{"name": "Alice", "age": 30}"#)
        .send()
        .await;

    response
        .assert_ok()
        .assert_header("Content-Type")
        .assert_header_eq("Content-Type", "application/json");

    let json: serde_json::Value = response.json().expect("Invalid JSON");
    assert_eq!(json["processed"], true);
    assert_eq!(json["received"]["name"], "Alice");
}

#[firework_test]
async fn test_custom_headers() {
    use firework::get;

    #[get("/headers")]
    async fn check_headers(req: Request, mut res: Response) -> Response {
        let custom = req.header("X-Custom-Header").unwrap_or("not found");
        res.set_body(custom.as_bytes().to_vec());
        res
    }

    let server = firework::Server::new().get("/headers", check_headers);
    let client = server.test();

    let response = client
        .get("/headers")
        .header("X-Custom-Header", "custom value")
        .send()
        .await;

    response.assert_ok().assert_body_eq("custom value");
}

#[firework_test]
async fn test_chained_assertions() {
    let server = routes!();
    let client = server.test();

    client
        .get("/")
        .send()
        .await
        .assert_ok()
        .assert_body_contains("Hello")
        .assert_body_contains("World");
}

fn main() {
    println!("Run tests with: cargo test --example testing_example --features testing");
}
