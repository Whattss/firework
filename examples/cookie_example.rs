use firework::prelude::*;
use std::sync::Arc;

#[get("/")]
async fn index() -> Response {
    html!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Cookie Example</title>
</head>
<body>
    <h1>Firework Cookie Example</h1>
    
    <h2>Set Cookie</h2>
    <p><a href="/set-cookie">Set a cookie</a></p>
    
    <h2>Read Cookie</h2>
    <p><a href="/get-cookie">Read the cookie</a></p>
    
    <h2>Delete Cookie</h2>
    <p><a href="/delete-cookie">Delete the cookie</a></p>
    
    <h2>Secure Cookie (HttpOnly + Secure + SameSite)</h2>
    <p><a href="/secure-cookie">Set secure cookie</a></p>
</body>
</html>
    "#)
}

#[get("/set-cookie")]
async fn set_cookie(mut res: Response) -> Response {
    // Create a simple cookie
    let cookie = Cookie::new("my_cookie", "hello_world")
        .path("/")
        .max_age(3600); // 1 hour
    
    res.set_cookie(cookie);
    
    html!(r#"
<!DOCTYPE html>
<html>
<head><title>Cookie Set</title></head>
<body>
    <h1>Cookie Set!</h1>
    <p>Cookie "my_cookie" has been set.</p>
    <p><a href="/get-cookie">Read it</a> | <a href="/">Home</a></p>
</body>
</html>
    "#)
}

#[get("/get-cookie")]
async fn get_cookie(req: Request) -> Response {
    match req.cookie("my_cookie") {
        Some(value) => html!(format!(r#"
<!DOCTYPE html>
<html>
<head><title>Cookie Value</title></head>
<body>
    <h1>Cookie Found!</h1>
    <p>Value: <code>{}</code></p>
    <p><a href="/">Home</a></p>
</body>
</html>
        "#, value)),
        None => html!(r#"
<!DOCTYPE html>
<html>
<head><title>No Cookie</title></head>
<body>
    <h1>No Cookie Found</h1>
    <p>The cookie hasn't been set yet.</p>
    <p><a href="/set-cookie">Set it</a> | <a href="/">Home</a></p>
</body>
</html>
        "#),
    }
}

#[get("/delete-cookie")]
async fn delete_cookie(mut res: Response) -> Response {
    res.delete_cookie("my_cookie");
    
    html!(r#"
<!DOCTYPE html>
<html>
<head><title>Cookie Deleted</title></head>
<body>
    <h1>Cookie Deleted!</h1>
    <p>Cookie "my_cookie" has been removed.</p>
    <p><a href="/get-cookie">Verify</a> | <a href="/">Home</a></p>
</body>
</html>
    "#)
}

#[get("/secure-cookie")]
async fn secure_cookie(mut res: Response) -> Response {
    // Create a secure cookie (for production use)
    let cookie = Cookie::new("session_id", "abc123def456")
        .http_only(true)       // Not accessible via JavaScript
        .secure(true)          // Only sent over HTTPS
        .same_site(SameSite::Strict)  // CSRF protection
        .max_age(86400)        // 24 hours
        .path("/");
    
    res.set_cookie(cookie);
    
    html!(r#"
<!DOCTYPE html>
<html>
<head><title>Secure Cookie Set</title></head>
<body>
    <h1>Secure Cookie Set!</h1>
    <p>A secure cookie has been set with:</p>
    <ul>
        <li>HttpOnly: Yes (not accessible via JavaScript)</li>
        <li>Secure: Yes (only sent over HTTPS)</li>
        <li>SameSite: Strict (CSRF protection)</li>
        <li>Max-Age: 24 hours</li>
    </ul>
    <p><a href="/">Home</a></p>
</body>
</html>
    "#)
}

#[get("/all-cookies")]
async fn all_cookies(req: Request) -> Response {
    let cookies = req.cookies();
    
    let cookie_list = if cookies.is_empty() {
        "<p>No cookies found.</p>".to_string()
    } else {
        let items: Vec<String> = cookies
            .iter()
            .map(|(k, v)| format!("<li><strong>{}:</strong> {}</li>", k, v))
            .collect();
        format!("<ul>{}</ul>", items.join("\n"))
    };
    
    html!(format!(r#"
<!DOCTYPE html>
<html>
<head><title>All Cookies</title></head>
<body>
    <h1>All Cookies</h1>
    {}
    <p><a href="/">Home</a></p>
</body>
</html>
    "#, cookie_list))
}

#[tokio::main]
async fn main() {
    println!("🔥 Firework Cookie Example");
    println!("");
    println!("Open: http://localhost:8080");
    println!("");
    println!("Try the different cookie operations:");
    println!("  - Set a cookie");
    println!("  - Read the cookie value");
    println!("  - Delete the cookie");
    println!("  - Create a secure cookie");
    println!("");
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
