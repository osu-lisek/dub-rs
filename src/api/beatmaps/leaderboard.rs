use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use tracing::error;

use crate::{
    api::{users::users::PublicScore, FailableResponse},
    context::Context,
    utils::{
        http_utils::OsuMode, score_utils::get_user_scores_on_beatmap,
        user_utils::find_user_by_id_or_username,
    },
};

use super::BeatmapLeaderboardRequestQuery;

pub async fn get_beatmap_leaderboard(
    Path(id): Path<String>,
    Query(params): Query<BeatmapLeaderboardRequestQuery>,
    Extension(ctx): Extension<Arc<Context>>,
) -> (StatusCode, Json<FailableResponse<Vec<PublicScore>>>) {
    let user = params.user.unwrap_or("0".to_string());
    let user = find_user_by_id_or_username(&ctx.pool, user).await;

    if let Err(e) = user {
        error!("{:#?}", e);
        return (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some("User not found.".to_string()),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    if let None = user {
        return (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some("User not found.".to_string()),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    let scores = get_user_scores_on_beatmap(
        &ctx.pool,
        user.id,
        id.parse::<i32>().unwrap_or(0),
        params.mode.unwrap_or(OsuMode::Osu),
    )
    .await;

    if let Err(e) = scores {
        error!("{:#?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some("Internal server error.".to_string()),
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
            data: Some(scores.iter().map(|x| x.publish()).collect()),
        }),
    )
}
