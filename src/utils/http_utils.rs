use serde::Deserialize;
use serde_repr::*;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum OsuMode {
    Osu = 0,
    Taiko = 1,
    Fruits = 2,
    Mania = 3,
    Relax = 4,
}
impl OsuMode {
    pub fn to_osu(&self) -> i32 {
        match &self {
            OsuMode::Osu => 0,
            OsuMode::Taiko => 1,
            OsuMode::Fruits => 2,
            OsuMode::Mania => 3,
            OsuMode::Relax => 4,
        }
    }

    pub fn to_db_suffix(&self) -> &str {
        match &self {
            OsuMode::Osu => "Std",
            OsuMode::Taiko => "Taiko",
            OsuMode::Fruits => "Ctb",
            OsuMode::Mania => "Mania",
            OsuMode::Relax => "Rx",
        }
    }

    pub fn from_id(id: u8) -> OsuMode {
        match id {
            0 => OsuMode::Osu,
            1 => OsuMode::Taiko,
            2 => OsuMode::Fruits,
            3 => OsuMode::Mania,
            4 => OsuMode::Relax,
            _ => OsuMode::Osu,
        }
    }

    pub fn to_string(&self) -> String {
        match &self {
            OsuMode::Osu => "osu!",
            OsuMode::Taiko => "osu:taiko",
            OsuMode::Fruits => "osu!catch",
            OsuMode::Mania => "osu!mania",
            OsuMode::Relax => "osu!rx",
        }
        .to_string()
    }
}

#[derive(Deserialize, Clone)]
pub struct ScoreRequestQuery {
    #[serde(rename = "us")]
    pub username: String,
    #[serde(rename = "ha")]
    pub password: String,
    #[serde(rename = "vv")]
    pub leaderboard_version: i32,
    #[serde(rename = "c")]
    pub beatmap_hash: String,
    #[serde(rename = "i")]
    pub set_id: String,
    #[serde(rename = "f")]
    pub filename: String,
    #[serde(rename = "m")]
    pub mode: OsuMode,
    #[serde(rename = "v")]
    pub leaderboard_type: i32,
    #[serde(rename = "mods")]
    pub mods: i32, // TODO: Make enum
}
