use std::{fmt::Debug, sync::Arc};

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};

use serde::Deserialize;
use tracing::info;

use crate::{
    api::FailableResponse,
    context::Context,
    db::user::User,
    utils::{
        http_utils::OsuMode,
        score_utils::{get_user_best_scores, get_user_recent_scores, SortMode},
        user_utils::{find_user_by_id_or_username, is_restricted},
    },
};

use super::users::PublicScore;

#[derive(Debug, Deserialize)]
pub struct ScoresRequestQuery {
    pub mode: Option<OsuMode>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort: Option<SortMode>,
}

pub async fn get_best_scores(
    Extension(ctx): Extension<Arc<Context>>,
    Query(query): Query<ScoresRequestQuery>,
    Extension(current_user): Extension<Option<User>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<FailableResponse<Vec<PublicScore>>>) {
    let user = find_user_by_id_or_username(&ctx.pool, id).await;

    if let Err(error) = user {
        info!("Error getting user: {:?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Internal server error.")),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    if let None = user {
        return (
            StatusCode::NOT_FOUND,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Not found")),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    if is_restricted(&user).await {
        let mut is_admin = false;

        if let Some(u) = current_user {
            is_admin = u.permissions & 1 > 0;

            if u.id == user.id {
                is_admin = true;
            }
        }

        if !is_admin {
            return (
                StatusCode::FORBIDDEN,
                Json(FailableResponse {
                    ok: false,
                    message: Some(String::from("This profile is unaccessable.")),
                    data: None,
                }),
            );
        }
    }

    let scores = get_user_best_scores(
        &ctx.pool,
        &user,
        query.limit,
        query.offset,
        query.mode.unwrap_or(OsuMode::Osu),
        query.sort,
    )
    .await;

    if let Err(error) = scores {
        info!("Error getting scores: {:?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Internal server error.")),
                data: None,
            }),
        );
    }

    let scores = scores.unwrap();

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(scores.iter().map(|record| record.publish()).collect()),
        }),
    )
}

pub async fn get_recent_scores(
    Extension(ctx): Extension<Arc<Context>>,
    Extension(current_user): Extension<Option<User>>,
    Query(query): Query<ScoresRequestQuery>,
    Path(id): Path<String>,
) -> (StatusCode, Json<FailableResponse<Vec<PublicScore>>>) {
    let user = find_user_by_id_or_username(&ctx.pool, id).await;

    if let Err(error) = user {
        info!("Error getting user: {:?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Internal server error.")),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    if let None = user {
        return (
            StatusCode::NOT_FOUND,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Not found")),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    if is_restricted(&user).await {
        let mut is_admin = false;

        if let Some(u) = current_user {
            is_admin = u.permissions & 1 > 0;

            if u.id == user.id {
                is_admin = true;
            }
        }

        if !is_admin {
            return (
                StatusCode::FORBIDDEN,
                Json(FailableResponse {
                    ok: false,
                    message: Some(String::from("This profile is unaccessable.")),
                    data: None,
                }),
            );
        }
    }
    let scores = get_user_recent_scores(&ctx.pool, &user, query.limit, query.offset).await;

    if let Err(error) = scores {
        info!("Error getting scores: {:?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Internal server error.")),
                data: None,
            }),
        );
    }

    let scores = scores.unwrap();

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(scores.iter().map(|record| record.publish()).collect()),
        }),
    )
}
