use std::{collections::HashMap, ops::BitAnd};

use redis::{Client, Commands};
use serde::Deserialize;
use sqlx::{prelude::FromRow, Pool, Postgres, Row};
use tracing::{debug, error, info};

use crate::{api::users::users::PublicScore, db::user::User};

use super::{beatmap_utils::Beatmap, general_utils::to_fixed, http_utils::OsuMode};

#[derive(Debug, Clone)]
pub enum OsuServerError {
    Internal(String),
    BeatmapProcessingFailed(String),
    FailedToFetch(String),
}

#[derive(FromRow, Debug, Clone)]
pub struct Score {
    #[sqlx(rename = "score_id")]
    pub id: i32,
    #[sqlx(rename = "score_max_combo")]
    pub max_combo: i32,
    #[sqlx(rename = "count50")]
    pub count_50: i32,
    #[sqlx(rename = "count100")]
    pub count_100: i32,
    #[sqlx(rename = "count300")]
    pub count_300: i32,
    #[sqlx(rename = "countMiss")]
    pub count_miss: i32,
    #[sqlx(rename = "countGeKi")]
    pub count_geki: i32,
    #[sqlx(rename = "totalScore")]
    pub total_score: i32,
    #[sqlx(rename = "countKatu")]
    pub count_katu: i32,
    #[sqlx(rename = "perfect")]
    pub is_perfect: bool,
    #[sqlx(rename = "score_status")]
    pub status: i32,
    #[sqlx(rename = "submittedAt")]
    pub submitted_at: sqlx::types::chrono::NaiveDateTime,
    #[sqlx(rename = "playMode")]
    pub playmode: i32,
    pub performance: f64,
    pub mods: i32,
    pub rank: i64,
    #[sqlx(rename = "beatmapChecksum")]
    pub beatmap_checksum: String,
}

impl Score {
    pub fn get_total_hits(&self) -> i32 {
        if self.playmode == 2 {
            return self.count_50
                + self.count_100
                + self.count_300
                + self.count_miss
                + self.count_katu;
        }

        if self.playmode == 3 {
            return self.count_300
                + self.count_100
                + self.count_50
                + self.count_miss
                + self.count_geki
                + self.count_katu;
        }

        self.count_300 + self.count_100 + self.count_50 + self.count_miss
    }

    pub fn calculate_accuracy(&self) -> f64 {
        (match OsuMode::from_id(self.playmode as u8) {
            OsuMode::Osu | OsuMode::Relax => {
                ((self.count_300 * 300 + self.count_100 * 100 + self.count_50 * 50) as f64)
                    / ((300 * self.get_total_hits()) as f64)
            }
            OsuMode::Taiko => {
                ((self.count_300 * 150 + self.count_100 * 300) as f64)
                    / (self.get_total_hits() as f64)
            }
            OsuMode::Fruits => {
                ((self.count_50 + self.count_100 + self.count_300) as f64)
                    / (self.get_total_hits() as f64)
            }
            OsuMode::Mania => {
                (((self.count_300 + self.count_geki) * 300)
                    + (self.count_katu * 200)
                    + (self.count_100 * 100)
                    + (self.count_50 * 50)) as f64
                    / (self.get_total_hits() * 300) as f64
            }
        }) * 100.0
    }

    pub fn calculate_grade(&self) -> String {
        if self.status == -1 {
            return "F".to_string();
        }

        match OsuMode::from_id(self.playmode as u8) {
            OsuMode::Osu | OsuMode::Relax | OsuMode::Taiko => self.calculate_grade_standard(),
            OsuMode::Fruits => self.calculate_grade_fruits(),
            OsuMode::Mania => self.calculate_grade_mania(),
        }
    }

    fn is_visibility_modes_applied(&self) -> bool {
        //HD and FL
        //Todo: Move it to enum
        if (self.mods & ((1 << (3_i32)) | (1 << (10_i32)))) != 0 {
            return true;
        }

        false
    }

    fn calculate_grade_standard(&self) -> String {
        let ratio300 = (self.count_300 as f64) / (self.get_total_hits() as f64);
        let ratio50 = (self.count_50 as f64) / (self.get_total_hits() as f64);

        if ratio300 == 1.0 && self.is_visibility_modes_applied() {
            return "XH".to_string();
        }

        if ratio300 > 0.9 && ratio50 <= 0.01 && self.count_miss == 0 {
            if self.is_visibility_modes_applied() {
                return "SH".to_string();
            }
            return "S".to_string();
        }
        if (ratio300 > 0.8 && self.count_miss == 0) || ratio300 > 0.9 {
            return "A".to_string();
        }

        if (ratio300 > 0.7 && self.count_miss == 0) || ratio300 > 0.8 {
            return "B".to_string();
        }

        if ratio300 > 0.6 {
            return "C".to_string();
        }

        "D".to_string()
    }

    fn calculate_grade_fruits(&self) -> String {
        let accuracy = self.calculate_accuracy();

        if accuracy == 100.0 {
            if self.is_visibility_modes_applied() {
                return "XH".to_string();
            }

            return "X".to_string();
        }

        if accuracy >= 98.0 {
            if self.is_visibility_modes_applied() {
                return "SH".to_string();
            }

            return "S".to_string();
        }

        if accuracy >= 94.0 {
            return "A".to_string();
        }

        if accuracy >= 90.0 {
            return "B".to_string();
        }

        if accuracy >= 85.0 {
            return "C".to_string();
        }

        "D".to_string()
    }

    fn calculate_grade_mania(&self) -> String {
        let accuracy = self.calculate_accuracy();

        if accuracy == 100.0 {
            if self.is_visibility_modes_applied() {
                return "XH".to_string();
            }

            return "X".to_string();
        }

        if accuracy >= 95.0 {
            if self.is_visibility_modes_applied() {
                return "SH".to_string();
            }

            return "S".to_string();
        }

        if accuracy >= 90.0 {
            return "A".to_string();
        }

        if accuracy >= 80.0 {
            return "B".to_string();
        }

        if accuracy >= 70.0 {
            return "C".to_string();
        }

        "D".to_string()
    }
}

#[derive(FromRow, Debug, Clone)]
pub struct UserScore {
    pub score: Score,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub enum SortMode {
    Performance,
    Score,
}

impl SortMode {
    pub fn to_sql(&self) -> &str {
        match self {
            SortMode::Performance => "performance",
            SortMode::Score => "totalScore",
        }
    }
}

impl UserScore {
    pub fn to_osu(&self, rank: i64) -> String {
        let data = vec![
            self.score.id.to_string(),
            self.user.username.clone(),
            128.bitand(self.score.mods)
                .eq(&128)
                .then(|| to_fixed(self.score.performance, 0).to_string())
                .unwrap_or(self.score.total_score.to_string()),
            self.score.max_combo.to_string(),
            self.score.count_50.to_string(),
            self.score.count_100.to_string(),
            self.score.count_300.to_string(),
            self.score.count_miss.to_string(),
            self.score.count_katu.to_string(),
            self.score.count_geki.to_string(),
            match self.score.is_perfect {
                true => "True".to_string(),
                false => "False".to_string(),
            },
            self.score.mods.to_string(),
            self.user.id.to_string(),
            rank.to_string(),
            self.score.submitted_at.timestamp().to_string(),
            "1".to_string(),
        ];

        data.join("|")
    }
}

pub async fn get_user_best(
    connection: &Pool<Postgres>,
    beatmap_checksum: String,
    user_id: i32,
    mode: OsuMode,
    mods: i32,
    _leaderboard_type: i32,
    status: Option<i32>,
) -> Result<Option<UserScore>, OsuServerError> {
    let score = sqlx::query(
        r#"
    WITH ranked_scores AS (
        SELECT "Score".*, "User".*,
        "Score"."id" as "score_id",
        "Score"."status" as "score_status",
        "Score"."maxCombo" as "score_max_combo",
        row_number() OVER (ORDER BY "Score"."totalScore" DESC) as rank
        FROM "Score"
        JOIN "User" ON "Score"."userId" = "User"."id"
        WHERE "Score"."beatmapChecksum" = $2 AND
        "Score"."status" = $3 AND
        "Score"."playMode" = $4 AND
        "User"."permissions" & 8 = 0
    )
    SELECT *,
           (SELECT rank FROM ranked_scores WHERE "userId" = $1 LIMIT 1) AS rank
    FROM ranked_scores
    WHERE "userId" = $1;
"#,
    )
    .bind(user_id)
    .bind(beatmap_checksum.clone())
    .bind(status.unwrap_or(0))
    .bind(if 128.bitand(mods).eq(&128) {
        4
    } else {
        mode.to_osu()
    })
    .bind(mods & 128)
    .fetch_all(connection)
    .await;

    match score {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            err => {
                error!("Failed while fetching user best: {}", err);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(score) => {
            if score.len() > 1 {
                info!(
                    "More than one score found on {} by user {}",
                    beatmap_checksum.clone(),
                    user_id
                );
            }

            let score = score.first();

            match score {
                Some(user_score) => Ok(Some(UserScore {
                    score: Score::from_row(user_score).unwrap(),
                    user: User::from_row(user_score).unwrap(),
                })),
                None => Ok(None),
            }
        }
    }
}

pub async fn get_beatmap_leaderboard(
    connection: &Pool<Postgres>,
    beatmap_checksum: String,
    mode: OsuMode,
    mods: i32,
    status: Option<i32>,
) -> Result<Vec<UserScore>, OsuServerError> {
    let scores = sqlx::query(
        format!(
            r#"
SELECT "Score".*, "User".*,
    row_number() OVER (ORDER BY "Score"."{}" DESC) as rank,
    "Score"."id" as "score_id",
    "Score"."status" as "score_status",
    "Score"."maxCombo" as "score_max_combo"
FROM "Score"
JOIN "User" ON "Score"."userId" = "User"."id"
WHERE
    "Score"."beatmapChecksum" = $1 AND
    "Score"."status" = $2 AND
    "Score"."playMode" = $3 AND
    "User"."permissions" & 8 = 0
ORDER BY
    "Score"."totalScore" DESC
"#,
            if mode.eq(&OsuMode::Relax) {
                "performance"
            } else {
                "totalScore"
            }
        )
        .as_str(),
    )
    .bind(beatmap_checksum)
    .bind(status.unwrap_or(2))
    .bind(if 128.bitand(mods).eq(&128) {
        4
    } else {
        mode.to_osu()
    })
    .fetch_all(connection)
    .await;

    match scores {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => {
                error!("Failed while fetching scores: {}", error);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(scores) => {
            let mut scores_out: Vec<UserScore> = Vec::new();

            debug!("Found {} rows", scores.len());
            for score in scores {
                let user = User::from_row(&score).unwrap();
                let score = Score::from_row(&score).unwrap();
                scores_out.push(UserScore { score, user });
            }
            Ok(scores_out)
        }
    }
}

pub async fn _get_beatmap_leaderboard_by_id(
    connection: &Pool<Postgres>,
    beatmap_id: i32,
    mode: OsuMode,
) -> Result<Vec<UserScore>, OsuServerError> {
    let scores = sqlx::query(
        r#"
SELECT "Score".*, "User".*,
    row_number() OVER (ORDER BY "Score"."totalScore" DESC) as rank,
    "Score"."id" as "score_id",
    "Score"."status" as "score_status",
    "Score"."maxCombo" as "score_max_combo"
FROM "Score"
JOIN "User" ON "Score"."userId" = "User"."id"
WHERE
    "Score"."beatmapId" = $1 AND
    "Score"."status" = $2 AND
    "Score"."playMode" = $3 AND
    "Score"."mods" & 128 = 0 AND
    "User"."permissions" & 8 = 0
ORDER BY
    "Score"."totalScore" DESC
"#,
    )
    .bind(beatmap_id)
    .bind(2)
    .bind(mode.to_osu())
    .fetch_all(connection)
    .await;

    match scores {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => {
                error!("Failed while fetching scores: {}", error);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(scores) => {
            let mut scores_out: Vec<UserScore> = Vec::new();

            debug!("Found {} rows", scores.len());
            for score in scores {
                let user = User::from_row(&score).unwrap();
                let score = Score::from_row(&score).unwrap();
                scores_out.push(UserScore { score, user });
            }
            Ok(scores_out)
        }
    }
}

#[derive(FromRow, Debug, Clone)]
pub struct UserScoreWithBeatmap {
    pub score: Score,
    pub user: User,
    pub beatmap: Beatmap,
    pub rank: i32,
}

impl UserScoreWithBeatmap {
    pub fn calculate_weight(&self) -> f32 {
        if self.score.status < 2 {
            return 0.0;
        }
        to_fixed(((0.95_f32).powi(self.rank - 1) * 100.0) as f64, 2) as f32
    }

    pub fn publish(&self) -> PublicScore {
        PublicScore {
            id: self.score.id,
            //This .clone is so cringy
            beatmap: self.beatmap.clone().to_public(),
            user_id: self.user.id,
            user: None,
            accuracy: to_fixed(self.score.calculate_accuracy(), 2),
            count300: self.score.count_300,
            count100: self.score.count_100,
            count50: self.score.count_50,
            count_geki: self.score.count_geki,
            count_katu: self.score.count_katu,
            grade: self.score.calculate_grade(),
            playmode: OsuMode::from_id(self.score.playmode as u8),
            max_combo: self.score.max_combo,
            mods: self.score.mods,
            weighted: self.calculate_weight(),
            performance: self.score.performance,
            submitted_at: self.score.submitted_at,
            passed: self.score.status > 0,
            count_miss: self.score.count_miss,
            total_score: self.score.total_score,
        }
    }
}

pub async fn get_user_best_scores(
    connection: &Pool<Postgres>,
    user: &User,
    limit: Option<i32>,
    offset: Option<i32>,
    mode: OsuMode,
    sorting: Option<SortMode>,
) -> Result<Vec<UserScoreWithBeatmap>, OsuServerError> {
    let limit = limit.unwrap_or(10);
    let offset = offset.unwrap_or(0);
    let sorting = sorting.unwrap_or(SortMode::Performance);

    let scores = sqlx::query(
        format!(
            r#"
    SELECT
    "Score".*, "User".*, "Beatmap".*,
    "Score"."status" as score_status,
    "Score"."maxCombo" as "score_max_combo",
    "Score"."id" as score_id,
    row_number() OVER (ORDER BY "Score"."{}" DESC) as rank
FROM
    "Score"
	JOIN "User" ON "Score"."userId" = "User"."id"
	JOIN "Beatmap" ON "Beatmap"."checksum" = "Score"."beatmapChecksum"
WHERE
    "Score"."status" = $1 AND
    "Score"."playMode" = $2 AND
    "User"."id" = $3
LIMIT
    $4
OFFSET
    $5
"#,
            sorting.to_sql()
        )
        .as_str(),
    )
    .bind(2)
    .bind(mode.to_osu())
    .bind(user.id)
    .bind(limit)
    .bind(offset)
    .fetch_all(connection)
    .await;

    match scores {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => {
                error!("Failed while fetching scores: {}", error);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(scores) => {
            let mut scores_out: Vec<Vec<UserScoreWithBeatmap>> = Vec::new();

            for row in scores {
                let beatmap = Beatmap::from_row(&row).unwrap();
                let user = User::from_row(&row).unwrap();
                let score = Score::from_row(&row).unwrap();
                let rank: Result<i64, sqlx::Error> = row.try_get("rank");
                if let Err(error) = rank {
                    return Err(OsuServerError::Internal(error.to_string()));
                }

                let rank = rank.unwrap();

                scores_out.push(vec![UserScoreWithBeatmap {
                    score,
                    user,
                    beatmap,
                    rank: rank as i32,
                }]);
            }

            Ok(scores_out.into_iter().flatten().collect())
        }
    }
}

pub async fn get_user_recent_scores(
    connection: &Pool<Postgres>,
    user: &User,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<UserScoreWithBeatmap>, OsuServerError> {
    let offset = offset.unwrap_or(0);

    let scores = sqlx::query(
        r#"
SELECT
    "Score".*, "User".*, "Beatmap".*,
    "Score"."status" as score_status,
    "Score"."id" as score_id,
    "Score"."maxCombo" as "score_max_combo",
    row_number() OVER (ORDER BY "Score"."submittedAt" DESC) as rank
FROM
    "Score"
	JOIN "User" ON "Score"."userId" = "User"."id"
	JOIN "Beatmap" ON "Beatmap"."checksum" = "Score"."beatmapChecksum"
WHERE
    "User"."id" = $1
ORDER BY
    "Score"."submittedAt" DESC
LIMIT
    $2
OFFSET
    $3
"#,
    )
    .bind(user.id)
    .bind(limit)
    .bind(offset)
    .fetch_all(connection)
    .await;

    match scores {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => {
                error!("Failed while fetching scores: {}", error);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(scores) => {
            let mut scores_out: Vec<Vec<UserScoreWithBeatmap>> = Vec::new();

            for row in scores {
                let beatmap = Beatmap::from_row(&row).unwrap();
                let user = User::from_row(&row).unwrap();
                let score = Score::from_row(&row).unwrap();
                let rank: Result<i64, sqlx::Error> = row.try_get("rank");
                if let Err(error) = rank {
                    return Err(OsuServerError::Internal(error.to_string()));
                }

                let rank = rank.unwrap();

                scores_out.push(vec![UserScoreWithBeatmap {
                    score,
                    user,
                    beatmap,
                    rank: rank as i32,
                }]);
            }

            Ok(scores_out.into_iter().flatten().collect())
        }
    }
}

pub async fn get_score_with_beatmap_by_id(
    connection: &Pool<Postgres>,
    id: i32,
) -> Result<Option<UserScoreWithBeatmap>, OsuServerError> {
    let scores = sqlx::query(
        r#"
SELECT
    "Score".*, "User".*, "Beatmap".*,
    "Score"."status" as score_status,
    "Score"."id" as score_id,
    "Score"."maxCombo" as "score_max_combo",
    row_number() OVER (ORDER BY "Score"."submittedAt" DESC) as rank
FROM
    "Score"
	JOIN "User" ON "Score"."userId" = "User"."id"
	JOIN "Beatmap" ON "Beatmap"."checksum" = "Score"."beatmapChecksum"
WHERE
    "Score"."id" = $1
"#,
    )
    .bind(id)
    .fetch_one(connection)
    .await;

    match scores {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            error => {
                error!("Failed while fetching scores: {}", error);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(row) => {
            let beatmap = Beatmap::from_row(&row).unwrap();
            let user = User::from_row(&row).unwrap();
            let score = Score::from_row(&row).unwrap();
            let rank: Result<i64, sqlx::Error> = row.try_get("rank");
            if let Err(error) = rank {
                return Err(OsuServerError::Internal(error.to_string()));
            }

            let rank = rank.unwrap();

            Ok(Some(UserScoreWithBeatmap {
                score,
                user,
                beatmap,
                rank: rank as i32,
            }))
        }
    }
}

pub async fn get_user_grades_count(
    connection: &Pool<Postgres>,
    client: &Client,
    user: &User,
    mode: &OsuMode,
    recalculate: Option<bool>,
    grade: String,
) -> Result<i32, OsuServerError> {
    let redis_connection = client.get_connection();

    if let Err(error) = redis_connection {
        error!("Failed to get connection: {}", error);
        return Err(OsuServerError::Internal(
            "Failed to get connection.".to_string(),
        ));
    }

    let mut redis_connection = redis_connection.unwrap();

    let cached_value: Result<Option<i32>, redis::RedisError> = redis_connection.get(format!(
        "user:{}:grades:{}:{}",
        user.id,
        mode.to_osu(),
        grade
    ));
    if let Err(error) = cached_value {
        error!("Failed to get cached value: {}", error);
        return Err(OsuServerError::Internal(
            "Failed to get cached value.".to_string(),
        ));
    }

    let cached_value = cached_value.unwrap().unwrap_or(-1);

    if cached_value == -1 || recalculate.unwrap_or(false) {
        let scores = get_user_best_scores(
            connection,
            user,
            None,
            Some(0),
            mode.clone(),
            Some(SortMode::Performance),
        )
        .await;
        if let Err(error) = scores {
            return Err(OsuServerError::Internal(format!(
                "Failed to fetch scores: {:#?}",
                error
            )));
        }

        let scores = scores.unwrap();

        let mut count = 0;

        for record in scores {
            if record.score.calculate_grade() == grade {
                count += 1;
            }
        }

        //Setting it in redis
        let _: () = redis_connection
            .set(
                format!("user:{}:grades:{}:{}", user.id, mode.to_osu(), grade),
                count,
            )
            .expect("Failed to update cache.");

        return Ok(count);
    }

    Ok(cached_value)
}

pub async fn get_score_by_id(
    connection: &Pool<Postgres>,
    score_id: i32,
    beatmap_checksum: String,
    mode: &OsuMode,
    status: Option<i32>,
) -> Result<Option<UserScoreWithBeatmap>, OsuServerError> {
    let rows = sqlx::query(
        r#"
    WITH RANKED_SCORES AS
	(SELECT "Score".*,
			"User".*,
			"Beatmap".*,
			"Score"."id" AS "score_id",
			"Score"."status" AS "score_status",
			"Score"."maxCombo" AS "score_max_combo",
			ROW_NUMBER() OVER (
		ORDER BY "Score"."totalScore" DESC) AS RANK
		FROM "Score"
		JOIN "User" ON "Score"."userId" = "User"."id"
		JOIN "Beatmap" ON "Beatmap"."checksum" = "Score"."beatmapChecksum"
		WHERE "Beatmap"."checksum" = $2
        AND "Score"."status" = $4
        AND "Score"."playMode" = $3
        )
SELECT *
FROM "ranked_scores"
WHERE "score_id" = $1
"#,
    )
    .bind(score_id)
    .bind(beatmap_checksum)
    .bind(mode.to_osu())
    .bind(status.unwrap_or(2))
    .fetch_one(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            error => {
                error!("Failed while fetching scores: {}", error);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(row) => {
            let _scores_out: Vec<UserScoreWithBeatmap> = Vec::new();

            let beatmap = Beatmap::from_row(&row).unwrap();
            let user = User::from_row(&row).unwrap();
            let score = Score::from_row(&row).unwrap();
            let rank: Result<i64, sqlx::Error> = row.try_get("rank");
            if let Err(error) = rank {
                return Err(OsuServerError::Internal(error.to_string()));
            }

            let rank = rank.unwrap();

            Ok(Some(UserScoreWithBeatmap {
                score,
                user,
                beatmap,
                rank: rank as i32,
            }))
        }
    }
}

pub async fn get_user_scores_on_beatmap(
    connection: &Pool<Postgres>,
    user_id: i32,
    beatmap_id: i32,
    mode: OsuMode,
) -> Result<Vec<UserScoreWithBeatmap>, OsuServerError> {
    let rows = sqlx::query(
        r#"
    WITH ranked_scores AS (
        SELECT "Score".*, "User".*, "Beatmap".*,
        "Score"."id" as "score_id",
        "Score"."status" as "score_status",
        "Score"."maxCombo" as "score_max_combo",
        row_number() OVER (ORDER BY "Score"."totalScore" DESC) as rank
        FROM "Score"
        JOIN "User" ON "Score"."userId" = "User"."id"
        JOIN "Beatmap" ON "Beatmap"."checksum" = "Score"."beatmapChecksum"
        WHERE "Beatmap"."beatmapId" = $1 AND
        "User"."id" = $2 AND
        "Score"."mods" & 128 = $3 AND
        "Score"."playMode" = $4 AND
        "User"."permissions" & 8 = 0
        LIMIT 1
    )
    SELECT *,
        (SELECT rank FROM ranked_scores WHERE "userId" = $2 LIMIT 1) AS "beatmap_rank"
    FROM ranked_scores
    WHERE "userId" = $2;
"#,
    )
    .bind(beatmap_id)
    .bind(user_id)
    .bind(mode.clone().eq(&OsuMode::Relax).then_some(128).or(Some(0)))
    .bind(mode.to_osu())
    .fetch_all(connection)
    .await;

    match rows {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            error => {
                error!("Failed while fetching scores: {}", error);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(scores) => {
            let mut scores_out: Vec<UserScoreWithBeatmap> = Vec::new();
            for row in scores {
                let beatmap = Beatmap::from_row(&row).unwrap();
                let user = User::from_row(&row).unwrap();
                let score = Score::from_row(&row).unwrap();
                let rank: Result<i64, sqlx::Error> = row.try_get("rank");
                if let Err(error) = rank {
                    return Err(OsuServerError::Internal(error.to_string()));
                }

                let rank = rank.unwrap();

                scores_out.push(UserScoreWithBeatmap {
                    score,
                    user,
                    beatmap,
                    rank: rank as i32,
                });
            }

            Ok(scores_out)
        }
    }
}

pub async fn get_first_place_on_beatmap(
    connection: &Pool<Postgres>,
    beatmap_checksum: String,
    mode: OsuMode,
) -> Option<UserScoreWithBeatmap> {
    let rows = sqlx::query(
        r#"
    SELECT "Score".*, "User".*, "Beatmap".*,
	"Score"."id" as "score_id",
	"Score"."status" as "score_status",
	"Score"."maxCombo" as "score_max_combo",
	row_number() OVER (ORDER BY "Score"."performance" DESC) as rank
FROM "Score"
JOIN "User" ON "Score"."userId" = "User"."id"
JOIN "Beatmap" ON "Beatmap"."checksum" = "Score"."beatmapChecksum"
WHERE
	"Score"."beatmapChecksum" = $1 AND
	"Score"."status" = $2 AND
	"Score"."playMode" = $3 AND
	"User"."permissions" & 8 = 0
ORDER BY "rank" ASC
LIMIT 1
    "#,
    )
    .bind(beatmap_checksum)
    .bind(2)
    .bind(mode.to_osu())
    .fetch_optional(connection)
    .await;

    match rows {
        Err(error) => {
            error!("Failed while fetching scores: {}", error);
            None
        }
        Ok(row) => {
            if let Some(row) = row {
                let beatmap = Beatmap::from_row(&row).unwrap();
                let user = User::from_row(&row).unwrap();
                let score = Score::from_row(&row).unwrap();
                let rank: Result<i64, sqlx::Error> = row.try_get("rank");
                if let Err(_error) = rank {
                    return None;
                }

                let rank = rank.unwrap();

                Some(UserScoreWithBeatmap {
                    score,
                    user,
                    beatmap,
                    rank: rank as i32,
                })
            } else {
                None
            }
        }
    }
}

pub fn get_mods_hashmap() -> HashMap<u32, String> {
    let mut mod_keys = HashMap::new();
    mod_keys.insert(1 << 0, "NF".to_string());
    mod_keys.insert(1 << 1, "EZ".to_string());
    mod_keys.insert(1 << 2, "TD".to_string());
    mod_keys.insert(1 << 3, "HD".to_string());
    mod_keys.insert(1 << 4, "HR".to_string());
    mod_keys.insert(1 << 5, "SD".to_string());
    mod_keys.insert(1 << 6, "DT".to_string());
    mod_keys.insert(1 << 7, "RX".to_string());
    mod_keys.insert(1 << 8, "HT".to_string());
    mod_keys.insert(1 << 9, "NC".to_string());
    mod_keys.insert(1 << 10, "FL".to_string());
    mod_keys.insert(1 << 12, "SO".to_string());
    mod_keys.insert(1 << 13, "AP".to_string());

    mod_keys
}

pub fn format_mods(mods: u32) -> String {
    let mod_keys = get_mods_hashmap();

    let mut result = Vec::new();

    for key in mod_keys.keys() {
        if (mods & key) > 0 {
            result.push(
                mod_keys
                    .get(key)
                    .unwrap_or(&"".to_string())
                    .clone()
                    .to_owned(),
            );
        }
    }

    if result.is_empty() {
        return "NM".to_string();
    }

    result.join("").to_string()
}

fn find_keys_for_value(map: HashMap<u32, String>, value: String) -> Option<u32> {
    let result = map.iter().find(|(_key, val)| **val == value);

    if let Some((key, _value)) = result {
        return Some(key.to_owned());
    }

    None
}

pub fn parse_mods(mods: String) -> u32 {
    let mut mods_out = 0;
    let mod_keys = get_mods_hashmap();

    let mut i = 0;
    let mut prev_mode: String = "".to_string();

    for mod_symbol in mods.to_uppercase().split("") {
        if i % 2 == 0 {
            prev_mode += mod_symbol;
            let mod_bitwise = find_keys_for_value(mod_keys.clone(), prev_mode);
            if let Some(mode_bitwise) = mod_bitwise {
                if mode_bitwise == 1 << 9 {
                    mods_out |= 1 << 6;
                }
                mods_out |= mode_bitwise;
            }
            prev_mode = "".to_string();
            i += 1;
            continue;
        }

        prev_mode += mod_symbol;
        i += 1;
    }

    mods_out
}
