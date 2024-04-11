use axum::{routing::get, Router};
use serde::Deserialize;

use crate::utils::http_utils::OsuMode;

use self::leaderboard::get_beatmap_leaderboard;

pub mod leaderboard;
pub mod performance;

#[derive(Debug, Deserialize)]
pub struct BeatmapLeaderboardRequestQuery {
    pub user: Option<String>,
    pub mode: Option<OsuMode>,
}

pub fn router() -> Router {
    Router::new().route("/:id/leaderboard", get(get_beatmap_leaderboard))
}
