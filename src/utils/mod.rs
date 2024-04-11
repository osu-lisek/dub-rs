use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::prelude::FromRow;

use crate::db::user::User;

pub mod beatmap_utils;
pub mod channel_utils;
pub mod chart;
pub mod general_utils;
pub mod http_utils;
pub mod ip_utils;
pub mod oauth_utils;
pub mod performance_utils;
pub mod score_utils;
pub mod user_utils;

//ranked, total, accuracy, playcount, rank, pp
#[derive(FromRow, Debug, Clone, Serialize)]
pub struct UserDbStats {
    pub id: Option<i32>,
    pub ranked_score: i64,
    pub total_score: i64,
    pub accuracy: f64,
    pub playcount: i32,
    pub performance: f64,
    pub max_combo: i32,
}

impl Default for UserDbStats {
    fn default() -> Self {
        Self {
            id: None,
            ranked_score: 0,
            total_score: 0,
            accuracy: 0.0,
            playcount: 0,
            performance: 0.0,
            max_combo: 0,
        }
    }
}

#[derive(FromRow, Debug, Serialize)]
pub struct Badge {
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub color: String,
}

#[derive(Debug, FromRow)]
pub struct DatabaseHwid {
    pub id: i32,
    #[sqlx(rename = "userId")]
    pub user_id: i32,
    pub mac: String,
    #[sqlx(rename = "uniqueId")]
    pub unique_id: String,
    #[sqlx(rename = "diskId")]
    pub disk_id: String,
}

#[derive(Debug)]
pub struct UserHwid {
    pub hwid: DatabaseHwid,
    pub user: User,
}

#[derive(Debug, FromRow)]
pub struct Punishment {
    pub id: String,
    pub date: NaiveDateTime,
    #[sqlx(rename = "appliedBy")]
    pub applied_by: i32,
    #[sqlx(rename = "appliedTo")]
    pub applied_to: i32,
    #[sqlx(rename = "punishmentType")]
    pub punishment_type: String,
    #[sqlx(rename = "level")]
    pub level: String,
    #[sqlx(rename = "expires")]
    pub expires: bool,
    #[sqlx(rename = "expiresAt")]
    pub expires_at: Option<NaiveDateTime>,
    #[sqlx(rename = "note")]
    pub note: String,
}

#[derive(Debug, FromRow)]
pub struct RelationShip {
    #[sqlx(rename = "friendId")]
    pub friend_id: i32,
    #[sqlx(rename = "username")]
    pub username: String,
    #[sqlx(rename = "is_mutual")]
    pub is_mutual: bool,
    #[sqlx(rename = "banner")]
    pub banner: Option<String>,
    #[sqlx(rename = "country")]
    pub country: String,
}

#[derive(Debug)]
pub struct UserRelationShip {
    pub friend_id: i32,
    pub is_mutual: bool,
    pub username: String,
    pub banner: Option<String>,
    pub country: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct GraphEntry {
    pub id: i32,
    pub date: NaiveDateTime,
    #[sqlx(rename = "rank")]
    pub rank: i32,
}

#[derive(Debug, FromRow)]
pub struct RedisRankingsEntry {}
