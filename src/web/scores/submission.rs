use std::{collections::HashMap, ops::BitAnd, path::Path, str::FromStr, sync::Arc};

use axum::{body::Bytes, extract::Multipart, Extension};

use base64::prelude::*;
use chrono::{NaiveDateTime, Utc};
use simple_rijndael::{impls::RijndaelCbc, paddings::ZeroPadding};
use sqlx::{Pool, Postgres};
use tokio::fs;
use tracing::{debug, error, info, warn};

use crate::{
    context::Context,
    utils::{
        beatmap_utils::get_beatmap_by_hash,
        chart::Chart,
        http_utils::OsuMode,
        performance_utils::{calculate_performance_safe, is_cap_reached},
        score_utils::{get_first_place_on_beatmap, get_score_by_id, get_user_best, UserScore},
        user_utils::{
            find_user_by_id_or_username, get_rank, get_user_stats, increase_user_playcount,
            increase_user_score, insert_user_punishment, is_restricted, recalculate_user_stats,
            restrict_user, send_bancho_message, send_message_announcement, update_user_max_combo,
            validate_auth,
        },
    },
};

fn generate_submittion_key(version: String) -> Vec<u8> {
    let formatted_key = format!("osu!-scoreburgr---------{}", version);

    formatted_key.as_bytes().to_vec()
}

pub struct ParsedMultipart {
    files: HashMap<String, Bytes>,
    fields: HashMap<String, Bytes>,
}

impl ParsedMultipart {
    pub async fn from_multipart(mut data: Multipart) -> Self {
        let mut files = HashMap::new();
        let mut fields = HashMap::new();

        while let Some(field) = data.next_field().await.unwrap() {
            let name = field.name().unwrap().to_string();
            let is_file = field.file_name().is_some();

            if is_file {
                let file_data = field.bytes().await.unwrap();
                files.insert(name, file_data);
                continue;
            }

            let field_data = field.bytes().await.unwrap();
            fields.insert(name, field_data);
        }

        Self { files, fields }
    }

    pub fn get_field<T: std::str::FromStr>(&self, field_name: &str) -> Option<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let field_data = self.fields.get(field_name);

        if let Some(field_data) = field_data {
            let field_data = String::from_utf8(field_data.to_vec()).unwrap();
            let field_data = field_data.parse();

            if let Err(_error) = field_data {
                return None;
            }

            let field_data = field_data.unwrap();
            return Some(field_data);
        }

        None
    }

    pub fn get_file(&self, file_name: &str) -> Option<Bytes> {
        if let Some(file_data) = self.files.get(file_name) {
            return Some(file_data.clone());
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct ScoreDecryptedData {
    pub beatmap_md5: String,
    pub player_name: String,
    pub some_hash: String,
    pub count_300: u16,
    pub count_100: u16,
    pub count_50: u16,
    pub count_geki: u16,
    pub count_katu: u16,
    pub count_miss: u16,
    pub total_score: u32,
    pub max_combo: u16,
    pub perfect: bool,
    pub rank_string: String,
    pub mods: i32,
    pub failed: bool,
    pub playmode: u8,
}

impl ScoreDecryptedData {
    pub fn new(data: String) -> Self {
        let mut data = data.split(":");
        let beatmap_md5 = data.next().unwrap().to_string();
        let player_name = data.next().unwrap().to_string();
        let some_hash = data.next().unwrap().to_string();
        let count_300 = data.next().unwrap().parse().unwrap();
        let count_100 = data.next().unwrap().parse().unwrap();
        let count_50 = data.next().unwrap().parse().unwrap();
        let count_geki = data.next().unwrap().parse().unwrap();
        let count_katu = data.next().unwrap().parse().unwrap();
        let count_miss = data.next().unwrap().parse().unwrap();
        let total_score = data.next().unwrap().parse().unwrap();
        let max_combo = data.next().unwrap().parse().unwrap();
        let perfect = data.next().unwrap().parse::<String>().unwrap() == "True";
        let rank_string = data.next().unwrap().to_string();
        let mods = data.next().unwrap().parse().unwrap();
        let failed = data.next().unwrap().parse::<String>().unwrap() == "False";
        let playmode = data.next().unwrap().parse().unwrap();
        Self {
            beatmap_md5,
            player_name,
            some_hash,
            count_300,
            count_100,
            count_50,
            count_geki,
            count_katu,
            count_miss,
            total_score,
            max_combo,
            perfect,
            rank_string,
            mods,
            failed,
            playmode,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScoreStatus {
    Failed = -1,
    Unranked = 0,
    Ranked = 1,
    Best = 2,
    Loved = 3,
    LovedBest = 4,
}
impl ScoreStatus {
    pub fn to_db(&self) -> i32 {
        match self {
            ScoreStatus::Failed => -1,
            ScoreStatus::Unranked => 0,
            ScoreStatus::Ranked => 1,
            ScoreStatus::Best => 2,
            ScoreStatus::Loved => 3,
            ScoreStatus::LovedBest => 4,
        }
    }
}

pub enum BeatmapStatus {
    Unknown = -2,
    NotSubmitted = -1,
    LatestPending = 0,
    NeedUpdate = 1,
    Ranked = 2,
    Approved = 3,
    Qualified = 4,
    Loved = 5,
}

impl From<i32> for BeatmapStatus {
    fn from(value: i32) -> Self {
        match value {
            -2 => Self::Unknown,
            -1 => Self::NotSubmitted,
            0 => Self::LatestPending,
            1 => Self::NeedUpdate,
            2 => Self::Ranked,
            3 => Self::Approved,
            4 => Self::Qualified,
            5 => Self::Loved,
            _ => Self::Unknown,
        }
    }
}

impl ScoreStatus {
    pub fn find_suitable_best_status_for_beatmap(status: BeatmapStatus) -> ScoreStatus {
        match status {
            BeatmapStatus::LatestPending
            | BeatmapStatus::NeedUpdate
            | BeatmapStatus::Unknown
            | BeatmapStatus::NotSubmitted => ScoreStatus::Unranked,
            BeatmapStatus::Approved | BeatmapStatus::Ranked => ScoreStatus::Best,
            BeatmapStatus::Qualified | BeatmapStatus::Loved => ScoreStatus::LovedBest,
        }
    }

    pub fn find_suitable_ranked_status_for_beatmap(status: BeatmapStatus) -> ScoreStatus {
        match status {
            BeatmapStatus::LatestPending
            | BeatmapStatus::NeedUpdate
            | BeatmapStatus::Unknown
            | BeatmapStatus::NotSubmitted => ScoreStatus::Unranked,
            BeatmapStatus::Approved | BeatmapStatus::Ranked => ScoreStatus::Ranked,
            BeatmapStatus::Qualified | BeatmapStatus::Loved => ScoreStatus::Loved,
        }
    }
}

impl From<i32> for ScoreStatus {
    fn from(value: i32) -> Self {
        match value {
            -1 => Self::Failed,
            0 => Self::Unranked,
            1 => Self::Ranked,
            2 => Self::Best,
            3 => Self::Loved,
            4 => Self::LovedBest,
            _ => Self::Unranked,
        }
    }
}

pub struct PlayerScore {
    pub id: Option<i32>,
    pub status: ScoreStatus,
    pub performance: Option<f64>,

    //Score metadata
    pub beatmap_md5: String,
    pub playmode: u8,
    pub total_score: i32,
    pub max_combo: i32,
    pub count_300: i32,
    pub count_100: i32,
    pub count_50: i32,
    pub count_geki: i32,
    pub count_katu: i32,
    pub count_miss: i32,
    pub mods: i32,
    pub perfect: bool,
    pub submitted_at: NaiveDateTime,
}

impl PlayerScore {
    pub fn from_user_score(score: &Option<UserScore>) -> Option<Self> {
        if let Some(score) = score {
            return Some(Self {
                status: score.score.status.into(),
                performance: Some(score.score.performance),
                id: Some(score.score.id),
                beatmap_md5: score.score.beatmap_checksum.to_string(),
                playmode: score.score.playmode as u8,
                total_score: score.score.total_score,
                max_combo: score.score.max_combo,
                count_300: score.score.count_300,
                count_100: score.score.count_100,
                count_50: score.score.count_50,
                count_geki: score.score.count_geki,
                count_katu: score.score.count_katu,
                count_miss: score.score.count_miss,
                mods: score.score.mods,
                perfect: score.score.is_perfect,
                submitted_at: score.score.submitted_at,
            });
        }

        None
    }

    pub async fn update_status(&mut self, pool: &Pool<Postgres>, status: ScoreStatus) {
        if let Some(id) = self.id {
            let _ = sqlx::query(
                r#"
                UPDATE "Score" SET "status" = $1 WHERE "id" = $2
            "#,
            )
            .bind(status.to_db())
            .bind(id)
            .execute(pool)
            .await;
        }
    }

    pub async fn insert_score_in_db(&self, pool: &Pool<Postgres>, user_id: i32) -> Option<i32> {
        let id = sqlx::query!(
            r#"
            INSERT INTO "Score" (
                "beatmapChecksum",
                "playMode",
                "totalScore",
                "maxCombo",
                "count300",
                "count100",
                "count50",
                "countGeKi",
                "countKatu",
                "countMiss",
                "mods",
                "perfect",
                "status",
                "submittedAt",
                "userId",
                "performance"
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING "id"
        "#,
            self.beatmap_md5.clone(),
            128.bitand(self.mods)
                .eq(&128)
                .then(|| 4)
                .unwrap_or(self.playmode as i32),
            self.total_score,
            self.max_combo,
            self.count_300,
            self.count_100,
            self.count_50,
            self.count_geki,
            self.count_katu,
            self.count_miss,
            self.mods,
            self.perfect,
            self.status.to_db(),
            self.submitted_at,
            user_id,
            self.performance.unwrap_or(0.0)
        )
        .fetch_one(pool)
        .await;

        if let Err(error) = id {
            warn!("{:#?}", error);
            return None;
        }

        let id = id.unwrap();

        Some(id.id)
    }
}

pub async fn submit_score(Extension(ctx): Extension<Arc<Context>>, data: Multipart) -> String {
    let form_data = ParsedMultipart::from_multipart(data).await;

    let version = form_data.get_field::<String>("osuver");

    if let None = version {
        warn!("no version");
        return "error: no".to_string();
    }
    let version = version.unwrap();
    let sized_key = generate_submittion_key(version);

    let score = form_data.get_field::<String>("score");
    let score = score.unwrap();
    let cipher = BASE64_STANDARD.decode(score);

    if let Err(error) = cipher {
        warn!("{:#?}", error);
        return "error: no".to_string();
    }

    let cipher = cipher.unwrap();

    let iv = BASE64_STANDARD
        .decode(form_data.get_field::<String>("iv").unwrap())
        .unwrap();

    let decrypted_score = RijndaelCbc::<ZeroPadding>::new(&sized_key, 32)
        .unwrap()
        .decrypt(&iv, cipher);

    if let Err(error) = decrypted_score {
        warn!("{:#?}", error);
        return "error: no".to_string();
    }
    let decrypted_score = decrypted_score.unwrap();
    let string_data = String::from_utf8(decrypted_score).unwrap();

    let decrypted_score = ScoreDecryptedData::new(string_data);

    let user_password = form_data
        .get_field::<String>("pass")
        .unwrap_or("".to_string());

    if !validate_auth(
        &ctx.redis,
        &ctx.pool,
        decrypted_score.player_name.trim_end(),
        user_password,
    )
    .await
    {
        warn!("invalid auth");
        return "error: pass".to_string();
    }

    let beatmap = get_beatmap_by_hash(&ctx.pool, decrypted_score.beatmap_md5.clone()).await;

    if let Err(error) = beatmap {
        warn!("{:#?}", error);
        return "error: no".to_string();
    }

    let beatmap = beatmap.unwrap();

    if let None = beatmap {
        warn!("no beatmap");
        return "error: no".to_string();
    }

    let beatmap = beatmap.unwrap();

    let quit = form_data
        .get_field::<String>("x")
        .unwrap_or("0".to_string())
        .eq("1");
    let user =
        find_user_by_id_or_username(&ctx.pool, decrypted_score.player_name.trim().to_string())
            .await;

    if let Err(error) = user {
        warn!("{:#?}", error);
        return "error: no".to_string();
    }

    let user = user.unwrap();

    if let None = user {
        warn!("no user");
        return "error: pass".to_string();
    }

    let user = user.unwrap();

    let osu_mode = 128
        .bitand(decrypted_score.mods)
        .eq(&128)
        .then(|| OsuMode::Relax)
        .unwrap_or(OsuMode::from_id(decrypted_score.playmode));

    let performance = calculate_performance_safe(
        beatmap.clone().beatmap_id as i64,
        decrypted_score.mods as u32,
        decrypted_score.count_300 as usize,
        decrypted_score.count_100 as usize,
        decrypted_score.count_50 as usize,
        decrypted_score.count_geki as usize,
        decrypted_score.count_katu as usize,
        decrypted_score.count_miss as usize,
        decrypted_score.max_combo as usize,
        osu_mode.clone(),
    )
    .await;

    let best_score = get_user_best(
        &ctx.pool,
        beatmap.clone().checksum,
        user.id,
        osu_mode.clone(),
        decrypted_score.mods,
        0,
        Some(
            ScoreStatus::find_suitable_best_status_for_beatmap(beatmap.clone().status.into())
                .to_db(),
        ),
    )
    .await;

    if let Err(error) = best_score {
        warn!("{:#?}", error);
        return "error: no".to_string();
    }

    let best_score = best_score.unwrap();
    let mut current_score = PlayerScore {
        id: None,
        status: ScoreStatus::Unranked,
        performance: Some(performance),
        beatmap_md5: decrypted_score.beatmap_md5.clone(),
        playmode: decrypted_score.playmode,
        total_score: decrypted_score.total_score as i32,
        max_combo: decrypted_score.max_combo as i32,
        count_300: decrypted_score.count_300 as i32,
        count_100: decrypted_score.count_100 as i32,
        count_50: decrypted_score.count_50 as i32,
        count_geki: decrypted_score.count_geki as i32,
        count_katu: decrypted_score.count_katu as i32,
        count_miss: decrypted_score.count_miss as i32,
        mods: decrypted_score.mods,
        perfect: decrypted_score.perfect,
        submitted_at: NaiveDateTime::from_timestamp_millis(Utc::now().timestamp_millis())
            .unwrap_or_default(),
    };

    let old_score = PlayerScore::from_user_score(&best_score);

    info!(
        "quit: {}, failed: {}, status: {:#?}",
        quit, decrypted_score.failed, current_score.status
    );
    if quit || decrypted_score.failed {
        current_score.status = ScoreStatus::Failed;
    }

    if current_score.status != ScoreStatus::Failed {
        if let Some(mut old_score) = old_score {
            if performance > old_score.performance.unwrap_or(0.0) {
                old_score
                    .update_status(
                        &ctx.pool,
                        ScoreStatus::find_suitable_ranked_status_for_beatmap(
                            beatmap.clone().status.into(),
                        ),
                    )
                    .await;

                current_score.status = ScoreStatus::find_suitable_best_status_for_beatmap(
                    beatmap.clone().status.into(),
                );

                info!("new status: {:#?}", current_score.status);
            }
        } else {
            current_score.status =
                ScoreStatus::find_suitable_best_status_for_beatmap(beatmap.clone().status.into());
        }
    }

    let _old_first_place_score = get_first_place_on_beatmap(
        &ctx.pool,
        decrypted_score.beatmap_md5.clone(),
        osu_mode.clone(),
    )
    .await;
    let score_id = current_score.insert_score_in_db(&ctx.pool, user.id).await;

    if let None = score_id {
        warn!("no score id");
        return "error: no".to_string();
    }

    let score_id = score_id.unwrap();

    let new_score = get_score_by_id(
        &ctx.pool,
        score_id,
        decrypted_score.beatmap_md5.clone(),
        &osu_mode,
        Some(current_score.status.to_db()),
    )
    .await;

    if let Err(error) = new_score {
        warn!("Score insertion failed {:#?}", error);
        return "error: no".to_string();
    }
    let new_score = new_score.unwrap();

    if let None = new_score {
        warn!("{}", score_id);
        warn!("no score");
        return "error: no".to_string();
    }

    let new_score = new_score.unwrap();

    let replay = form_data.get_file("score");

    // if let Some(replay) = replay {

    // }else{

    // }
    match replay {
        Some(replay_bytes) => {
            let replay_path = Path::new("data")
                .join("replays")
                .join(format!("{}.osr_frames", score_id));
            if !fs::metadata(&replay_path).await.is_ok() {
                if let Err(error) = fs::create_dir_all(Path::new("data").join("replays")).await {
                    error!("Unable to create data folder: {}", error);
                    return "error: no".to_string();
                }
            }

            if let Err(error) = fs::write(
                format!("data/replays/{}.osr_frames", score_id),
                replay_bytes,
            )
            .await
            {
                warn!("Unable to write replay file: {}", error);
                return "error: no".to_string();
            }
        }
        None => {
            if !decrypted_score.failed && !quit {
                insert_user_punishment(
                    &ctx.pool,
                    "CRITICAL".to_string(),
                    1,
                    user.id,
                    "RESTRICTION".to_string(),
                    false,
                    NaiveDateTime::UNIX_EPOCH,
                    "Lia: Hasn't sent a replay file.".to_string(),
                )
                .await;
                restrict_user(&ctx.pool, user.id).await;
                send_bancho_message(&user.id, "user:restricted".to_string(), None).await;
            }
        }
    }

    //Everything works fine, prcessing everything
    current_score.id = Some(score_id);
    let stats_before = get_user_stats(&ctx.pool, &user.id, &osu_mode)
        .await
        .unwrap();

    increase_user_score(
        &ctx.pool,
        osu_mode.clone(),
        decrypted_score.total_score as i64,
        &user.id,
    )
    .await;
    increase_user_playcount(&ctx.pool, osu_mode.clone(), &user.id).await;

    if decrypted_score.max_combo > (stats_before.max_combo as u16) {
        update_user_max_combo(
            &ctx.pool,
            &osu_mode.clone(),
            &user.id,
            decrypted_score.max_combo as i16,
        )
        .await;
    }

    let rank_before = get_rank(&ctx.redis, &user, &osu_mode).await.unwrap_or(0);

    recalculate_user_stats(&ctx.pool, &ctx.redis, &user, &osu_mode).await;

    let rank_after = get_rank(&ctx.redis, &user, &osu_mode).await.unwrap_or(0);
    let stats_after = get_user_stats(&ctx.pool, &user.id, &osu_mode)
        .await
        .unwrap();

    let beatmap_chart = Chart {
        chart_id: "beatmap".to_string(),
        chart_url: format!("https://lisek.world/b/{}", beatmap.clone().beatmap_id),
        chart_name: "Beatmap Ranking".to_string(),
        achievements: "".to_ascii_lowercase(),
        score_id,
        rank_before: best_score
            .clone()
            .and_then(|x| Some(x.score.rank))
            .unwrap_or(0) as i32,
        rank_after: new_score.rank,
        accruacy_before: best_score
            .clone()
            .and_then(|x| Some(x.score.calculate_accuracy()))
            .unwrap_or(0.0),
        accuracy_after: new_score.score.calculate_accuracy(),
        ranked_score_before: best_score
            .clone()
            .and_then(|x| Some(x.score.total_score))
            .unwrap_or(0) as i64,
        ranked_score_after: new_score.score.total_score as i64,
        combo_before: best_score
            .clone()
            .and_then(|x| Some(x.score.max_combo))
            .unwrap_or(0) as i32,
        combo_after: new_score.score.max_combo,
        total_score_before: best_score
            .clone()
            .and_then(|x| Some(x.score.total_score))
            .unwrap_or(0) as i64,
        total_score_after: new_score.score.total_score as i64,
        performance_before: best_score
            .clone()
            .and_then(|x| Some(x.score.performance))
            .unwrap_or(0.0) as f64,
        performance_after: new_score.score.performance,
    };

    let overall_chart = Chart {
        chart_id: "overall".to_string(),
        chart_url: format!("https://lisek.world/u/{}", user.id),
        chart_name: "Overall Ranking".to_string(),
        achievements: "".to_ascii_lowercase(),
        score_id,
        rank_before: rank_before,
        rank_after: rank_after,
        accruacy_before: stats_before.accuracy * 100.0,
        accuracy_after: stats_after.accuracy * 100.0,
        ranked_score_before: stats_before.ranked_score,
        ranked_score_after: stats_after.ranked_score,
        combo_before: stats_before.max_combo,
        combo_after: stats_after.max_combo,
        total_score_before: stats_before.total_score,
        total_score_after: stats_after.total_score,
        performance_before: stats_before.performance,
        performance_after: stats_after.performance,
    };

    info!("new rank: {}", new_score.rank);
    info!("Score status: {}", new_score.score.status);

    if let Ok(_redis) = ctx.redis.get_connection() {
        // let _: Result<i32, redis::RedisError> = redis.publish("users:refresh", user.id.to_string());
        send_bancho_message(&user.id, "user:refresh".to_string(), None).await;
    }

    if new_score.rank == 1
        && !is_restricted(&user).await
        && (new_score.score.status == 2 || new_score.score.status == 4)
    {
        debug!("Sending announcement and checking for pp cap");

        if is_cap_reached(&new_score) {
            //Restricting user due to cap
            insert_user_punishment(
                &ctx.pool,
                "CRITICAL".to_string(),
                1,
                user.id,
                "RESTRICTION".to_string(),
                false,
                NaiveDateTime::UNIX_EPOCH,
                format!(
                    "Lia: user has reached pp cap (score_id: {}).",
                    new_score.score.id
                ),
            )
            .await;
            restrict_user(&ctx.pool, user.id).await;
        }

        let performance_string = beatmap
            .status
            .eq(&2)
            .then(|| format!("({:.2}pp)", performance))
            .unwrap_or("".to_string());
        let formatted_announce_message = format!("[https://{}/users/{} {}] has just achieved #1 {} on [https://{}/b/{} {} - {} [{}]] ({})", ctx.config.server_url, user.id, user.username, performance_string, ctx.config.server_url, beatmap.beatmap_id, beatmap.artist, beatmap.title, beatmap.version, &osu_mode.to_string().clone());

        //Announcing it
        send_message_announcement(
            format!(
                "https://c.{}/api/v2/bancho/notification",
                ctx.config.server_url
            ),
            formatted_announce_message,
            "chat".to_string(),
            "#announce".to_string(),
            ctx.config.token_hmac_secret.clone(),
        )
        .await;
    }

    if new_score.score.status == 2 && new_score.score.playmode == 4 {
        send_message_announcement(
            format!(
                "https://c.{}/api/v2/bancho/notification",
                ctx.config.server_url
            ),
            format!(
                "Submitted {:.2}pp (+{:.2})",
                performance,
                stats_after.performance - stats_before.performance
            ),
            "notification".to_string(),
            user.username.clone(),
            ctx.config.token_hmac_secret.clone(),
        )
        .await;
    }

    Chart::build(&beatmap, beatmap_chart, overall_chart)
}
