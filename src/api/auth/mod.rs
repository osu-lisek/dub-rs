use axum::{routing::post, Router};
use serde::{Deserialize, Serialize};

use self::auth::login;

pub mod auth;
pub mod middleware;

#[derive(Deserialize)]
pub struct AccessTokenRequestBody {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub scope: String,

    // grant_type = password
    pub username: Option<String>,
    pub password: Option<String>,

    // grant_type = refresh_token
    pub refresh_token: Option<String>,

    // grant_type = authorization_code
    pub code: Option<String>,
}

#[derive(Serialize)]
pub struct AccessTokenResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub fn router() -> Router {
    Router::new().route("/oauth/token", post(login))
}
