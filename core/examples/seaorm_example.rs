use firework::{get, middleware, post, register_plugin, routes, scope, Flow, Request, Response};
use firework_seaorm::{
    sea_orm::{self, ConnectionTrait, EntityTrait, Set},
    RequestDbExt, SeaOrmPlugin,
};
use std::sync::Arc;

// Define entity using SeaORM macros
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Database middleware to inject connection and ensure table exists
#[middleware]
fn db_middleware(mut req: Request, res: Response) -> Flow {
    if let Some(db) = req.db() {
        // Ensure table exists (this is a hack for demo purposes - use migrations in production)
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let create_table_sql = r#"
                    CREATE TABLE IF NOT EXISTS users (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        username TEXT NOT NULL,
                        email TEXT NOT NULL
                    )
                "#;
                let _ = db.execute_unprepared(create_table_sql).await;

                // Insert sample data if table is empty
                let count_sql = "SELECT COUNT(*) FROM users";
                if let Ok(_) = db.execute_unprepared(count_sql).await {
                    let insert_sql = r#"
                        INSERT OR IGNORE INTO users (id, username, email) VALUES 
                        (1, 'alice', 'alice@example.com'),
                        (2, 'bob', 'bob@example.com')
                    "#;
                    let _ = db.execute_unprepared(insert_sql).await;
                }
            })
        });

        req.set_context(db);
    }
    Flow::Next(req, res)
}

// Routes
#[scope("/api", middleware = [db_middleware])]
mod api {
    use super::*;

    #[get("/users")]
    async fn list_users(req: Request, _res: Response) -> Response {
        let db = match req.db() {
            Some(db) => db,
            None => {
                return firework::Error::Internal("No database connection".into()).into_response()
            }
        };

        match Entity::find().all(&db).await {
            Ok(users) => {
                let user_list: Vec<serde_json::Value> = users
                    .iter()
                    .map(|u| {
                        serde_json::json!({
                            "id": u.id,
                            "username": &u.username,
                            "email": &u.email
                        })
                    })
                    .collect();

                firework::json!(serde_json::json!({ "users": user_list }))
            }
            Err(err) => firework_seaorm::helpers::db_error_to_response(err),
        }
    }

    #[get("/users/:id")]
    async fn get_user(req: Request, _res: Response) -> Response {
        let db = match req.db() {
            Some(db) => db,
            None => {
                return firework::Error::Internal("No database connection".into()).into_response()
            }
        };

        let id: i32 = match req.param("id").and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => return firework::Error::BadRequest("Invalid user ID".into()).into_response(),
        };

        match Entity::find_by_id(id).one(&db).await {
            Ok(Some(user)) => firework::json!(serde_json::json!({
                "id": user.id,
                "username": &user.username,
                "email": &user.email
            })),
            Ok(None) => firework::Error::NotFound("User not found".into()).into_response(),
            Err(err) => firework_seaorm::helpers::db_error_to_response(err),
        }
    }

    #[post("/users")]
    async fn create_user(req: Request, _res: Response) -> Response {
        let db = match req.db() {
            Some(db) => db,
            None => {
                return firework::Error::Internal("No database connection".into()).into_response()
            }
        };

        // Parse JSON body (simplified for example)
        let body = String::from_utf8_lossy(&req.body);
        let data: serde_json::Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => return firework::Error::BadRequest("Invalid JSON".into()).into_response(),
        };

        let username = data["username"].as_str().unwrap_or("").to_string();
        let email = data["email"].as_str().unwrap_or("").to_string();

        if username.is_empty() || email.is_empty() {
            return firework::Error::BadRequest("Username and email are required".into())
                .into_response();
        }

        let new_user = ActiveModel {
            username: Set(username),
            email: Set(email),
            ..Default::default()
        };

        match new_user.insert(&db).await {
            Ok(user) => {
                let mut res = firework::json!(serde_json::json!({
                    "id": user.id,
                    "username": &user.username,
                    "email": &user.email
                }));
                res.status = firework::StatusCode::Created;
                res
            }
            Err(err) => firework_seaorm::helpers::db_error_to_response(err),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Firework - SeaORM Plugin Example\n");

    // Register SeaORM plugin
    let plugin = Arc::new(SeaOrmPlugin::new("sqlite::memory:"));
    register_plugin(plugin);

    println!();
    println!("Routes:");
    println!("  GET  /api/users - List all users");
    println!("  GET  /api/users/:id - Get user by ID");
    println!("  POST /api/users - Create new user");
    println!();
    println!("Test commands:");
    println!("  curl http://localhost:8080/api/users");
    println!("  curl http://localhost:8080/api/users/1");
    println!("  curl -X POST -H 'Content-Type: application/json' -d '{{\"username\":\"charlie\",\"email\":\"charlie@example.com\"}}' http://localhost:8080/api/users");
    println!();

    println!("Listening on http://127.0.0.1:8080");
    routes!().listen("127.0.0.1:8080").await?;

    Ok(())
}
