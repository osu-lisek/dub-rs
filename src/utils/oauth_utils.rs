use sqlx::{prelude::FromRow, Pool, Postgres};

use super::score_utils::OsuServerError;

#[derive(Debug, FromRow)]
pub struct App {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub secret: String,
    #[sqlx(rename = "redirectUrl")]
    pub redirect_uri: String,
    #[sqlx(rename = "ownerId")]
    pub owner_id: i32,
    #[sqlx(rename = "iconHash")]
    pub icon_hash: Option<String>,
    #[sqlx(rename = "allowedGrantTypes")]
    pub allowed_grant_type: Vec<String>,
}

pub async fn get_app_by_id(
    connection: &Pool<Postgres>,
    id: i32,
) -> Result<Option<App>, OsuServerError> {
    let result = sqlx::query_as::<_, App>(r#"SELECT * FROM "OAuthApplication" WHERE id = $1"#)
        .bind(id)
        .fetch_optional(connection)
        .await;

    match result {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(OsuServerError::Internal(error.to_string())),
        },
        Ok(app) => Ok(app),
    }
}
