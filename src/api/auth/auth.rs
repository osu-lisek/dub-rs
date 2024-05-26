use std::{ops::Add, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    extract,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use jwt::{Error, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha256;

use crate::{
    context::Context,
    utils::{
        oauth_utils::get_app_by_id,
        user_utils::{find_user_by_id_or_username, get_user_by_id},
    },
};

use super::{middleware::TokenClaim, AccessTokenRequestBody, AccessTokenResponse};

pub async fn login(req: extract::Request<Body>) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let context = parts.extensions.get::<Arc<Context>>().unwrap();

    let body = to_bytes(body, usize::MAX).await;
    if let Err(error) = body {
        return (
            StatusCode::BAD_REQUEST,
            Json(AccessTokenResponse {
                access_token: None,
                refresh_token: None,
                expires_in: None,
                token_type: None,
                error: Some("body_parse".to_string()),
                hint: Some("body".to_string()),
                message: Some(error.to_string()),
            }),
        );
    }

    let body = body.unwrap();

    let body: Result<AccessTokenRequestBody, serde_json::Error> = serde_json::from_slice(&body);

    if let Err(error) = body {
        return (
            StatusCode::BAD_REQUEST,
            Json(AccessTokenResponse {
                access_token: None,
                refresh_token: None,
                expires_in: None,
                token_type: None,
                error: Some("body_parse".to_string()),
                hint: Some("body".to_string()),
                message: Some(error.to_string()),
            }),
        );
    }

    let body = body.unwrap();

    let app = get_app_by_id(&context.pool, body.client_id.parse::<i32>().unwrap_or(0)).await;

    match app {
        Ok(app) => {
            if app.is_none() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(AccessTokenResponse {
                        access_token: None,
                        refresh_token: None,
                        expires_in: None,
                        token_type: None,
                        error: Some("client_id".to_string()),
                        hint: Some("client_id".to_string()),
                        message: Some("client_id is incorrect".to_string()),
                    }),
                );
            }
            let app = app.unwrap();

            if !app.allowed_grant_type.contains(&body.grant_type) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(AccessTokenResponse {
                        access_token: None,
                        refresh_token: None,
                        expires_in: None,
                        token_type: None,
                        error: Some("grant_type".to_string()),
                        hint: Some("grant_type".to_string()),
                        message: Some("grant_type is incorrect".to_string()),
                    }),
                );
            }

            if app.secret != body.client_secret {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(AccessTokenResponse {
                        access_token: None,
                        refresh_token: None,
                        expires_in: None,
                        token_type: None,
                        error: Some("client_secret".to_string()),
                        hint: Some("client_secret".to_string()),
                        message: Some("client_secret is incorrect".to_string()),
                    }),
                );
            }
        }
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    token_type: None,
                    error: Some("client_id".to_string()),
                    hint: Some("client_id".to_string()),
                    message: Some("client_id is incorrect".to_string()),
                }),
            );
        }
    }

    if body.grant_type == "refresh_token" {
        if body.refresh_token.is_none() {
            return (
                StatusCode::BAD_REQUEST,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    token_type: None,
                    error: Some("username".to_string()),
                    hint: Some("username".to_string()),
                    message: Some("refresh token are required for this grant_type".to_string()),
                }),
            );
        }
        let refresh_token = body.refresh_token.unwrap();

        let hmac: hmac::digest::core_api::CoreWrapper<hmac::HmacCore<Sha256>> =
            Hmac::new_from_slice(context.config.token_hmac_secret.as_bytes()).unwrap();

        let token: Result<Token<Header, TokenClaim, _>, Error> =
            refresh_token.verify_with_key(&hmac);

        if let Err(error) = token {
            return (
                StatusCode::BAD_REQUEST,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    token_type: None,
                    error: Some("refresh_token".to_string()),
                    hint: Some("refresh_token".to_string()),
                    message: Some(error.to_string()),
                }),
            );
        }

        let token = token.unwrap();

        let claims: &TokenClaim = token.claims();
        let user = get_user_by_id(&context.pool, claims.sub).await;
        if user.is_err() {
            return (
                StatusCode::BAD_REQUEST,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    token_type: None,
                    error: Some("refresh_token".to_string()),
                    hint: Some("refresh_token".to_string()),
                    message: Some("Invalid refresh token.".to_string()),
                }),
            );
        }

        let user = user.unwrap();

        if user.is_none() {
            return (
                StatusCode::BAD_REQUEST,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    token_type: None,
                    error: Some("refresh_token".to_string()),
                    hint: Some("refresh_token".to_string()),
                    message: Some("Invalid refresh token.".to_string()),
                }),
            );
        }

        let user = user.unwrap();
        //Verifying hash
        let mut hmac: hmac::digest::core_api::CoreWrapper<hmac::HmacCore<Sha256>> =
            Hmac::new_from_slice(context.config.token_hmac_secret.as_bytes()).unwrap();

        hmac.update(user.password.as_bytes());

        let result = hmac.finalize();
        let result = result.into_bytes();
        let result = result.to_vec();

        if result
            != STANDARD_NO_PAD
                .decode(claims.hash.clone())
                .unwrap_or_default()
        {
            return (
                StatusCode::BAD_REQUEST,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    token_type: None,
                    error: Some("refresh_token".to_string()),
                    hint: Some("refresh_token".to_string()),
                    message: Some("Invalid refresh token.".to_string()),
                }),
            );
        }

        let mut hmac: hmac::digest::core_api::CoreWrapper<hmac::HmacCore<Sha256>> =
            Hmac::new_from_slice(context.config.token_hmac_secret.as_bytes()).unwrap();

        hmac.update(user.password.as_bytes());

        let result = hmac.finalize();
        let result = result.into_bytes();
        let result = result.to_vec();

        let result = STANDARD_NO_PAD.encode(result);
        let hmac: hmac::digest::core_api::CoreWrapper<hmac::HmacCore<Sha256>> =
            Hmac::new_from_slice(context.config.token_hmac_secret.as_bytes()).unwrap();
        let exp = chrono::Utc::now().timestamp() + 3600;

        let access_token_claims = TokenClaim {
            exp,
            sub: user.id,
            iat: chrono::Utc::now().timestamp(),
            hash: result.clone(),
        };

        let access_token_claims = Token::new(
            Header {
                algorithm: jwt::AlgorithmType::Hs256,
                ..Default::default()
            },
            access_token_claims,
        )
        .sign_with_key(&hmac);

        let access_token_claims = match access_token_claims {
            Err(error) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AccessTokenResponse {
                        access_token: None,
                        refresh_token: None,
                        expires_in: None,
                        token_type: None,
                        error: Some("internal_error".to_string()),
                        hint: Some("internal_error".to_string()),
                        message: Some(error.to_string()),
                    }),
                );
            }
            Ok(claims) => claims,
        };

        let refresh_token_exp = chrono::Utc::now().add(chrono::Days::new(14)).timestamp();

        let refresh_token_claims = TokenClaim {
            exp: refresh_token_exp,
            sub: user.id,
            iat: chrono::Utc::now().timestamp(),
            hash: result,
        };

        let refresh_token_claims = Token::new(
            Header {
                algorithm: jwt::AlgorithmType::Hs256,
                ..Default::default()
            },
            refresh_token_claims,
        )
        .sign_with_key(&hmac);

        let refresh_token_claims = match refresh_token_claims {
            Err(error) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AccessTokenResponse {
                        access_token: None,
                        refresh_token: None,
                        expires_in: None,
                        token_type: None,
                        error: Some("internal_error".to_string()),
                        hint: Some("internal_error".to_string()),
                        message: Some(error.to_string()),
                    }),
                );
            }
            Ok(claims) => claims,
        };

        return (
            StatusCode::OK,
            Json(AccessTokenResponse {
                access_token: Some(access_token_claims.as_str().to_string()),
                refresh_token: Some(refresh_token_claims.as_str().to_string()),
                expires_in: Some(3600),
                token_type: Some("Bearer".to_string()),
                error: None,
                hint: None,
                message: None,
            }),
        );
    }

    if body.grant_type != "password" {
        return (
            StatusCode::BAD_REQUEST,
            Json(AccessTokenResponse {
                access_token: None,
                refresh_token: None,
                expires_in: None,
                token_type: None,
                error: Some("grant_type".to_string()),
                hint: Some("grant_type".to_string()),
                message: Some("grant_type must be password".to_string()),
            }),
        );
    }

    if body.username.is_none() || body.password.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AccessTokenResponse {
                access_token: None,
                refresh_token: None,
                expires_in: None,
                token_type: None,
                error: Some("username".to_string()),
                hint: Some("username".to_string()),
                message: Some("username and password are required for this grant_type".to_string()),
            }),
        );
    }

    let user = find_user_by_id_or_username(&context.pool, body.username.unwrap()).await;

    if user.is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AccessTokenResponse {
                access_token: None,
                refresh_token: None,
                expires_in: None,
                token_type: None,
                error: Some("invalid_credentials".to_string()),
                hint: Some("username".to_string()),
                message: Some("Invalid username".to_string()),
            }),
        );
    }

    let user = user.unwrap();

    if user.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AccessTokenResponse {
                access_token: None,
                refresh_token: None,
                expires_in: None,
                token_type: None,
                error: Some("invalid_credentials".to_string()),
                hint: Some("username".to_string()),
                message: Some("Invalid credentials.".to_string()),
            }),
        );
    }

    let user = user.unwrap();

    //Verifying password
    //Because osu sends it in md5, we need to hash it to md5 first and then handle it by bcrypt
    //Skipped in debug mode because we don't need to check it on local instance
    let password = body.password.unwrap();
    let password_md5 = md5::compute(password);
    let password_md5_display = format!("{:x}", password_md5);
    match bcrypt::verify(password_md5_display.as_bytes(), &user.password) {
        Err(_) | Ok(false) => {
            #[cfg(not(debug_assertions))]
            return (
                StatusCode::BAD_REQUEST,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    error: Some("invalid_credentials".to_string()),
                    hint: Some("username".to_string()),
                    message: Some("Invalid credentials.".to_string()),
                    token_type: None,
                }),
            );
        }
        Ok(true) => (),
    }

    //Fun fact: I skill issued myself
    let exp = chrono::Utc::now().timestamp() + 3600;

    let mut hmac: hmac::digest::core_api::CoreWrapper<hmac::HmacCore<Sha256>> =
        Hmac::new_from_slice(context.config.token_hmac_secret.as_bytes()).unwrap();

    hmac.update(user.password.as_bytes());

    let result = hmac.finalize();
    let result = result.into_bytes();
    let result = result.to_vec();

    let result = STANDARD_NO_PAD.encode(result);
    let hmac: hmac::digest::core_api::CoreWrapper<hmac::HmacCore<Sha256>> =
        Hmac::new_from_slice(context.config.token_hmac_secret.as_bytes()).unwrap();

    let access_token_claims = TokenClaim {
        exp,
        sub: user.id,
        iat: chrono::Utc::now().timestamp(),
        hash: result.clone(),
    };

    let access_token_claims = Token::new(
        Header {
            algorithm: jwt::AlgorithmType::Hs256,
            ..Default::default()
        },
        access_token_claims,
    )
    .sign_with_key(&hmac);

    let access_token_claims = match access_token_claims {
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    token_type: None,
                    error: Some("internal_error".to_string()),
                    hint: Some("internal_error".to_string()),
                    message: Some(error.to_string()),
                }),
            );
        }
        Ok(claims) => claims,
    };

    let refresh_token_exp = chrono::Utc::now().add(chrono::Days::new(14)).timestamp();

    let refresh_token_claims = TokenClaim {
        exp: refresh_token_exp,
        sub: user.id,
        iat: chrono::Utc::now().timestamp(),
        hash: result,
    };

    let refresh_token_claims = Token::new(
        Header {
            algorithm: jwt::AlgorithmType::Hs256,
            ..Default::default()
        },
        refresh_token_claims,
    )
    .sign_with_key(&hmac);

    let refresh_token_claims = match refresh_token_claims {
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AccessTokenResponse {
                    access_token: None,
                    refresh_token: None,
                    expires_in: None,
                    token_type: None,
                    error: Some("internal_error".to_string()),
                    hint: Some("internal_error".to_string()),
                    message: Some(error.to_string()),
                }),
            );
        }
        Ok(claims) => claims,
    };

    (
        StatusCode::OK,
        Json(AccessTokenResponse {
            access_token: Some(access_token_claims.as_str().to_string()),
            refresh_token: Some(refresh_token_claims.as_str().to_string()),
            expires_in: Some(3600),
            token_type: Some("Bearer".to_string()),
            error: None,
            hint: None,
            message: None,
        }),
    )
}
