use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    api::FailableResponse,
    context::Context,
    db::user::User,
    utils::{
        beatmap_utils::PublicBeatmap,
        http_utils::OsuMode,
        score_utils::{get_user_grades_count, SortMode},
        user_utils::{
            calculate_level, calculate_level_progress, find_user_by_id_or_username,
            get_country_rank, get_rank, get_user_badges, get_user_followers, get_user_graph_data,
            get_user_stats, is_restricted, is_user_friend, is_user_mutual,
        },
        GraphEntry,
    },
};

use super::{Grades, Leveling, PublicUserProfile, UserRankings, UserRequestQuery};

#[allow(unused_assignments)]
pub async fn get_user(
    Extension(ctx): Extension<Arc<Context>>,
    Extension(current_user): Extension<Option<User>>,
    Query(query): Query<UserRequestQuery>,
    Path(id): Path<String>,
) -> (StatusCode, Json<FailableResponse<PublicUserProfile>>) {
    let mode = query.mode.unwrap_or(OsuMode::Osu);
    let mut user: Option<User> = None;

    if id == "@me" && current_user.clone().is_some() {
        user = current_user.clone();
    } else {
        let fetched_user = find_user_by_id_or_username(&ctx.pool, id).await;

        if let Err(error) = fetched_user {
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

        user = fetched_user.unwrap();
    }

    if user.is_none() {
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

        if let Some(u) = current_user.clone() {
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

    let stats = get_user_stats(&ctx.pool, &user.id, &mode).await;

    if let Err(error) = stats {
        info!("Error getting user stats: {:?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Internal server error.")),
                data: None,
            }),
        );
    }

    let stats = stats.unwrap();

    let global_rank = get_rank(&ctx.redis, &user, &mode).await;
    let country_rank = get_country_rank(&ctx.redis, &user, &mode).await;

    let badges = get_user_badges(&ctx.pool, &user).await;
    if let Err(error) = badges {
        error!("{:#?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Internal server error.")),
                data: None,
            }),
        );
    }

    let badges = badges.unwrap();

    let level_progress = calculate_level_progress(&stats).await;
    let level = calculate_level(stats.total_score);

    let grades_xh =
        get_user_grades_count(&ctx.pool, &ctx.redis, &user, &mode, None, "XH".to_string())
            .await
            .unwrap_or(0);
    let grades_x =
        get_user_grades_count(&ctx.pool, &ctx.redis, &user, &mode, None, "X".to_string())
            .await
            .unwrap_or(0);
    let grades_sh =
        get_user_grades_count(&ctx.pool, &ctx.redis, &user, &mode, None, "SH".to_string())
            .await
            .unwrap_or(0);
    let grades_s =
        get_user_grades_count(&ctx.pool, &ctx.redis, &user, &mode, None, "S".to_string())
            .await
            .unwrap_or(0);
    let grades_a =
        get_user_grades_count(&ctx.pool, &ctx.redis, &user, &mode, None, "A".to_string())
            .await
            .unwrap_or(0);

    let coins: Option<i32> = current_user
        .clone()
        .map(|u| {
            if u.id == user.id {
                return Some(u.coins);
            }

            None
        })
        .unwrap_or(None);

    let mut is_friend: Option<bool> = None;
    let mut is_mutual_friend: Option<bool> = None;

    if let Some(u) = current_user {
        is_friend = Some(is_user_friend(&ctx.pool, &u.id, &user.id).await);
        is_mutual_friend = Some(
            is_user_mutual(&ctx.pool, &u.id, &user.id)
                .await
                .unwrap_or(false),
        );
    }

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(PublicUserProfile {
                username: user.username,
                id: user.id,
                stats,
                country: user.country,
                rankings: UserRankings {
                    global: global_rank,
                    country: country_rank,
                },
                username_history: user.username_history.unwrap_or_default(),
                flags: user.flags,
                permissions: user.permissions,
                created_at: user.created_at,
                last_seen: user.last_seen,
                badges,
                is_donor: user
                    .donor_until
                    .unwrap_or(NaiveDateTime::UNIX_EPOCH)
                    .timestamp()
                    > Utc::now().timestamp(),
                background_url: user.background_url,
                leveling: Leveling {
                    progress: level_progress.unwrap_or(0),
                    level,
                },
                grades: Grades {
                    xh: grades_xh,
                    sh: grades_sh,
                    s: grades_s,
                    a: grades_a,
                    x: grades_x,
                },
                userpage_content: user.userpage_content,
                followers: get_user_followers(&ctx.pool, &user.id)
                    .await
                    .unwrap_or(Vec::new())
                    .len() as i64,
                is_mutual: is_mutual_friend,
                coins,
                is_friend,
            }),
        }),
    )
}

pub async fn get_user_graph(
    Extension(ctx): Extension<Arc<Context>>,
    Extension(current_user): Extension<Option<User>>,
    Query(query): Query<UserRequestQuery>,
    Path(id): Path<String>,
) -> (StatusCode, Json<FailableResponse<Vec<GraphEntry>>>) {
    let mode = query.mode.unwrap_or(OsuMode::Osu);
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

    if user.is_none() {
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

    let graph = get_user_graph_data(&ctx.pool, &user.id, &mode, None).await;

    if let Err(error) = graph {
        info!("Error getting user graph: {:?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Internal server error.")),
                data: None,
            }),
        );
    }

    let graph = graph.unwrap();

    (
        StatusCode::NOT_FOUND,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(graph),
        }),
    )
}

#[derive(Debug, Serialize)]
pub struct PublicScore {
    pub id: i32,
    pub beatmap: PublicBeatmap,
    pub user_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<PublicUserProfile>,
    pub accuracy: f64,
    pub count300: i32,
    pub count100: i32,
    pub count50: i32,
    pub count_geki: i32,
    pub count_katu: i32,
    pub count_miss: i32,
    pub total_score: i32,
    pub grade: String,
    pub playmode: OsuMode,
    pub max_combo: i32,
    pub mods: i32, //TODO: Make array of strings
    pub weighted: f32,
    pub performance: f64,
    pub submitted_at: NaiveDateTime,
    pub passed: bool,
}

#[derive(Debug, Deserialize)]
pub struct ScoresRequestQuery {
    pub mode: Option<OsuMode>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort: Option<SortMode>,
}
