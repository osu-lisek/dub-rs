use sqlx::prelude::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,

    #[sqlx(rename = "usernameSafe")]
    pub username_safe: String,
    #[sqlx(rename = "backgroundUrl")]
    pub background_url: Option<String>,
    pub password: String,
    pub country: String,
    pub permissions: i32,
    pub flags: i32,
    #[sqlx(rename = "oldUsernames")]
    pub username_history: Option<Vec<String>>,
    #[sqlx(rename = "lastSeen")]
    pub last_seen: sqlx::types::chrono::NaiveDateTime,
    #[sqlx(rename = "createdAt")]
    pub created_at: sqlx::types::chrono::NaiveDateTime,
    #[sqlx(rename = "donorUntil")]
    pub donor_until: Option<sqlx::types::chrono::NaiveDateTime>,
    #[sqlx(rename = "userpageContent")]
    pub userpage_content: String,
    #[sqlx(rename = "coins")]
    pub coins: i32,
}
