use firework::{Flow, Request, Response};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

const SECRET: &[u8] = b"your-secret-key-change-in-production";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,
    pub username: String,
    pub exp: usize,
}

pub fn create_token(user_id: i32, username: String) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET))
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn auth_middleware(mut req: Request, res: Response) -> Flow {
    let auth_header = req.headers.get("authorization")
        .and_then(|v| v.first())
        .map(|s| s.as_str());

    if let Some(auth) = auth_header {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            if let Ok(claims) = verify_token(token) {
                req.set_context(claims);
                return Flow::Next(req, res);
            }
        }
    }

    Flow::Stop(firework::json!(json!({
        "error": "Unauthorized",
        "status": 401
    })))
}
