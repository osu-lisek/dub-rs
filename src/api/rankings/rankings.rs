use std::sync::Arc;

use axum::{extract::Query, http::StatusCode, Extension, Json};
use axum_macros::debug_handler;

use crate::{
    api::FailableResponse,
    context::Context,
    utils::{
        general_utils::to_fixed,
        user_utils::{
            calculate_level, get_leaderboard, get_leaderboard_count, get_users_many,
            get_usersats_many,
        },
    },
};

use super::{RankingsEntry, RankingsRequestQuery, RankingsResponse, RankingsUser};

#[debug_handler]
pub async fn leaderboard(
    Extension(ctx): Extension<Arc<Context>>,
    Query(query): Query<RankingsRequestQuery>,
) -> (StatusCode, Json<FailableResponse<RankingsResponse>>) {
    let leaderboard = get_leaderboard(
        &ctx.redis,
        query.clone().mode,
        query.clone().offset,
        query.clone().limit,
    )
    .await;

    let total_users = get_leaderboard_count(&ctx.redis, query.clone().mode).await;
    let stats = get_usersats_many(&ctx.pool, &leaderboard, query.clone().mode).await;
    let users = get_users_many(&ctx.pool, &leaderboard).await;

    let mut result = Vec::new();

    let mut index = query.offset.unwrap_or(0) + 1;
    for user_id in leaderboard {
        let user = users.iter().find(|u| u.id == user_id);
        if user.is_none() {
            continue;
        }
        let user = user.unwrap();
        let stats = stats.iter().find(|u| u.id.unwrap_or(0) == user_id).unwrap();

        result.push(RankingsEntry {
            place: index,
            user: RankingsUser {
                id: stats.id.unwrap_or(0),
                accuracy: to_fixed(stats.accuracy * 100.0, 2),
                country: user.country.clone(),
                is_donor: false,
                level: calculate_level(stats.ranked_score),
                performance: stats.performance as i16,
                username: user.username.clone(),
                playcount: stats.playcount,
                ranked_score: stats.ranked_score,
            },
        });

        index += 1;
    }
    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(RankingsResponse {
                entries: result,
                total_users,
            }),
        }),
    )
}
