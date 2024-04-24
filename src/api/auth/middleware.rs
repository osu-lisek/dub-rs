use std::sync::Arc;

use axum::{
    extract::Request, http::StatusCode, middleware::Next, response::IntoResponse, Extension, Json,
};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tracing::debug;

use crate::{context::Context, db::user::User, utils::user_utils::get_user_by_id};

#[derive(Debug, Serialize)]
pub enum ErrorKind {
    Auth,
}

#[derive(Debug, Serialize)]
pub struct AuthFailedError {
    pub ok: bool,
    pub message: String,
    pub kind: ErrorKind,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenClaim {
    pub sub: i32,
    pub exp: i64,
    pub iat: i64,
    //hmac of our password to verify is this token valid or not
    pub hash: String,
}

type HmacSha256 = Hmac<Sha256>;

pub async fn auth(
    Extension(ctx): Extension<Arc<Context>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<AuthFailedError>)> {
    let token = req.headers().get("Authorization");
    let token = match token {
        Some(token) => token,
        None => {
            req.extensions_mut().insert(None as Option<User>);

            debug!("No token found, processing to next");
            return Ok(next.run(req).await);
        }
    };

    let token_data = token.to_str().unwrap().split(' ').collect::<Vec<&str>>();

    if token_data.len() != 2 {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthFailedError {
                ok: false,
                message: "Invalid token".to_string(),
                kind: ErrorKind::Auth,
            }),
        ));
    }

    let token_type = token_data[0];
    if token_type != "Bearer" {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthFailedError {
                ok: false,
                message: "Invalid token".to_string(),
                kind: ErrorKind::Auth,
            }),
        ));
    }

    let token = token_data[1];

    let key: Result<Hmac<Sha256>, hmac::digest::InvalidLength> =
        HmacSha256::new_from_slice(ctx.config.token_hmac_secret.as_bytes());

    if let Err(error) = key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthFailedError {
                ok: false,
                message: error.to_string(),
                kind: ErrorKind::Auth,
            }),
        ));
    }

    let key = key.unwrap();

    let claims: Result<TokenClaim, jwt::Error> = token.verify_with_key(&key);

    if let Err(error) = claims {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthFailedError {
                ok: false,
                message: error.to_string(),
                kind: ErrorKind::Auth,
            }),
        ));
    }

    let claims = claims.unwrap();

    let user = get_user_by_id(&ctx.pool, claims.sub).await;

    if let Err(_error) = user {
        return Ok(next.run(req).await);
    }

    let user = user.unwrap();

    if user.is_none() {
        return Ok(next.run(req).await);
    }

    let user = user.unwrap();

    let mut key = HmacSha256::new_from_slice(ctx.config.token_hmac_secret.as_bytes()).unwrap();

    key.update(user.password.as_bytes());

    let result = key.finalize();
    let verify_bytes: Vec<u8> = result.into_bytes().to_vec();
    //Verifing hmac of password
    let verified_hash = STANDARD_NO_PAD.encode(&verify_bytes);

    if verified_hash != claims.hash.clone() {
        debug!(
            "Mac verification failed, bytes #1: {:#?}, \nbytes #2: {:#?}",
            claims.hash, verified_hash
        );
    } else {
        req.extensions_mut().insert(Some(user));
    }

    Ok(next.run(req).await)
}
