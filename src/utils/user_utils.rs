use std::time::Duration;

use bcrypt::verify;
use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use redis::Commands;
use serde_json::json;
use sqlx::{prelude::FromRow, Pool, Postgres, Row};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use webhook::client::WebhookClient;

use crate::{bancho::client::HWID, config::RunConfiguration, db::user::User};

use super::{
    general_utils::to_fixed,
    http_utils::OsuMode,
    score_utils::{OsuServerError, Score},
    Badge, DatabaseHwid, GraphEntry, Punishment, RelationShip, UserDbStats, UserHwid,
    UserRelationShip,
};

pub fn to_safe(name: impl ToString) -> String {
    name.to_string().to_lowercase().replace(' ', "_")
}

//Ignoring dead code and unsued variables
#[allow(unused_variables)]
#[allow(unreachable_code)]
pub async fn validate_auth(
    redis: &redis::Client,
    connection: &Pool<Postgres>,
    user: impl ToString,
    password: impl ToString,
) -> bool {
    #[cfg(debug_assertions)]
    return true;

    let redis_connection = redis.get_connection();

    if let Err(error) = redis_connection {
        error!("Failed to get redis connection: {}", error);
        return false;
    }

    let mut redis_connection = redis_connection.unwrap();
    let cached_password: Option<String> = redis_connection
        .get(format!(
            "user:{}:password:{}",
            to_safe(user.to_string()),
            password.to_string()
        ))
        .unwrap();

    let user_id = get_user_id(redis, connection, user.to_string()).await;

    if user_id.is_none() {
        return false;
    }

    let user_id = user_id.unwrap();

    let user = get_user_by_id(connection, user_id).await;

    if let Err(error) = user {
        error!("Failed to fetch user: {:#?}", error);
        return false;
    }

    let user = user.unwrap();

    if user.is_none() {
        return false;
    }
    let user = user.unwrap();

    if let Some(cached_pass) = cached_password {
        return user.password == cached_pass;
    }

    let verification_result = verify(password.to_string(), &user.password);

    match verification_result {
        Ok(result) => {
            if result {
                let _: () = redis_connection
                    .set(
                        format!(
                            "user:{}:password:{}",
                            to_safe(user.username_safe),
                            password.to_string()
                        ),
                        user.password.to_string(),
                    )
                    .expect("Failed to save password to cache.");

                info!("cached: {}, real: {} [2]", password.to_string(), 0);
            }
            result
        }
        Err(_) => false,
    }
}

pub async fn get_user_id(
    redis: &redis::Client,
    connection: &Pool<Postgres>,
    username: impl ToString,
) -> Option<i32> {
    let redis_connection = redis.get_connection();

    if let Err(error) = redis_connection {
        error!("Failed to get redis connection: {}", error);
        return None;
    }

    let mut redis_connection = redis_connection.unwrap();
    let cached_id: Option<i32> = redis_connection
        .get(format!("user:{}:id", to_safe(username.to_string())))
        .unwrap_or_default();

    if let Some(cached_id) = cached_id {
        return Some(cached_id);
    }

    let user = sqlx::query("SELECT * FROM \"User\" WHERE \"usernameSafe\"=$1")
        .bind(to_safe(username))
        .fetch_one(connection)
        .await;

    match user {
        Err(error) => {
            info!("Failed to fetch user: {}", error);
            None
        }
        Ok(user) => {
            let user = User::from_row(&user).unwrap();
            let _: Result<i32, redis::RedisError> =
                redis_connection.set(format!("user:{}:id", to_safe(user.username_safe)), user.id);

            Some(user.id)
        }
    }
}

pub async fn get_user_by_id(
    connection: &Pool<Postgres>,
    id: i32,
) -> Result<Option<User>, OsuServerError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM \"User\" WHERE \"id\"=$1")
        .bind(id)
        .fetch_one(connection)
        .await;

    match user {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            error => Err(OsuServerError::Internal(format!(
                "Failed to fetch user: {}",
                error
            ))),
        },
        Ok(user) => Ok(Some(user)),
    }
}

pub async fn find_user_by_id_or_username(
    connection: &Pool<Postgres>,
    term: String,
) -> Result<Option<User>, OsuServerError> {
    let user = sqlx::query(
        r#"
SELECT
    *
FROM
    "User"
WHERE
    "id"=$1
    OR
    "usernameSafe"=$2
"#,
    )
    .bind(term.parse::<i32>().unwrap_or(0))
    .bind(to_safe(term))
    .fetch_one(connection)
    .await;

    match user {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            error => Err(OsuServerError::Internal(format!(
                "Failed to fetch user: {}",
                error
            ))),
        },
        Ok(user) => Ok(Some(User::from_row(&user).unwrap())),
    }
}

pub async fn find_hwids(
    connection: &Pool<Postgres>,
    hwid: &HWID,
) -> Result<Vec<UserHwid>, OsuServerError> {
    let rows = sqlx::query(
        r#"
SELECT
    "Hwid".*, "User".*
FROM
    "Hwid"
JOIN
    "User" ON "Hwid"."userId" = "User"."id"
WHERE
    "Hwid"."mac" = $1 OR "Hwid"."mac" = $2 OR
    "Hwid"."uniqueId" = $3 OR "Hwid"."uniqueId" = $1 OR
    "Hwid"."diskId" = $4 OR "Hwid"."diskId" = $3
"#,
    )
    .bind(&hwid.uid)
    .bind(&hwid.mac)
    .bind(&hwid.disk)
    .bind(&hwid.plain)
    .fetch_all(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => Err(OsuServerError::Internal(format!(
                "Failed to fetch hwids: {}",
                error
            ))),
        },
        Ok(rows) => {
            let mut result = Vec::new();
            for row in rows {
                let user = User::from_row(&row);
                let hwid = DatabaseHwid::from_row(&row);

                if let Err(error) = user {
                    return Err(OsuServerError::Internal(format!(
                        "Failed to fetch user: {}",
                        error
                    )));
                }

                let user = user.unwrap();

                if let Err(error) = hwid {
                    return Err(OsuServerError::Internal(format!(
                        "Failed to fetch hwid: {}",
                        error
                    )));
                }

                let hwid = hwid.unwrap();

                result.push(UserHwid { hwid, user });
            }

            Ok(result)
        }
    }
}

pub async fn update_user_country(connection: &Pool<Postgres>, user_id: i32, country: String) {
    sqlx::query("UPDATE \"User\" SET \"country\"=$1 WHERE \"id\"=$2")
        .bind(country)
        .bind(user_id)
        .execute(connection)
        .await
        .unwrap_or_default();
}

pub async fn get_user_stats(
    connection: &Pool<Postgres>,
    user_id: &i32,
    mode: &OsuMode,
) -> Result<UserDbStats, OsuServerError> {
    let result = sqlx::query(
        format!(
            r#"
SELECT
    "UserStats"."userId" as id,
    "UserStats"."pp{0}" as performance,
    "UserStats"."rankedScore{0}" as ranked_score,
    "UserStats"."totalScore{0}" as total_score,
    "UserStats"."avgAcc{0}" as accuracy,
    "UserStats"."playCount{0}" as playcount,
    "UserStats"."maxCombo{0}" as max_combo
FROM
    "UserStats"
WHERE
    "UserStats"."userId" = $1
"#,
            mode.to_db_suffix()
        )
        .as_str(),
    )
    .bind(user_id)
    .fetch_one(connection)
    .await;

    match result {
        Err(error) => Err(OsuServerError::Internal(format!(
            "Error while fetching stats: {}",
            error
        ))),
        Ok(stats) => Ok(UserDbStats::from_row(&stats).unwrap()),
    }
}

pub async fn is_restricted(user: &User) -> bool {
    (user.permissions & 8) > 0 && (user.flags & 32) == 0
}

pub fn is_pending_verification(user: &User) -> bool {
    user.permissions & 8 > 0 && user.flags & 32 > 0
}

pub fn is_verified(user: &User) -> bool {
    user.flags & 2 > 0
}

pub async fn get_country_rank(redis: &redis::Client, user: &User, mode: &OsuMode) -> Option<i32> {
    let redis = redis.get_connection();

    if let Err(error) = redis {
        error!("Failed to get redis connection: {}", error);
        return None;
    }

    let mut redis = redis.unwrap();
    let response: Result<Option<i32>, redis::RedisError> = redis.zrevrank(
        format!("leaderboard:{}:performance:{}", mode.to_osu(), user.country),
        user.id,
    );

    if let Ok(rank) = response {
        match rank {
            Some(rank) => {
                return Some(rank + 1);
            }
            None => {
                return None;
            }
        }
    }

    None
}

pub async fn get_rank(redis: &redis::Client, user: &User, mode: &OsuMode) -> Option<i32> {
    let redis = redis.get_connection();

    if let Err(error) = redis {
        error!("Failed to get redis connection: {}", error);
        return None;
    }

    let mut redis = redis.unwrap();
    let response: Result<Option<i32>, redis::RedisError> = redis.zrevrank(
        format!("leaderboard:{}:performance", mode.to_osu()),
        user.id,
    );

    if let Ok(rank) = response {
        match rank {
            Some(rank) => {
                return Some(rank + 1);
            }
            None => {
                return None;
            }
        }
    }

    None
}

pub async fn get_inactive_users(connection: &Pool<Postgres>) -> Result<Vec<User>, OsuServerError> {
    let users = sqlx::query(
        r#"
SELECT
    *
FROM
    "User"
WHERE
    flags & 4 > 0 OR
    "lastSeen" < NOW() - INTERVAL '1 month'
    "#,
    )
    .fetch_all(connection)
    .await;

    match users {
        Err(error) => Err(OsuServerError::Internal(format!(
            "Error while fetching inactive users: {}",
            error
        ))),
        Ok(users) => {
            return Ok(users
                .iter()
                .map(|user| User::from_row(user).unwrap())
                .collect());
        }
    }
}

pub async fn get_restricted_users(
    connection: &Pool<Postgres>,
) -> Result<Vec<User>, OsuServerError> {
    let users = sqlx::query(
        r#"
SELECT
    *
FROM
    "User"
WHERE
    permissions & 8 > 0
    "#,
    )
    .fetch_all(connection)
    .await;

    match users {
        Err(error) => Err(OsuServerError::Internal(format!(
            "Error while fetching inactive users: {}",
            error
        ))),
        Ok(users) => Ok(users
            .iter()
            .map(|user| User::from_row(user).unwrap())
            .collect()),
    }
}

pub async fn remove_ranking(redis: &redis::Client, user: &User) {
    let redis = redis.get_connection_with_timeout(Duration::from_secs(60));

    if let Err(error) = redis {
        error!("Failed to get redis connection: {}", error);
        return;
    }

    let mut redis = redis.unwrap();

    for mode in [
        OsuMode::Osu,
        OsuMode::Taiko,
        OsuMode::Fruits,
        OsuMode::Mania,
        OsuMode::Relax,
    ] {
        let result: Result<Option<i32>, redis::RedisError> = redis.zrem(
            format!("leaderboard:{}:performance", mode.to_osu()),
            user.id,
        );

        match result {
            Err(error) => {
                error!("Failed to remove ranking: {}", error);
            }
            Ok(_) => {
                info!(
                    "Removed ranking for user {} in mode {}",
                    user.id,
                    mode.to_osu()
                );
            }
        }

        let result: Result<Option<i32>, redis::RedisError> = redis.zrem(
            format!("leaderboard:{}:performance:{}", mode.to_osu(), user.country),
            user.id,
        );

        match result {
            Err(error) => {
                error!("Failed to remove ranking: {}", error);
            }
            Ok(_) => {
                info!(
                    "Removed ranking for user {} in mode {}",
                    user.id,
                    mode.to_osu()
                );
            }
        }
    }
}

pub async fn update_user_hwid(connection: &Pool<Postgres>, user: &User, hwid: &HWID) {
    sqlx::query(
        "UPDATE \"Hwid\" SET \"mac\"=$1, \"uniqueId\"=$2, \"diskId\"=$3 WHERE \"userId\"=$4",
    )
    .bind(&hwid.mac)
    .bind(&hwid.uid)
    .bind(&hwid.disk)
    .bind(user.id)
    .execute(connection)
    .await
    .unwrap_or_default();
}

pub async fn get_user_badges(
    connection: &Pool<Postgres>,
    user: &User,
) -> Result<Vec<Badge>, OsuServerError> {
    let badges = sqlx::query(
        r#"
SELECT
    *
FROM
    "UserBadge"
WHERE
    "userId" = $1
"#,
    )
    .bind(user.id)
    .fetch_all(connection)
    .await;

    match badges {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => Err(OsuServerError::Internal(format!(
                "Failed to fetch badges: {}",
                error
            ))),
        },
        Ok(badges) => {
            return Ok(badges
                .iter()
                .map(|badge| Badge::from_row(badge).unwrap())
                .collect());
        }
    }
}

pub fn calculate_score_from_level(level: i64) -> i64 {
    if level <= 100 {
        return calculate_score_less_than_100(level);
    }

    calculate_score_more_than_100(level)
}

pub async fn calculate_level_progress(stats: &UserDbStats) -> Result<i64, OsuServerError> {
    let current_level = calculate_level(stats.total_score);
    let next_level = calculate_score_from_level(current_level + 1);

    Ok((((stats.total_score as f64) / (next_level as f64)) * 100.0) as i64)
}

pub fn calculate_score_more_than_100(score: i64) -> i64 {
    26931190827 + 99999999999 * (score - 100)
}

pub fn calculate_score_less_than_100(score: i64) -> i64 {
    (5000 / 3) * (4 * score.pow(3) - 3 * score.pow(2) - score)
        + ((1.25 * (1.8_f32).powf((score as f32) - (60_f32))) as i64)
}

pub fn calculate_level(score: i64) -> i64 {
    if score <= calculate_score_less_than_100(100) {
        for i in 0..100 {
            if score <= calculate_score_less_than_100(i) {
                return i;
            }
        }
    }

    for i in 100..1000 {
        if score <= calculate_score_more_than_100(i) {
            return i;
        }
    }

    0
}

pub async fn get_user_recent_vilations(
    connection: &Pool<Postgres>,
    user: &User,
) -> Result<Vec<Punishment>, OsuServerError> {
    let rows = sqlx::query(
        r#"
SELECT
    *,
    "Punishment"."punishmentType" :: Text as "punishmentType",
    "Punishment"."level" :: Text as "level"
FROM
    "Punishment"
WHERE
    "appliedTo" = $1 AND
    ("date" < NOW() - INTERVAL '1 month' OR "expiresAt" IS NULL)
    "#,
    )
    .bind(user.id)
    .fetch_all(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => Err(OsuServerError::Internal(error.to_string())),
        },
        Ok(rows) => {
            let mut result = Vec::new();

            for row in rows {
                result.push(Punishment::from_row(&row).unwrap());
            }

            Ok(result)
        }
    }
}

pub async fn get_user_relationships(
    connection: &Pool<Postgres>,
    user_id: &i32,
) -> Result<Vec<UserRelationShip>, OsuServerError> {
    let rows = sqlx::query(
        r#"
            SELECT
            r1."userId",
            r1."friendId",
            CASE
              WHEN EXISTS (
                SELECT *
                FROM "RelationShips"
                WHERE "userId" = r1."friendId"
                  AND "friendId" = r1."userId"
              ) THEN TRUE
              ELSE FALSE
            END AS "is_mutual",
            r2."username" AS "username",
            r2."backgroundUrl" AS "banner",
            r2."country" AS "country"
          FROM
            "RelationShips" AS r1
          INNER JOIN "User" AS r2 ON r1."friendId" = r2."id"
          WHERE
            r1."userId" = $1;
    "#,
    )
    .bind(user_id)
    .fetch_all(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => Err(OsuServerError::Internal(error.to_string())),
        },
        Ok(rows) => {
            let mut relationships = Vec::new();

            for row in rows {
                relationships.push(RelationShip::from_row(&row).unwrap());
            }

            let mut result: Vec<UserRelationShip> = Vec::new();

            for realtionship in relationships {
                result.push(UserRelationShip {
                    friend_id: realtionship.friend_id,
                    is_mutual: realtionship.is_mutual,
                    username: realtionship.username,
                    banner: realtionship.banner,
                    country: realtionship.country,
                });
            }

            Ok(result)
        }
    }
}

pub async fn get_user_followers(
    connection: &Pool<Postgres>,
    user_id: &i32,
) -> Result<Vec<UserRelationShip>, OsuServerError> {
    let rows = sqlx::query(
        r#"
        SELECT
        r1."userId" as "friendId",
        CASE
          WHEN EXISTS (
            SELECT *
            FROM "RelationShips"
            WHERE "userId" = r1."friendId"
              AND "friendId" = r1."userId"
          ) THEN TRUE
          ELSE FALSE
        END AS "is_mutual",
        r2."username" AS "username",
        r2."backgroundUrl" AS "banner",
        r2."country" AS "country"
      FROM
        "RelationShips" AS r1
      INNER JOIN "User" AS r2 ON r1."userId" = r2."id"
      WHERE
        r1."friendId" = $1;
    "#,
    )
    .bind(user_id)
    .fetch_all(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => Err(OsuServerError::Internal(error.to_string())),
        },
        Ok(rows) => {
            let mut relationships = Vec::new();

            for row in rows {
                relationships.push(RelationShip::from_row(&row).unwrap());
            }

            let mut result: Vec<UserRelationShip> = Vec::new();

            for realtionship in relationships {
                result.push(UserRelationShip {
                    friend_id: realtionship.friend_id,
                    is_mutual: realtionship.is_mutual,
                    username: realtionship.username,
                    banner: realtionship.banner,
                    country: realtionship.country,
                });
            }

            Ok(result)
        }
    }
}

pub async fn is_user_mutual(
    connection: &Pool<Postgres>,
    user_id: &i32,
    friend_id: &i32,
) -> Result<bool, OsuServerError> {
    let rows = sqlx::query(
        r#"
        SELECT
        CASE
          WHEN (EXISTS (
            SELECT *
            FROM "RelationShips"
            WHERE "userId" = $2 AND "friendId" = $1
           )
            AND EXISTS (
            SELECT *
            FROM "RelationShips"
            WHERE "userId" = $1 AND "friendId" = $2
           ))
          THEN TRUE
          ELSE FALSE
        END AS "is_mutual"
    "#,
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_one(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(false),
            error => Err(OsuServerError::Internal(error.to_string())),
        },
        Ok(row) => Ok(row.try_get("is_mutual").unwrap_or(false)),
    }
}

pub async fn get_user_graph_data(
    connection: &Pool<Postgres>,
    user_id: &i32,
    mode: &OsuMode,
    limit: Option<i32>,
) -> Result<Vec<GraphEntry>, OsuServerError> {
    let rows = sqlx::query(
        r#"
SELECT
    MAX(id) as "id",
    MAX(date) as "date",
    MAX(rank) as "rank"
FROM
    public."GraphEntry"
WHERE
    "userId" = $1 AND "mode" = $2
GROUP BY
    DATE(date)
ORDER BY
    date DESC
LIMIT
    $3
    "#,
    )
    .bind(user_id)
    .bind(mode.to_osu())
    .bind(limit.unwrap_or(100))
    .fetch_all(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => Err(OsuServerError::Internal(error.to_string())),
        },
        Ok(rows) => {
            let mut result = Vec::new();

            for row in rows {
                result.push(GraphEntry::from_row(&row).unwrap());
            }

            Ok(result)
        }
    }
}

pub async fn increment_user_coins(connection: &Pool<Postgres>, user_id: &i32, coins: i32) {
    sqlx::query!(
        r#"
UPDATE
    "User"
SET
    "coins" = "coins" + $1
WHERE id = $2
"#,
        coins,
        user_id
    )
    .execute(connection)
    .await
    .unwrap_or_default();
}

pub async fn get_leaderboard(
    redis: &redis::Client,
    mode: Option<OsuMode>,
    offset: Option<i32>,
    limit: Option<i32>,
) -> Vec<i32> {
    let mut connection = redis
        .get_connection()
        .expect("Failed to open redis connection.");

    let resp: Result<Vec<i32>, redis::RedisError> = connection.zrevrange(
        format!(
            "leaderboard:{}:performance",
            mode.unwrap_or(OsuMode::Osu).to_osu()
        ),
        offset.unwrap_or(0) as isize,
        (limit.unwrap_or(50) - 1) as isize,
    );

    resp.unwrap_or_default()
}

pub async fn get_leaderboard_count(redis: &redis::Client, mode: Option<OsuMode>) -> i32 {
    let mut connection = redis
        .get_connection()
        .expect("Failed to open redis connection.");

    let resp: Result<i32, redis::RedisError> = connection.zcount(
        format!(
            "leaderboard:{}:performance",
            mode.unwrap_or(OsuMode::Osu).to_osu()
        ),
        "-inf",
        "+inf",
    );

    resp.unwrap_or(0)
}

pub async fn get_usersats_many(
    connection: &Pool<Postgres>,
    users: &[i32],
    mode: Option<OsuMode>,
) -> Vec<UserDbStats> {
    if users.is_empty() {
        return vec![];
    }

    let result = sqlx::query(
        format!(
            r#"
SELECT
    "UserStats"."pp{0}" as performance,
    "UserStats"."rankedScore{0}" as ranked_score,
    "UserStats"."totalScore{0}" as total_score,
    "UserStats"."avgAcc{0}" as accuracy,
    "UserStats"."playCount{0}" as playcount,
    "UserStats"."maxCombo{0}" as max_combo,
    "UserStats"."userId" as id
FROM
    "UserStats"
WHERE
    "UserStats"."userId" IN ({1})
"#, //I really don't want to use this macro, because it fuck ups everything
            mode.unwrap_or(OsuMode::Osu).to_db_suffix(),
            users
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .as_str(),
    )
    .fetch_all(connection)
    .await;

    match result {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Vec::new(),
            error => {
                panic!("Failed to fetch user stats: {}", error);
            }
        },
        Ok(rows) => {
            let mut result = Vec::new();

            for row in rows {
                result.push(UserDbStats::from_row(&row).unwrap());
            }

            result
        }
    }
}

pub async fn get_users_many(connection: &Pool<Postgres>, users: &[i32]) -> Vec<User> {
    if users.is_empty() {
        return vec![];
    }

    let result = sqlx::query(
        format!(
            r#"
SELECT
    *
FROM
    "User"
WHERE
    "User"."id" IN ({0})
"#, //I really don't want to use this macro, because it fuck ups everything
            users
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .as_str(),
    )
    .fetch_all(connection)
    .await;

    match result {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Vec::new(),
            error => {
                panic!("Failed to fetch user stats: {}", error);
            }
        },
        Ok(rows) => {
            let mut result = Vec::new();

            for row in rows {
                result.push(User::from_row(&row).unwrap());
            }

            result
        }
    }
}

pub fn is_user_manager(user: &User) -> bool {
    user.permissions & 1 > 0
}

pub async fn get_punishment_by_id(connection: &Pool<Postgres>, id: String) -> Option<Punishment> {
    sqlx::query_as(r#"SELECT * FROM "Punishment" WHERE id = $1 "#)
        .bind(id)
        .fetch_optional(connection)
        .await
        .unwrap_or_default()
}

pub async fn restrict_user(connection: &Pool<Postgres>, user_id: i32) {
    let e = sqlx::query!(
        r#"UPDATE "User" SET "permissions" = 8 WHERE id = $1"#,
        user_id
    )
    .execute(connection)
    .await;

    match e {
        Ok(_) => {}
        Err(err) => {
            error!("Error while updating user: {}", err)
        }
    };
}

pub async fn unrestrict_user(connection: &Pool<Postgres>, user_id: i32) {
    sqlx::query!(
        r#"UPDATE "User" SET "permissions" = 0 WHERE id = $1"#,
        user_id
    )
    .execute(connection)
    .await
    .unwrap_or_default();
}

pub async fn get_silenced_until(connection: &Pool<Postgres>, user_id: i32) -> i64 {
    // let result = sqlx::query_as!(r#"SELECT *, "punishmentType" as "punishmentType: String" FROM "Punishment" WHERE "appliedTo" = $1 AND "punishmentType" = 'TIMEOUT'"#, user_id).fetch_one(connection).await;
    let result: Result<Option<Punishment>, sqlx::Error> = sqlx::query_as(
        r#"SELECT * FROM "Punishment" WHERE "appliedTo" = $1 AND "punishmentType" = 'TIMEOUT'""#,
    )
    .bind(user_id)
    .fetch_optional(connection)
    .await;
    match result {
        Ok(row) => {
            if let Some(punishment) = row {
                return punishment
                    .expires_at
                    .unwrap_or(NaiveDateTime::UNIX_EPOCH)
                    .timestamp();
            }

            0
        }
        Err(_) => 0,
    }
}

pub async fn insert_user_punishment(
    connection: &Pool<Postgres>,
    level: String,
    applied_by: i32,
    applied_to: i32,
    punishment_type: String,
    expires: bool,
    expires_at: Option<NaiveDateTime>,
    note: String,
) -> Option<Punishment> {
    let id = Uuid::new_v4().to_string();
    let row = sqlx::query!(
        r#"
        INSERT INTO "Punishment" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING "id"
    "#,
        id,
        Utc::now().naive_utc(),
        level,
        applied_by,
        applied_to,
        punishment_type,
        expires,
        expires_at.unwrap_or(NaiveDateTime::UNIX_EPOCH),
        note
    )
    .fetch_one(connection)
    .await;

    match row {
        Err(err) => {
            error!("{}", err);
            None
        }
        Ok(record) => {
            let id = record.id;
            let punishment = get_punishment_by_id(connection, id).await;

            if let Some(punishment) = punishment {
                punishment_alert(
                    &punishment,
                    &get_user_by_id(connection, applied_to)
                        .await
                        .expect("Failed to fetch user that received punishment")
                        .expect("Failed to unwrap user"),
                    &get_user_by_id(connection, applied_by)
                        .await
                        .expect("Failed to fetch user that applied punishment")
                        .expect("Failed to unwrap user"),
                )
                .await;
                return Some(punishment);
            }
            None
        }
    }
}

pub async fn punishment_alert(punishment: &Punishment, user: &User, moderator: &User) {
    let config = RunConfiguration::parse();

    if let Some(webhook) = config.alert_discord_webhook {
        let client = WebhookClient::new(webhook.as_str());

        let formatted_report = format!(
            r#"
New punishment has been received from [{0}](https://osu.{2}/users/{1}).
Applied to: [{3}](https://osu.{2}/users/{4})
Type: `{5}`
Expires: `{6}`
Note: ```
{7}
```
"#,
            moderator.username,
            moderator.id,
            config.server_url,
            user.username,
            user.id,
            punishment.punishment_type,
            punishment.expires_at.unwrap_or(NaiveDateTime::UNIX_EPOCH),
            punishment.note
        );

        if let Err(error) = client
            .send(|message| {
                message.content("New user punishment!").embed(|embed| {
                    embed
                        .author(
                            &user.username,
                            Some(format!("https://{}/users/{}", config.server_url, user.id)),
                            Some(format!("https://a.{}/{}", config.server_url, user.id)),
                        )
                        .description(formatted_report.as_str())
                })
            })
            .await
        {
            warn!("Failed to send alert: {}", error);
        }
    }
}

pub async fn increase_user_score(
    connection: &Pool<Postgres>,
    mode: OsuMode,
    score: i64,
    user_id: &i32,
) {
    sqlx::query(
        format!(
            r#"
UPDATE
    "UserStats"
SET
    "totalScore{0}" = "totalScore{0}" + $1
WHERE
    "userId" = $2
"#,
            mode.to_db_suffix()
        )
        .as_str(),
    )
    .bind(score)
    .bind(user_id)
    .execute(connection)
    .await
    .unwrap_or_default();
}

pub async fn increase_user_playcount(connection: &Pool<Postgres>, mode: OsuMode, user_id: &i32) {
    sqlx::query(
        format!(
            r#"
UPDATE
"UserStats"
SET
"playCount{0}" = "playCount{0}" + 1
WHERE
"userId" = $1
"#,
            mode.to_db_suffix()
        )
        .as_str(),
    )
    .bind(user_id)
    .execute(connection)
    .await
    .unwrap_or_default();
}

pub async fn update_user_max_combo(
    connection: &Pool<Postgres>,
    mode: &OsuMode,
    user_id: &i32,
    max_combo: i16,
) {
    sqlx::query(
        format!(
            r#"
UPDATE
"UserStats"
SET
"maxCombo{0}" = $1
WHERE
"userId" = $2
"#,
            mode.to_db_suffix()
        )
        .as_str(),
    )
    .bind(user_id)
    .bind(max_combo)
    .execute(connection)
    .await
    .unwrap_or_default();
}

pub async fn recalculate_user_stats(
    connection: &Pool<Postgres>,
    redis: &redis::Client,
    user: &User,
    mode: &OsuMode,
) {
    let scores: Result<Vec<Score>, sqlx::Error> = sqlx::query_as(
        r#"
    SELECT
	row_number() OVER (ORDER BY "Score"."performance" DESC) as rank,
    "Score"."id" as "score_id",
    "Score"."maxCombo" as "score_max_combo",
    "Score"."status" as "score_status",
    *
FROM
    "Score"
WHERE
    "userId" = $1 AND
    "status" = 2 AND
	"playMode" = $2
GROUP BY "Score"."beatmapChecksum", "Score"."id"
ORDER BY "performance" DESC
"#,
    )
    .bind(user.id)
    .bind(mode.to_osu())
    .fetch_all(connection)
    .await;

    if let Err(error) = scores {
        error!("Failed to get scores: {}", error);
        return;
    }

    let scores = scores.unwrap();
    let avarage_accuracy = scores
        .iter()
        .map(|score| score.calculate_accuracy())
        .sum::<f64>()
        / (scores.len() as f64);
    //weight
    let mut pp = 0.0;

    for score in scores {
        pp += score.performance * to_fixed(((0.95_f32).powi((score.rank - 1_i64) as i32)) as f64, 2)
    }

    sqlx::query(
        format!(
            r#"
    UPDATE
        "UserStats"
    SET
        "pp{0}" = $1,
        "avgAcc{0}" = $2
    WHERE
        "userId" = $3
    "#,
            mode.to_db_suffix()
        )
        .as_str(),
    )
    .bind(pp.round())
    .bind(avarage_accuracy / 100.0)
    .bind(user.id)
    .execute(connection)
    .await
    .unwrap_or_default();

    //Updating leaderboard in redis

    match redis.get_connection() {
        Ok(mut connection) => match is_restricted(user).await {
            true => {
                let _: Result<i32, redis::RedisError> = connection.zrem(
                    format!("leaderboard:{}:performance", mode.to_osu()),
                    user.id.to_string(),
                );
                let _: Result<i32, redis::RedisError> = connection.zrem(
                    format!("leaderboard:{}:performance:{}", mode.to_osu(), user.country),
                    user.id.to_string(),
                );
            }
            false => {
                let _: Result<i32, redis::RedisError> = connection.zadd(
                    format!("leaderboard:{}:performance", mode.to_osu()),
                    user.id.to_string(),
                    pp as i64,
                );
                let _: Result<i32, redis::RedisError> = connection.zadd(
                    format!("leaderboard:{}:performance:{}", mode.to_osu(), user.country),
                    user.id.to_string(),
                    pp as i64,
                );
            }
        },
        Err(err) => {
            error!("Failed to update rankings in redis: {}", err);
        }
    }
}

pub async fn send_message_announcement(
    url: String,
    message: String,
    message_type: String,
    target: String,
    key: String,
) {
    let response = reqwest::Client::new()
        .post(url)
        .json(&json!({
            "message": message,
            "message_type": message_type,
            "target": target,
            "key": key
        }))
        .send()
        .await;

    if let Ok(response) = response {
        debug!("{:#?}", response.text().await);
    }
}

pub async fn remove_friend(connection: &Pool<Postgres>, user_id: &i32, friend_id: &i32) {
    sqlx::query("DELETE FROM \"RelationShips\" WHERE \"userId\" = $1 AND \"friendId\" = $2")
        .bind(user_id)
        .bind(friend_id)
        .execute(connection)
        .await
        .unwrap_or_default();
}

pub async fn add_friend(connection: &Pool<Postgres>, user_id: &i32, friend_id: &i32) {
    sqlx::query("INSERT INTO \"RelationShips\" (\"userId\", \"friendId\") VALUES ($1, $2)")
        .bind(user_id)
        .bind(friend_id)
        .execute(connection)
        .await
        .unwrap_or_default();
}

pub async fn is_user_friend(connection: &Pool<Postgres>, user_id: &i32, friend_id: &i32) -> bool {
    let result = sqlx::query(
        r#"
    SELECT
    CASE
      WHEN EXISTS (
        SELECT *
        FROM "RelationShips"
        WHERE "userId" = r1."userId" AND "friendId" = r1."friendId"
      ) THEN TRUE
      ELSE FALSE
    END AS "is_reciprocated"
  FROM
    "RelationShips" r1
  WHERE
    r1."userId" = $1
    AND
    r1."friendId" = $2;

"#,
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_one(connection)
    .await;
    if let Ok(result) = result {
        return result.try_get("is_reciprocated").unwrap_or(false);
    }
    false
}

pub fn is_donator(user: &User) -> bool {
    user.donor_until
        .unwrap_or(NaiveDateTime::UNIX_EPOCH)
        .timestamp()
        > Utc::now().timestamp()
}

pub async fn send_bancho_message(user_id: &i32, method: String, arguments: Option<Vec<String>>) {
    let config = RunConfiguration::parse();
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://c.{}/api/v2/bancho/update",
            config.server_url
        ))
        .body(
            json!({
                "key": config.token_hmac_secret,
                "user_id": user_id,
                "method": method,
                "args": arguments.unwrap_or_default()
            })
            .to_string(),
        )
        .header("Content-Type", "application/json")
        .send()
        .await;

    if let Err(error) = response {
        error!("Failed to send request: {}", error);
        return;
    }

    let response = response.unwrap();

    let data = response
        .text()
        .await
        .unwrap_or("Failed to unwrap".to_string());

    debug!("Bancho message sent request: {}", data);
}
