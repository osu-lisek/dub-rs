use std::sync::Arc;

use axum::{extract::Query, http::StatusCode, Extension, Json};

use crate::{
    api::FailableResponse,
    context::Context,
    utils::user_utils::{find_user_by_id_or_username, increment_user_coins},
};

use super::GetBackQuery;

pub async fn handle_vote(
    Extension(ctx): Extension<Arc<Context>>,
    Query(query): Query<GetBackQuery>,
) -> (StatusCode, Json<FailableResponse<bool>>) {
    let key = ctx.config.listing_key.clone();

    if key.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some("No listing key set".to_string()),
                data: None,
            }),
        );
    }

    let key = key.unwrap();

    if key != query.key {
        return (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some("Invalid key".to_string()),
                data: None,
            }),
        );
    };

    let user = find_user_by_id_or_username(&ctx.pool, query.username).await;

    if user.is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some("Failed to find user".to_string()),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    if user.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some("Invalid username".to_string()),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    increment_user_coins(&ctx.pool, &user.id, 50).await;

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(true),
        }),
    )
}
