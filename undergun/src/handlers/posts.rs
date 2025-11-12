use firework::{Json, Request, Response, Path, FromRequest};
use firework_seaorm::DbConn;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde_json::json;
use crate::models::*;
use crate::entities::{prelude::*, *};

pub async fn list_posts(mut req: Request, mut res: Response) -> Response {
    let db = match DbConn::from_request(&mut req, &mut res).await {
        Ok(conn) => conn,
        Err(_) => {
            return firework::json!(json!({
                "error": "Database connection failed",
                "status": 500
            }));
        }
    };

    let posts = Posts::find()
        .order_by_desc(posts::Column::CreatedAt)
        .all(&db.0)
        .await
        .unwrap_or_default();

    let mut posts_with_authors = Vec::new();
    for post in posts {
        if let Ok(Some(user)) = Users::find_by_id(post.user_id).one(&db.0).await {
            posts_with_authors.push(PostWithAuthor {
                id: post.id,
                title: post.title,
                content: post.content,
                created_at: post.created_at,
                updated_at: post.updated_at,
                author: UserPublic {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                },
            });
        }
    }

    firework::json!(posts_with_authors)
}

pub async fn get_post(mut req: Request, mut res: Response) -> Response {
    let db = match DbConn::from_request(&mut req, &mut res).await {
        Ok(conn) => conn,
        Err(_) => {
            return firework::json!(json!({
                "error": "Database connection failed",
                "status": 500
            }));
        }
    };
    
    let id: i32 = match Path::from_request(&mut req, &mut res).await {
        Ok(Path(id)) => id,
        Err(_) => {
            return firework::json!(json!({
                "error": "Invalid ID",
                "status": 400
            }));
        }
    };

    match Posts::find_by_id(id).one(&db.0).await {
        Ok(Some(post)) => {
            if let Ok(Some(user)) = Users::find_by_id(post.user_id).one(&db.0).await {
                return firework::json!(PostWithAuthor {
                    id: post.id,
                    title: post.title,
                    content: post.content,
                    created_at: post.created_at,
                    updated_at: post.updated_at,
                    author: UserPublic {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                    },
                });
            }
            firework::json!(json!({
                "error": "Author not found",
                "status": 500
            }))
        }
        _ => firework::json!(json!({
            "error": "Post not found",
            "status": 404
        })),
    }
}

pub async fn create_post(mut req: Request, mut res: Response) -> Response {
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
    
    let input: CreatePostInput = match Json::from_request(&mut req, &mut res).await {
        Ok(Json(data)) => data,
        Err(_) => {
            return firework::json!(json!({
                "error": "Invalid request body",
                "status": 400
            }));
        }
    };

    let now = chrono::Utc::now().to_rfc3339();
    let post = posts::ActiveModel {
        user_id: Set(claims.sub),
        title: Set(input.title),
        content: Set(input.content),
        created_at: Set(now.clone()),
        updated_at: Set(now),
        ..Default::default()
    };

    let result = Posts::insert(post).exec(&db.0).await.unwrap();

    firework::json!(json!({
        "id": result.last_insert_id,
        "status": 201
    }))
}

pub async fn delete_post(mut req: Request, mut res: Response) -> Response {
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
    
    let id: i32 = match Path::from_request(&mut req, &mut res).await {
        Ok(Path(id)) => id,
        Err(_) => {
            return firework::json!(json!({
                "error": "Invalid ID",
                "status": 400
            }));
        }
    };

    match Posts::find_by_id(id).one(&db.0).await {
        Ok(Some(post)) => {
            if post.user_id != claims.sub {
                return firework::json!(json!({
                    "error": "Forbidden",
                    "status": 403
                }));
            }

            Posts::delete_by_id(id).exec(&db.0).await.unwrap();
            firework::json!(json!({
                "message": "Post deleted",
                "status": 200
            }))
        }
        _ => firework::json!(json!({
            "error": "Post not found",
            "status": 404
        })),
    }
}
