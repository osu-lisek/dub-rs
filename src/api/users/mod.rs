use axum::{
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::utils::{http_utils::OsuMode, Badge, UserDbStats};

pub mod avatar;
pub mod friends;
pub mod scores;
pub mod security;
pub mod users;

#[derive(Debug, Serialize)]
pub struct Leveling {
    pub level: i64,
    pub progress: i64,
}

#[derive(Debug, Serialize)]
pub struct UserRankings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct Grades {
    pub xh: i32,
    pub x: i32,
    pub sh: i32,
    pub s: i32,
    pub a: i32,
}

#[derive(Debug, Serialize)]
pub struct PublicUserProfile {
    pub username: String,
    pub id: i32,
    pub flags: i32,
    pub permissions: i32,
    pub stats: UserDbStats,
    pub country: String,
    pub rankings: UserRankings,
    pub username_history: Vec<String>,
    pub created_at: chrono::NaiveDateTime,
    pub last_seen: chrono::NaiveDateTime,
    pub badges: Vec<Badge>,
    pub is_donor: bool,
    pub background_url: Option<String>,
    pub leveling: Leveling,
    pub grades: Grades,
    pub userpage_content: String,
    pub followers: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coins: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_friend: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mutual: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UserRequestQuery {
    pub mode: Option<OsuMode>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Friend {
    pub id: i32,
    pub username: String,
    pub is_mutual: bool,
    pub is_online: bool,
    pub banner: Option<String>,
    pub country: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/v2/users/:id", get(crate::api::users::users::get_user))
        .route(
            "/api/v2/users/:id/best",
            get(crate::api::users::scores::get_best_scores),
        )
        .route(
            "/api/v2/users/:id/recent",
            get(crate::api::users::scores::get_recent_scores),
        )
        .route(
            "/api/v2/users/:id/violations",
            get(crate::api::users::security::get_user_account_standing),
        )
        .route(
            "/api/v2/users/:id/graph",
            get(crate::api::users::users::get_user_graph),
        )
        .route(
            "/api/v2/users/:id/friends",
            get(crate::api::users::friends::get_friends),
        )
        .route(
            "/api/v2/users/:id/friend",
            post(crate::api::users::friends::change_friend_status),
        )
        .route(
            "/api/v2/users/:id/followers",
            get(crate::api::users::friends::get_followers),
        )
        .route(
            "/api/v2/users/:id/avatar",
            post(crate::api::users::avatar::upload_avatar),
        )
}
