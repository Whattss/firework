use firework::prelude::*;

#[get("/")]
async fn index() -> &'static str {
    "Hello!"
}

fn main() {
    run!();
}
