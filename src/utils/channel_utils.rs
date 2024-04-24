use sqlx::{prelude::FromRow, Pool, Postgres};

use super::score_utils::OsuServerError;

#[derive(FromRow, Debug)]
pub struct DatabaseChannel {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub channel_type: String,
}

#[derive(sqlx::Type, Debug, PartialEq)]
pub enum ChannelType {
    #[sqlx(rename = "public")]
    Public,
    #[sqlx(rename = "private")]
    Private,
    #[sqlx(rename = "multi")]
    Multi,
}

pub async fn fetch_channels(
    connection: &Pool<Postgres>,
) -> Result<Vec<DatabaseChannel>, OsuServerError> {
    let rows = sqlx::query(
        r#"
SELECT
    "Channel"."id",
    "Channel"."name",
    "Channel"."description",
    "Channel"."channel_type"
FROM
    "Channel"
"#,
    )
    .fetch_all(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(vec![]),
            error => Err(OsuServerError::Internal(format!(
                "Error while fetching channels: {}",
                error
            ))),
        },
        Ok(rows) => {
            let mut channels = Vec::new();

            for row in rows {
                channels.push(DatabaseChannel::from_row(&row).unwrap());
            }

            Ok(channels)
        }
    }
}
