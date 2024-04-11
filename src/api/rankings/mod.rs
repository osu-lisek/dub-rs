use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};

use crate::utils::http_utils::OsuMode;

use self::rankings::leaderboard;

pub mod rankings;

#[derive(Debug, Serialize)]
pub struct RankingsUser {
    pub id: i32,
    pub username: String,
    pub country: String,
    pub performance: i16,
    pub accuracy: f64,
    pub playcount: i32,
    pub ranked_score: i64,
    pub level: i64,
    pub is_donor: bool,
}

#[derive(Debug, Serialize)]
pub struct RankingsEntry {
    pub place: i32,
    pub user: RankingsUser,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RankingsRequestQuery {
    pub mode: Option<OsuMode>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
    pub country: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RankingsResponse {
    pub entries: Vec<RankingsEntry>,
    pub total_users: i32,
}

pub fn router() -> Router {
    Router::new().route("/leaderboard", get(leaderboard))
}
