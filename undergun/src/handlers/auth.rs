use firework::{Json, Request, Response, FromRequest};
use firework_seaorm::DbConn;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use crate::models::*;
use crate::entities::{prelude::*, *};

pub async fn register(mut req: Request, mut res: Response) -> Response {
    let db = match DbConn::from_request(&mut req, &mut res).await {
        Ok(conn) => conn,
        Err(_) => {
            return firework::json!(json!({
                "error": "Database connection failed",
                "status": 500
            }));
        }
    };
    
    let input: RegisterInput = match Json::from_request(&mut req, &mut res).await {
        Ok(Json(data)) => data,
        Err(_) => {
            return firework::json!(json!({
                "error": "Invalid request body",
                "status": 400
            }));
        }
    };

    // Check if user exists
    if let Ok(Some(_)) = Users::find()
        .filter(users::Column::Username.eq(&input.username))
        .one(&db.0)
        .await
    {
        return firework::json!(json!({
            "error": "Username already exists",
            "status": 400
        }));
    }

    // Hash password
    let password_hash = bcrypt::hash(&input.password, bcrypt::DEFAULT_COST).unwrap();

    // Create user
    let user = users::ActiveModel {
        username: Set(input.username.clone()),
        email: Set(input.email),
        password_hash: Set(password_hash),
        created_at: Set(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };

    let user = Users::insert(user).exec(&db.0).await.unwrap();
    let token = crate::auth::create_token(user.last_insert_id, input.username.clone()).unwrap();

    firework::json!(AuthResponse {
        token,
        user: UserPublic {
            id: user.last_insert_id,
            username: input.username,
            email: String::new(),
        }
    })
}

pub async fn login(mut req: Request, mut res: Response) -> Response {
    let db = match DbConn::from_request(&mut req, &mut res).await {
        Ok(conn) => conn,
        Err(_) => {
            return firework::json!(json!({
                "error": "Database connection failed",
                "status": 500
            }));
        }
    };
    
    let input: LoginInput = match Json::from_request(&mut req, &mut res).await {
        Ok(Json(data)) => data,
        Err(_) => {
            return firework::json!(json!({
                "error": "Invalid request body",
                "status": 400
            }));
        }
    };

    let user = match Users::find()
        .filter(users::Column::Username.eq(&input.username))
        .one(&db.0)
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return firework::json!(json!({
                "error": "Invalid credentials",
                "status": 401
            }));
        }
    };

    if !bcrypt::verify(&input.password, &user.password_hash).unwrap_or(false) {
        return firework::json!(json!({
            "error": "Invalid credentials",
            "status": 401
        }));
    }

    let token = crate::auth::create_token(user.id, user.username.clone()).unwrap();

    firework::json!(AuthResponse {
        token,
        user: UserPublic {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    })
}

pub async fn me(mut req: Request, mut res: Response) -> Response {
    let db = match DbConn::from_request(&mut req, &mut res).await {
        Ok(conn) => conn,
        Err(_) => {
            return firework::json!(json!({
                "error": "Database connection failed",
                "status": 500
            }));
        }
    };
    
    let claims = match req.get_context::<crate::auth::Claims>() {
        Some(c) => c,
        None => {
            return firework::json!(json!({
                "error": "Unauthorized",
                "status": 401
            }));
        }
    };

    match Users::find_by_id(claims.sub).one(&db.0).await {
        Ok(Some(user)) => firework::json!(UserPublic {
            id: user.id,
            username: user.username,
            email: user.email,
        }),
        _ => firework::json!(json!({
            "error": "User not found",
            "status": 404
        })),
    }
}
