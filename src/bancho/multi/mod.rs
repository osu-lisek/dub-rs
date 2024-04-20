use std::{fmt::Debug, path::Display, sync::Arc};

use bancho_packets::{BanchoPacketWrite, MatchData, MatchUpdate, PayloadReader};
use hmac::digest::typenum::uint;
use tokio::sync::Mutex;
use tracing::info;

use crate::utils::{beatmap_utils::Beatmap, http_utils::OsuMode};

use super::{bancho_manager::BanchoManager, channel_manager::ChannelManager, presence::Presence};

#[derive(Clone, PartialEq)]
pub enum SlotStatus {
    Open = 1,
    Locked = 2,
    NotReady = 4,
    Ready = 8,
    NoMap = 16,
    Playing = 32,
    Complete = 64,
    Quit = 128,
}

impl From<u8> for SlotStatus {
    fn from(value: u8) -> Self {
        match value as i32 {
            1 => SlotStatus::Open,
            2 => SlotStatus::Locked,
            4 => SlotStatus::NotReady,
            8 => SlotStatus::Ready,
            16 => SlotStatus::NoMap,
            32 => SlotStatus::Playing,
            64 => SlotStatus::Complete,
            128 => SlotStatus::Quit,
            _ => SlotStatus::Open,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum SlotTeam {
    Neutral = 0,
    Blue = 1,
    Red = 2,
}

impl From<u8> for SlotTeam {
    fn from(value: u8) -> Self {
        match value as i32 {
            0 => SlotTeam::Neutral,
            1 => SlotTeam::Blue,
            2 => SlotTeam::Red,
            _ => SlotTeam::Neutral,
        }
    }
}

pub struct Multislot {
    pub finished: bool,
    pub presence: Mutex<Option<Arc<Presence>>>,
    pub status: Mutex<SlotStatus>,
    pub team: Mutex<SlotTeam>,
    pub mods: Mutex<u32>,
    pub skipped: Mutex<bool>,
    pub loaded: Mutex<bool>,
}

#[derive(Clone)]
pub struct Multiroom {
    bancho_manager: Arc<BanchoManager>,
    channel_manager: Arc<ChannelManager>,
    pub id: Arc<Mutex<i32>>,
    pub slots: Arc<Mutex<Vec<Multislot>>>,
    pub name: Arc<Mutex<String>>,
    pub password: Arc<Mutex<String>>,

    pub beatmap_name: Arc<Mutex<String>>,
    pub beatmap_id: Arc<Mutex<i32>>,
    pub beatmap_md5: Arc<Mutex<String>>,

    pub room_type: i32,        //ussualy 1
    pub mods: Arc<Mutex<u32>>, //ussualy 1
    pub host: Arc<Mutex<Option<Arc<Presence>>>>,
    pub team: Arc<Mutex<Option<Beatmap>>>,
    pub in_progress: Arc<Mutex<bool>>,
    pub team_type: Arc<Mutex<u8>>,
    pub freemode: Arc<Mutex<bool>>,
    pub seed: Arc<Mutex<i32>>,
    pub mode: Arc<Mutex<OsuMode>>,
    pub scoring: Arc<Mutex<u8>>,
}

impl Multiroom {
    pub async fn display(&self) -> String {
        return format!("Room with name: {}", self.name.lock().await);
    }
}

impl Multiroom {
    pub fn new(
        bancho_manager: Arc<BanchoManager>,
        channel_manager: Arc<ChannelManager>,
        id: i32,
        name: String,
    ) -> Self {
        let mut slots = Vec::new();

        for i in 1..16 {
            slots.push(Multislot {
                finished: false,
                loaded: Mutex::new(false),
                mods: Mutex::new(0),
                presence: Mutex::new(None),
                skipped: Mutex::new(false),
                status: Mutex::new(SlotStatus::Open),
                team: Mutex::new(SlotTeam::Neutral),
            })
        }

        Self {
            bancho_manager: bancho_manager,
            channel_manager: channel_manager,
            id: Arc::new(Mutex::new(id)),
            slots: Arc::new(Mutex::new(slots)),
            name: Arc::new(Mutex::new(name)),
            password: Arc::new(Mutex::new(String::new())),
            room_type: 0,
            host: Arc::new(Mutex::new(None)),
            team: Arc::new(Mutex::new(None)),
            in_progress: Arc::new(Mutex::new(false)),
            team_type: Arc::new(Mutex::new(0)),
            freemode: Arc::new(Mutex::new(false)),
            seed: Arc::new(Mutex::new(0)),
            mode: Arc::new(Mutex::new(OsuMode::Osu)),
            scoring: Arc::new(Mutex::new(0)),
            mods: Arc::new(Mutex::new(0)),
            beatmap_name: Arc::new(Mutex::new(String::new())),
            beatmap_id: Arc::new(Mutex::new(0)),
            beatmap_md5: Arc::new(Mutex::new(String::new())),
        }
    }

    pub async fn to_buffer(&mut self, buffer: Vec<u8>) {
        let writer = MatchUpdate {
            send_password: true,
            data: MatchData {
                match_id: self.id.lock().await,
                in_progress: self.in_progress.lock().await,
                match_type: 1,
                play_mods: self.mode.lock().await,
                match_name: self.name.lock().await,
                password: self.password.lock().await,
                beatmap_name: self.beatmap_name.lock().await,
                beatmap_id: self.beatmap_id.lock().await,
                beatmap_md5: self.beatmap_md5.lock().await.to_string(),
                slot_status: self.slots.lock().await.iter().map(|x| x.status).collect(),
                slot_teams: (),
                slot_players: (),
                host_player_id: (),
                match_game_mode: (),
                win_condition: (),
                team_type: (),
                freemods: (),
                player_mods: (),
                match_seed: (),
            },
        };

        writer.into_packet()
    }

    pub async fn from_buffer(&mut self, buffer: Vec<u8>) {
        let mut reader = PayloadReader::new(&buffer);
        let _match_id = reader.read::<u16>();
        let in_progress = reader.read::<bool>();
        let _match_type = reader.read::<i8>();
        let mods = reader.read::<u32>();
        let match_name = reader.read::<String>();
        let password: Option<String> = reader.read::<String>();

        let beatmap_name: Option<String> = reader.read::<String>();
        let beatmap_id: Option<i32> = reader.read::<i32>();
        let beatmap_md5: Option<String> = reader.read::<String>();

        *self.in_progress.lock().await = in_progress.unwrap_or(false);
        *self.mods.lock().await = mods.unwrap_or(0);
        *self.name.lock().await = match_name.unwrap_or(String::new());
        *self.password.lock().await = password.unwrap_or(String::new());

        *self.beatmap_name.lock().await = beatmap_name.unwrap_or_default();
        *self.beatmap_id.lock().await = beatmap_id.unwrap_or_default();
        *self.beatmap_md5.lock().await = beatmap_md5.unwrap_or_default();

        for slot in self.slots.lock().await.iter() {
            *slot.status.lock().await = SlotStatus::from(reader.read::<u8>().unwrap_or(1));
        }

        for slot in self.slots.lock().await.iter() {
            *slot.team.lock().await = SlotTeam::from(reader.read::<u8>().unwrap_or(1));
        }

        for slot in self.slots.lock().await.iter() {
            let status = slot.status.lock().await.clone();
            if status != SlotStatus::Open && status != SlotStatus::Locked {
                let user_id = reader.read::<i32>().unwrap_or(0);
                let presence = self.bancho_manager.get_presence_by_user_id(user_id).await;
                if let Some(presence) = presence {
                    *slot.presence.lock().await = Some(presence.clone());
                }
            }
        }

        let host_id = reader.read::<i32>();
        if let Some(host_id) = host_id {
            //Trying to lookup it inside slots
            for slot in self.slots.lock().await.iter() {
                let presence = slot.presence.lock().await.clone();
                if presence.clone().is_none() {
                    continue;
                }

                let presence = presence.unwrap();

                if presence.user.id == host_id {
                    *self.host.lock().await = Some(presence);
                }
            }
        }

        let mode = reader.read::<u8>();
        let scoring = reader.read::<u8>();
        let team_type = reader.read::<u8>();
        let freemode = reader.read::<bool>();

        if let Some(freemode) = freemode {
            if freemode {
                for slot in self.slots.lock().await.iter() {
                    *slot.mods.lock().await = reader.read::<u32>().unwrap_or(0);
                }
            }
        }

        let seed = reader.read::<i32>();

        *self.mode.lock().await = OsuMode::from_id(mode.unwrap_or(0));
        *self.scoring.lock().await = scoring.unwrap_or(0);
        *self.team_type.lock().await = team_type.unwrap_or(0);
        *self.freemode.lock().await = freemode.unwrap_or(false);
        *self.seed.lock().await = seed.unwrap_or(0);
    }
}
