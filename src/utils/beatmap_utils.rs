use std::{
    fs::File,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{prelude::FromRow, Pool, Postgres};
use tracing::{error, info, warn};
use webhook::client::WebhookClient;

use crate::{bancho::presence::Presence, db::user::User, web::scores::submission::BeatmapStatus};

use super::{
    general_utils::to_fixed,
    http_utils::OsuMode,
    score_utils::{format_mods, OsuServerError, UserScoreWithBeatmap},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineBeatmapset {
    pub artist: String,
    #[serde(rename = "artist_unicode")]
    pub artist_unicode: String,
    pub covers: Covers,
    pub creator: String,
    #[serde(rename = "favourite_count")]
    pub favourite_count: i64,
    pub hype: Option<Hype>,
    pub id: i64,
    pub nsfw: bool,
    pub offset: i64,
    #[serde(rename = "play_count")]
    pub play_count: i64,
    #[serde(rename = "preview_url")]
    pub preview_url: String,
    pub source: String,
    pub spotlight: bool,
    pub status: String,
    pub title: String,
    #[serde(rename = "title_unicode")]
    pub title_unicode: String,
    #[serde(rename = "track_id")]
    pub track_id: Value,
    #[serde(rename = "user_id")]
    pub user_id: i64,
    pub video: bool,
    pub bpm: f64,
    #[serde(rename = "can_be_hyped")]
    pub can_be_hyped: bool,
    #[serde(rename = "deleted_at")]
    pub deleted_at: Value,
    #[serde(rename = "discussion_enabled")]
    pub discussion_enabled: bool,
    #[serde(rename = "discussion_locked")]
    pub discussion_locked: bool,
    #[serde(rename = "is_scoreable")]
    pub is_scoreable: bool,
    #[serde(rename = "last_updated")]
    pub last_updated: String,
    #[serde(rename = "legacy_thread_url")]
    pub legacy_thread_url: String,
    #[serde(rename = "nominations_summary")]
    pub nominations_summary: Option<NominationsSummary>,
    pub ranked: i64,
    #[serde(rename = "ranked_date")]
    pub ranked_date: Value,
    pub storyboard: bool,
    #[serde(rename = "submitted_date")]
    pub submitted_date: String,
    pub tags: String,
    pub availability: Availability,
    #[serde(rename = "has_favourited")]
    pub has_favourited: bool,
    pub beatmaps: Vec<OnlineBeatmap>,
    #[serde(rename = "pack_tags")]
    pub pack_tags: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Covers {
    pub cover: String,
    #[serde(rename = "cover@2x")]
    pub cover_2x: String,
    pub card: String,
    #[serde(rename = "card@2x")]
    pub card_2x: String,
    pub list: String,
    #[serde(rename = "list@2x")]
    pub list_2x: String,
    pub slimcover: String,
    #[serde(rename = "slimcover@2x")]
    pub slimcover_2x: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hype {
    pub current: i64,
    pub required: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NominationsSummary {
    pub current: i64,
    pub required: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    #[serde(rename = "download_disabled")]
    pub download_disabled: bool,
    #[serde(rename = "more_information")]
    pub more_information: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineBeatmap {
    #[serde(rename = "beatmapset_id")]
    pub beatmapset_id: i64,
    #[serde(rename = "difficulty_rating")]
    pub difficulty_rating: f64,
    pub id: i64,
    pub mode: String,
    pub status: String,
    #[serde(rename = "total_length")]
    pub total_length: i64,
    #[serde(rename = "user_id")]
    pub user_id: i64,
    pub version: String,
    pub accuracy: f64,
    pub ar: f64,
    pub bpm: f32,
    pub convert: bool,
    #[serde(rename = "count_circles")]
    pub count_circles: i64,
    #[serde(rename = "count_sliders")]
    pub count_sliders: i64,
    #[serde(rename = "count_spinners")]
    pub count_spinners: i64,
    pub cs: f64,
    #[serde(rename = "deleted_at")]
    pub deleted_at: Value,
    pub drain: f64,
    #[serde(rename = "hit_length")]
    pub hit_length: i64,
    #[serde(rename = "is_scoreable")]
    pub is_scoreable: bool,
    #[serde(rename = "last_updated")]
    pub last_updated: String,
    #[serde(rename = "mode_int")]
    pub mode_int: i64,
    pub passcount: i64,
    pub playcount: i64,
    pub ranked: i64,
    pub url: String,
    pub checksum: String,
    #[serde(rename = "max_combo")]
    pub max_combo: i64,
}

#[derive(Debug, Serialize)]
pub struct PublicBeatmap {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub version: String,
    pub bpm: f64,
    pub ar: f64,
    pub od: f64,
    pub cs: f64,
    pub hp: f64,
    pub status: i32,
    pub max_combo: i32,
    pub total_length: i32,
}

#[derive(FromRow, Debug, Clone)]
pub struct Beatmap {
    pub title: String,
    #[sqlx(rename = "titleUnicode")]
    pub title_unicode: String,
    pub artist: String,
    #[sqlx(rename = "artistUnicode")]
    pub artist_unicode: String,
    pub creator: String,
    pub version: String,
    #[sqlx(rename = "parentId")]
    pub parent_id: i32,
    #[sqlx(rename = "beatmapId")]
    pub beatmap_id: i32,
    pub ar: f64,
    pub od: f64,
    pub cs: f64,
    pub hp: f64,
    pub stars: f64,
    #[sqlx(rename = "gameMode")]
    pub game_mode: i32,
    pub bpm: f64,
    #[sqlx(rename = "maxCombo")]
    pub max_combo: i32,
    #[sqlx(rename = "hitLength")]
    pub hit_length: i32,
    #[sqlx(rename = "totalLength")]
    pub total_length: i32,
    pub status: i32,
    pub frozen: bool,
    pub checksum: String,
}

impl Beatmap {
    pub fn to_public(self) -> PublicBeatmap {
        PublicBeatmap {
            id: self.beatmap_id,
            parent_id: Some(self.parent_id),
            artist: self.artist,
            title: self.title,
            creator: self.creator,
            version: self.version,
            bpm: self.bpm,
            ar: to_fixed(self.ar, 2),
            od: to_fixed(self.od, 2),
            cs: to_fixed(self.cs, 2),
            hp: to_fixed(self.hp, 2),
            status: self.status,
            max_combo: self.max_combo,
            total_length: self.total_length,
        }
    }
    pub async fn insert_in_db(self, pool: &Pool<Postgres>) {
        let result = sqlx::query!(
            r#"INSERT INTO "Beatmap"
        ("title", "titleUnicode",
        "artist", "artistUnicode",
        "creator", "version",
        "parentId", "beatmapId",
        "ar", "od", "cs", "hp",
        "stars",
        "gameMode",
        "bpm",
        "maxCombo",
        "hitLength",
        "totalLength",
        "status",
        "frozen",
        "checksum") VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)"#,
            self.title,
            self.title_unicode,
            self.artist,
            self.artist_unicode,
            self.creator,
            self.version,
            self.parent_id,
            self.beatmap_id,
            self.ar,
            self.od,
            self.cs,
            self.hp,
            self.stars,
            self.game_mode,
            self.bpm,
            self.max_combo,
            self.hit_length,
            self.total_length,
            self.status,
            self.frozen,
            self.checksum
        ).execute(pool).await;

        match result {
            Err(e) => {
                error!("Error while inserting beatmap: {}", e);
            }
            Ok(_) => {
                info!("Beatmap inserted.")
            }
        }
    }
}

pub async fn get_beatmap_by_hash(
    connection: &Pool<Postgres>,
    checksum: String,
) -> Result<Option<Beatmap>, OsuServerError> {
    let beatmap: Result<_, sqlx::Error> =
        sqlx::query_as::<_, Beatmap>("SELECT * FROM \"Beatmap\" WHERE \"checksum\" = $1")
            .bind(checksum)
            .fetch_one(connection)
            .await;

    match beatmap {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            err => {
                error!("Failed to fetch beatmap from database: {}", err);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(beatmap) => Ok(Some(beatmap)),
    }
}

pub async fn get_beatmap_by_term(
    connection: &Pool<Postgres>,
    term: String,
) -> Result<Option<Beatmap>, OsuServerError> {
    let beatmap: Result<_, sqlx::Error> = sqlx::query_as::<_, Beatmap>(
        r#"SELECT * FROM "Beatmap" WHERE "beatmapId" = $1 OR "checksum" = $2"#,
    )
    .bind(term.parse::<i64>().unwrap_or(-1))
    .bind(term)
    .fetch_one(connection)
    .await;

    match beatmap {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            err => {
                error!("Failed to fetch beatmap from database: {}", err);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(beatmap) => Ok(Some(beatmap)),
    }
}

pub async fn get_beatmap_by_id(
    connection: &Pool<Postgres>,
    id: i64,
) -> Result<Option<Beatmap>, OsuServerError> {
    let beatmap: Result<_, sqlx::Error> =
        sqlx::query_as::<_, Beatmap>("SELECT * FROM \"Beatmap\" WHERE \"beatmapId\" = $1")
            .bind(id)
            .fetch_one(connection)
            .await;

    match beatmap {
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            err => {
                error!("Failed to fetch beatmap from database: {}", err);
                Err(OsuServerError::Internal("Failed to fetch.".to_string()))
            }
        },
        Ok(beatmap) => Ok(Some(beatmap)),
    }
}

pub async fn get_beatmap_file(id: i64) -> Result<Option<Vec<u8>>, OsuServerError> {
    //Checking if it exist in path .data/beatmaps/{}.osu

    let f = File::open(format!(".data/beatmaps/{}.osu", id));

    if let Err(error) = f {
        match error.kind() {
            std::io::ErrorKind::NotFound => match force_download_beatmap_by_id(id).await {
                Ok(bytes) => return Ok(Some(bytes)),
                Err(error) => {
                    error!("Failed to download beatmap file: {:#?}", error);
                    return Err(OsuServerError::BeatmapProcessingFailed(
                        "Failed to download beatmap file.".to_string(),
                    ));
                }
            },
            _ => {
                error!("Failed to open beatmap file: {}", error);
                return Err(OsuServerError::BeatmapProcessingFailed(
                    "Failed to open beatmap file.".to_string(),
                ));
            }
        }
    }

    let mut f = f.unwrap();

    let mut buffer = Vec::new();
    let bytes = f.read_to_end(&mut buffer);
    if let Err(error) = bytes {
        error!("Failed to read beatmap file: {}", error);
        return Err(OsuServerError::BeatmapProcessingFailed(
            "Failed to read beatmap file.".to_string(),
        ));
    }

    Ok(Some(buffer))
}

pub async fn force_download_beatmap_by_id(id: i64) -> Result<Vec<u8>, OsuServerError> {
    let response = reqwest::get(format!("https://osu.ppy.sh/osu/{}", id)).await;

    if let Err(error) = response {
        error!("Failed to download beatmap: {}", error);
        return Err(OsuServerError::BeatmapProcessingFailed(
            "Failed to download beatmap.".to_string(),
        ));
    }

    let response = response.unwrap();

    let status = response.status();

    if !status.is_success() {
        error!("Failed to download beatmap: {}", status);
        return Err(OsuServerError::BeatmapProcessingFailed(
            "Failed to download beatmap.".to_string(),
        ));
    }

    let bytes = response.bytes().await;

    if let Err(error) = bytes {
        error!("Failed to download beatmap: {}", error);
        return Err(OsuServerError::BeatmapProcessingFailed(
            "Failed to download beatmap.".to_string(),
        ));
    }

    let bytes = bytes.unwrap();

    //Saving it
    let mut file = File::create(format!("data/beatmaps/{}.osu", id)).unwrap();
    file.write_all(&bytes).unwrap();
    file.flush().unwrap();

    Ok(bytes.to_vec())
}

pub async fn _get_online_beatmap_by_id(id: i64) -> Result<Beatmap, OsuServerError> {
    let response = reqwest::get(format!(
        "https://mirror.lisek.cc/api/v1/beatmapsets/beatmap/{}",
        id
    ))
    .await;
    if let Err(error) = response {
        return Err(OsuServerError::FailedToFetch(format!(
            "Failed to fetch beatmap: {}",
            error
        )));
    }

    let response = response.unwrap();

    let data = response.json::<OnlineBeatmapset>().await;

    if let Err(error) = data {
        return Err(OsuServerError::FailedToFetch(format!(
            "Failed to process beatmap response: {}",
            error
        )));
    }

    let data = data.unwrap();

    let beatmap = data.beatmaps.iter().find(|x| x.id == id);

    if beatmap.is_none() {
        return Err(OsuServerError::FailedToFetch(
            "Failed to fetch beatmap, no beatmap in set".to_string(),
        ));
    }

    let beatmap = beatmap.unwrap();

    Ok(Beatmap {
        title: data.title,
        title_unicode: data.title_unicode,
        artist: data.artist,
        artist_unicode: data.artist_unicode,
        creator: data.creator,
        version: beatmap.version.to_string(),
        parent_id: data.id as i32,
        beatmap_id: id as i32,
        ar: beatmap.ar,
        od: beatmap.accuracy,
        cs: beatmap.cs,
        hp: beatmap.drain,
        stars: beatmap.difficulty_rating,
        game_mode: beatmap.mode_int as i32,
        bpm: beatmap.bpm as f64,
        max_combo: beatmap.max_combo as i32,
        hit_length: beatmap.hit_length as i32,
        total_length: beatmap.total_length as i32,
        status: match beatmap.ranked {
            -2 => 0,
            -1 => 0,
            0 => 0,
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 5,
            _ => 0,
        },
        frozen: false,
        checksum: beatmap.checksum.to_string(),
    })
}

pub async fn get_online_beatmap_by_checksum(checksum: String) -> Result<Beatmap, OsuServerError> {
    let response = reqwest::get(format!(
        "https://mirror.lisek.cc/api/v1/beatmaps/md5/{}",
        checksum
    ))
    .await;
    if let Err(error) = response {
        return Err(OsuServerError::FailedToFetch(format!(
            "Failed to fetch beatmap: {}",
            error
        )));
    }

    let response = response.unwrap();
    let r = response
        .text()
        .await
        .unwrap_or("".to_string())
        .as_str()
        .to_owned();
    let jd = &mut serde_json::Deserializer::from_str(r.as_str());

    let data = serde_path_to_error::deserialize(jd);
    // let data = response.json::<OnlineBeatmapset>().await;

    if let Err(error) = data {
        return Err(OsuServerError::FailedToFetch(format!(
            "Failed to process beatmap response: {}",
            error
        )));
    }

    let data: OnlineBeatmapset = data.unwrap();

    let beatmap = data.beatmaps.iter().find(|x| x.checksum == checksum);

    if beatmap.is_none() {
        return Err(OsuServerError::FailedToFetch(
            "Failed to fetch beatmap, no beatmap in set".to_string(),
        ));
    }

    let beatmap = beatmap.unwrap();

    Ok(Beatmap {
        title: data.title,
        title_unicode: data.title_unicode,
        artist: data.artist,
        artist_unicode: data.artist_unicode,
        creator: data.creator,
        version: beatmap.version.to_string(),
        parent_id: data.id as i32,
        beatmap_id: beatmap.id as i32,
        ar: beatmap.ar,
        od: beatmap.accuracy,
        cs: beatmap.cs,
        hp: beatmap.drain,
        stars: beatmap.difficulty_rating,
        game_mode: beatmap.mode_int as i32,
        bpm: beatmap.bpm as f64,
        max_combo: beatmap.max_combo as i32,
        hit_length: beatmap.hit_length as i32,
        total_length: beatmap.total_length as i32,
        status: match beatmap.ranked {
            -2 => 0,
            -1 => 0,
            0 => 0,
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 5,
            _ => 0,
        },
        frozen: false,
        checksum: beatmap.checksum.to_string(),
    })
}

pub async fn _get_online_beatmapset_by_id(id: i64) -> Result<OnlineBeatmapset, OsuServerError> {
    let response = reqwest::get(format!("https://mirror.lisek.cc/api/v1/beatmapsets/{}", id)).await;
    if let Err(error) = response {
        return Err(OsuServerError::FailedToFetch(format!(
            "Failed to fetch beatmap: {}",
            error
        )));
    }

    let response = response.unwrap();

    let data = response.json::<OnlineBeatmapset>().await;

    if let Err(error) = data {
        return Err(OsuServerError::FailedToFetch(format!(
            "Failed to process beatmap response: {}",
            error
        )));
    }

    Ok(data.unwrap())
}

pub fn rank_to_str(status: &BeatmapStatus) -> &'static str {
    match status {
        BeatmapStatus::Unknown => "Unknown",
        BeatmapStatus::NotSubmitted => "Unsubmitted",
        BeatmapStatus::LatestPending => "Unranked",
        BeatmapStatus::NeedUpdate => "Needs Update",
        BeatmapStatus::Ranked => "Ranked",
        BeatmapStatus::Approved => "Approved",
        BeatmapStatus::Qualified => "Qualified",
        BeatmapStatus::Loved => "Loved",
    }
}

pub async fn announce_beatmap_status(author: &Presence, beatmap: &Beatmap, status: &BeatmapStatus) {
    let old_status = rank_to_str(&BeatmapStatus::from(beatmap.status));
    let new_status = rank_to_str(status);

    let client = WebhookClient::new(std::env::var("DISCORD_BEATMAPS").unwrap().as_str());

    if let Err(_error) = client
        .send(|message| {
            message.username("lisek.world/beatmaps").embed(|embed| {
                embed
                    .author(
                        &author.user.username_safe,
                        Some(format!("https://lisek.world/users/{}", author.user.id)),
                        Some(format!("https://a.lisek.world/{}", author.user.id)),
                    )
                    .url(format!("https://lisek.world/b/{}", beatmap.parent_id).as_str())
                    .image(
                        format!(
                            "https://assets.ppy.sh/beatmaps/{}/covers/card@2x.jpg",
                            beatmap.parent_id
                        )
                        .as_str(),
                    )
                    .title(format!("{} - {}", beatmap.artist, beatmap.title).as_str())
                    .footer(format!("{} to {}", old_status, new_status).as_str(), None)
                    .field(
                        "Circle Size (CS)",
                        format!("{:.2}", beatmap.cs).as_str(),
                        true,
                    )
                    .field("HP Drain (HP)", format!("{:.2}", beatmap.hp).as_str(), true)
                    .field("Accuracy (OD)", format!("{:.2}", beatmap.od).as_str(), true)
                    .field(
                        "Approach Rate (AR)",
                        format!("{:.2}", beatmap.ar).as_str(),
                        true,
                    )
                    .field("BPM", format!("{:.2}", beatmap.bpm).as_str(), true)
                    .field(
                        "Star Rating (SR)",
                        format!("{:.2}", beatmap.stars).as_str(),
                        true,
                    )
            })
        })
        .await
    {
        warn!("Failed: {}", _error);
    }
}

pub async fn announce_insane_score(author: &User, score: &UserScoreWithBeatmap) {
    let client = WebhookClient::new(std::env::var("DISCORD_GENERIC").unwrap().as_str());

    if let Err(_error) = client
        .send(|message| {
            message.username("lisek.world/generic").embed(|embed| {
                embed
                    .author(
                        &author.username_safe,
                        Some(format!("https://lisek.world/users/{}", author.id)),
                        Some(format!("https://a.lisek.world/{}", author.id)),
                    )
                    .title(
                        format!(
                            "{} - {} [{}]",
                            score.beatmap.artist, score.beatmap.title, score.beatmap.version
                        )
                        .as_str(),
                    )
                    .url(format!("https://lisek.world/b/{}", score.beatmap.parent_id).as_str())
                    .image(
                        format!(
                            "https://assets.ppy.sh/beatmaps/{}/covers/card@2x.jpg",
                            score.beatmap.parent_id
                        )
                        .as_str(),
                    )
                    .footer(
                        OsuMode::from_id(score.score.playmode as u8)
                            .to_string()
                            .as_str(),
                        None,
                    )
                    .field(
                        "Mods",
                        format!("+{}", &format_mods(score.score.mods as u32)).as_str(),
                        true,
                    )
                    .field(
                        "Accuracy",
                        format!(
                            "{:.2}% ({})",
                            score.score.calculate_accuracy(),
                            score.score.calculate_grade()
                        )
                        .as_str(),
                        true,
                    )
                    .field(
                        "PP",
                        format!("{:.2}pp", score.score.performance).as_str(),
                        true,
                    )
            })
        })
        .await
    {
        warn!("Failed: {}", _error);
    }
}
