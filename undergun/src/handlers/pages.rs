use firework::{Request, Response};

pub async fn index(_req: Request, _res: Response) -> Response {
    firework::serve!("static/index.html")
}

pub async fn login_page(_req: Request, _res: Response) -> Response {
    firework::serve!("static/login.html")
}

pub async fn register_page(_req: Request, _res: Response) -> Response {
    firework::serve!("static/register.html")
}
