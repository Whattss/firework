#![cfg(feature = "testing")]

use firework::{Server, Response, ResponseBody, StatusCode, Flow, Request, TestClient};
use std::future::Future;
use std::pin::Pin;

fn set_body(res: &mut Response, body: impl Into<Vec<u8>>) {
    res.body = ResponseBody::Static(body.into());
}

// Helper handler that returns the incoming Response (preserves middleware mutations)
async fn passthrough_handler(_req: Request, res: Response) -> Response {
    res
}

#[tokio::test]
async fn sync_middleware_runs_and_mutates_response() {
    fn add_header(_req: &mut Request, res: &mut Response) -> Flow {
        res.headers.insert("X-Test".to_string(), "yes".to_string());
        set_body(res, b"ok");
        Flow::Continue
    }

    let server = Server::new()
        .middleware(add_header)
        .get("/", passthrough_handler);

    let client = TestClient::new(server);
    let resp = client.get("/").send().await;

    assert_eq!(resp.status(), &StatusCode::Ok);
    assert_eq!(resp.header("X-Test"), Some(&"yes".to_string()));
    assert_eq!(resp.text(), "ok");
}

#[tokio::test]
async fn async_middleware_can_short_circuit_request() {
    fn stop_middleware<'a>(
        _req: &'a mut Request,
        _res: &'a mut Response,
    ) -> Pin<Box<dyn Future<Output = Flow> + Send + 'a>> {
        Box::pin(async { Flow::Stop(Response::new(StatusCode::Forbidden, b"blocked")) })
    }

    async fn never_called_handler(_req: Request, _res: Response) -> Response {
        Response::new(StatusCode::Ok, b"should_not_happen")
    }

    let server = Server::new()
        .async_middleware(stop_middleware)
        .get("/", never_called_handler);

    let client = TestClient::new(server);
    let resp = client.get("/").send().await;

    assert_eq!(resp.status(), &StatusCode::Forbidden);
    assert_eq!(resp.text(), "blocked");
}
