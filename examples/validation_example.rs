use firework::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(email(message = "Must be a valid email address"))]
    email: String,
    
    #[validate(length(min = 3, max = 20, message = "Username must be 3-20 characters"))]
    #[validate(custom = "validators::validate_username")]
    username: String,
    
    #[validate(custom = "validators::validate_strong_password")]
    password: String,
    
    #[validate(range(min = 18, max = 120, message = "Age must be between 18 and 120"))]
    age: u8,
}

#[derive(Deserialize, Validate)]
struct SearchQuery {
    #[validate(length(min = 1, max = 100))]
    q: String,
    
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

#[get("/")]
async fn index() -> Response {
    html!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Validation Example</title>
    <style>
        body { font-family: sans-serif; max-width: 800px; margin: 50px auto; }
        form { background: #f0f0f0; padding: 20px; margin: 20px 0; border-radius: 5px; }
        input, button { display: block; margin: 10px 0; padding: 8px; width: 100%; }
        button { background: #007bff; color: white; border: none; cursor: pointer; }
        .error { color: red; }
        .success { color: green; }
    </style>
</head>
<body>
    <h1>🔥 Firework Validation Example</h1>
    
    <h2>Create User (POST /api/users)</h2>
    <form id="userForm">
        <input type="email" name="email" placeholder="Email" required>
        <input type="text" name="username" placeholder="Username (3-20 chars, alphanumeric)" required>
        <input type="password" name="password" placeholder="Password (strong)" required>
        <input type="number" name="age" placeholder="Age (18-120)" required>
        <button type="submit">Create User</button>
    </form>
    <div id="userResult"></div>
    
    <h2>Search (GET /api/search)</h2>
    <form id="searchForm">
        <input type="text" name="q" placeholder="Search query" required>
        <input type="number" name="limit" placeholder="Limit (1-100)" value="10">
        <button type="submit">Search</button>
    </form>
    <div id="searchResult"></div>
    
    <script>
        document.getElementById('userForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const data = {
                email: formData.get('email'),
                username: formData.get('username'),
                password: formData.get('password'),
                age: parseInt(formData.get('age'))
            };
            
            try {
                const res = await fetch('/api/users', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });
                const result = await res.json();
                
                const div = document.getElementById('userResult');
                if (res.ok) {
                    div.className = 'success';
                    div.textContent = '✅ ' + JSON.stringify(result);
                } else {
                    div.className = 'error';
                    div.textContent = '❌ ' + (result.error || 'Validation failed');
                }
            } catch (err) {
                document.getElementById('userResult').className = 'error';
                document.getElementById('userResult').textContent = '❌ ' + err;
            }
        });
        
        document.getElementById('searchForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const q = formData.get('q');
            const limit = formData.get('limit');
            
            try {
                const res = await fetch(`/api/search?q=${q}&limit=${limit}`);
                const result = await res.json();
                
                const div = document.getElementById('searchResult');
                if (res.ok) {
                    div.className = 'success';
                    div.textContent = '✅ ' + JSON.stringify(result);
                } else {
                    div.className = 'error';
                    div.textContent = '❌ ' + (result.error || 'Validation failed');
                }
            } catch (err) {
                document.getElementById('searchResult').className = 'error';
                document.getElementById('searchResult').textContent = '❌ ' + err;
            }
        });
    </script>
</body>
</html>
    "#)
}

#[post("/api/users")]
async fn create_user(Validated(Json(user)): Validated<Json<CreateUser>>) -> Response {
    // user is already validated!
    json!({
        "success": true,
        "message": "User created successfully",
        "user": {
            "email": user.email,
            "username": user.username,
            "age": user.age
        }
    })
}

#[get("/api/search")]
async fn search(Validated(Query(query)): Validated<Query<SearchQuery>>) -> Response {
    // query is already validated!
    json!({
        "success": true,
        "query": query.q,
        "limit": query.limit,
        "results": vec!["Result 1", "Result 2", "Result 3"]
    })
}

// Test endpoints for validation errors
#[post("/api/test/invalid")]
async fn test_invalid() -> Response {
    json!({"test": "This should work without validation"})
}

#[tokio::main]
async fn main() {
    println!("🔥 Firework Validation Example");
    println!("");
    println!("Open: http://localhost:8080");
    println!("");
    println!("Try these validation scenarios:");
    println!("  ✅ Valid user:");
    println!(r#"     curl -X POST http://localhost:8080/api/users \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{{"email":"user@example.com","username":"john_doe","password":"Strong123","age":25}}'"#);
    println!("");
    println!("  ❌ Invalid email:");
    println!(r#"     curl -X POST http://localhost:8080/api/users \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{{"email":"invalid","username":"john_doe","password":"Strong123","age":25}}'"#);
    println!("");
    println!("  ❌ Weak password:");
    println!(r#"     curl -X POST http://localhost:8080/api/users \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{{"email":"user@example.com","username":"john_doe","password":"weak","age":25}}'"#);
    println!("");
    println!("  ❌ Out of range age:");
    println!(r#"     curl -X POST http://localhost:8080/api/users \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{{"email":"user@example.com","username":"john_doe","password":"Strong123","age":150}}'"#);
    println!("");
    
    routes!()
        .listen("127.0.0.1:8080")
        .await
        .expect("Failed to start server");
}
