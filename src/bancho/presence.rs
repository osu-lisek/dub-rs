use std::sync::Arc;

use bancho_packets::{
    server::{
        FellowSpectatorJoined, FellowSpectatorLeft, Notification, SilenceEnd, SpectatorFrames,
        SpectatorJoined, SpectatorLeft, UserStats,
    },
    BanchoMessage, BanchoPacket, ClientChangeAction,
};
use chrono::{DateTime, Utc};
use redis::Client;
use sqlx::{Pool, Postgres};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    context::Context,
    db::user::User,
    utils::{
        http_utils::OsuMode,
        user_utils::{get_rank, get_user_stats},
        UserDbStats,
    },
};

use super::{bancho_manager::BanchoManager, channel_manager::ChannelManager, client::ClientData};

#[derive(Debug, Clone)]
pub struct Presence {
    pub token: String,
    pub user: User,
    pub client_data: ClientData,
    packet_queue: Arc<Mutex<Vec<u8>>>,
    pub status: Arc<RwLock<ClientChangeAction>>,
    pub country: u8,
    pub lat: f32,
    pub lon: f32,
    cached_stats: Arc<RwLock<UserDbStats>>,

    previous_message: Arc<Mutex<String>>,
    previous_message_repeated: Arc<Mutex<i32>>,
    pub silenced_until: Arc<RwLock<i64>>,

    pub spectating: Arc<Mutex<Option<Presence>>>,
    pub spectators: Arc<Mutex<Vec<i32>>>,
    pub last_ping: Arc<Mutex<DateTime<Utc>>>,
}

impl Presence {
    pub fn new(
        user: User,
        data: Option<ClientData>,
        country: Option<u8>,
        lat: Option<f32>,
        lon: Option<f32>,
    ) -> Self {
        //Generating token from uuid
        let token = Uuid::new_v4().to_string();
        Self {
            token,
            user,
            packet_queue: Arc::new(Mutex::new(Vec::new())),
            client_data: data.unwrap_or_default(),
            status: Arc::new(RwLock::new(ClientChangeAction::default())),
            country: country.unwrap_or(0),
            cached_stats: Arc::new(RwLock::new(UserDbStats::default())),
            lat: lat.unwrap_or(0.0),
            lon: lon.unwrap_or(0.0),
            previous_message: Arc::new(Mutex::new("".into())),
            previous_message_repeated: Arc::new(Mutex::new(0)),
            silenced_until: Arc::new(RwLock::new(0)),
            spectating: Arc::new(Mutex::new(None)),
            spectators: Arc::new(Mutex::new(Vec::new())),
            last_ping: Arc::new(Mutex::new(Utc::now())),
        }
    }

    pub async fn start_spectating(
        &self,
        other: &Presence,
        manager: &ChannelManager,
        bancho_manager: &BanchoManager,
    ) {
        if other.user.id == self.user.id {
            return;
        }

        let mut spectating = self.spectating.lock().await;

        if spectating.is_some() {
            self.stop_spectating(bancho_manager).await;
        }
        *spectating = Some(other.clone());

        other.spectator_joined(self, bancho_manager).await;

        if let None = manager
            .get_channel_by_name(format!("#spec_{}", other.user.id).as_str())
            .await
        {
            manager
                .create_private_channel(-other.user.id, format!("#spec_{}", other.user.id))
                .await;

            manager
                .join_channel_with_friendly_name(
                    format!("#spec_{}", other.user.id).as_str(),
                    other,
                    "#spectator".to_string(),
                )
                .await;
            manager
                .join_channel_with_friendly_name(
                    format!("#spec_{}", other.user.id).as_str(),
                    self,
                    "#spectator".to_string(),
                )
                .await;
        }
    }

    pub async fn spectator_joined(&self, other: &Presence, manager: &BanchoManager) {
        let mut spectators = self.spectators.lock().await;
        spectators.push(other.user.id);

        self.enqueue(SpectatorJoined::new(other.user.id).into_packet_data())
            .await;

        for spectator_id in &*spectators {
            if let Some(spectator) = manager.get_presence_by_user_id(*spectator_id).await {
                spectator
                    .enqueue(FellowSpectatorJoined::new(spectator.user.id).into_packet_data())
                    .await;
            }
        }
    }

    pub async fn stop_spectating(&self, manager: &BanchoManager) {
        let mut spectating = self.spectating.lock().await;
        if let Some(spectator) = spectating.as_ref() {
            spectator.spectator_left(self, manager).await;
            *spectating = None;
        }
    }

    pub async fn spectator_left(&self, other: &Presence, manager: &BanchoManager) {
        let mut spectators = self.spectators.lock().await;
        spectators.retain(|spectator| *spectator != other.user.id);

        self.enqueue(SpectatorLeft::new(other.user.id).into_packet_data())
            .await;
        for spectator_id in &*spectators {
            if let Some(spectator) = manager.get_presence_by_user_id(*spectator_id).await {
                spectator
                    .enqueue(FellowSpectatorLeft::new(spectator.user.id).into_packet_data())
                    .await;
            }
        }
    }

    pub async fn spectate_frames(&self, frames: Vec<u8>, manager: &BanchoManager) {
        let spectators = self.spectators.lock().await;

        for spectator_id in spectators.iter() {
            if let Some(spectator) = manager.get_presence_by_user_id(*spectator_id).await {
                spectator
                    .enqueue(SpectatorFrames::new(frames.clone()).into_packet_data())
                    .await;
            }
        }
    }

    pub async fn refresh_stats(&self, connection: &Pool<Postgres>, redis: &Client) {
        let status = self.status.read().await;

        let stats = get_user_stats(connection, &self.user.id, &self.get_active_mode().await).await;
        if let Err(err) = stats {
            error!("Failed to fetch user stats: {:#?}", err);
            self.enqueue(
                Notification::new(
                    "Failed to fetch user stats, please, contact server administrator.".into(),
                )
                .into_packet_data(),
            )
            .await;
            return;
        }
        let stats = stats.unwrap();

        let mut stats_cached = self.cached_stats.write().await;

        //Placing stats in stats_cached
        *stats_cached = stats.clone();

        self.enqueue(
            UserStats::new(
                self.user.id,
                status.online_status,
                status.description.clone().into(),
                status.beatmap_md5.clone().into(),
                status.mods,
                status.mode,
                status.beatmap_id,
                stats.ranked_score,
                (stats.accuracy * 100.0) as f32,
                stats.playcount,
                stats.total_score,
                get_rank(redis, &self.user, &self.get_active_mode().await)
                    .await
                    .unwrap_or(0),
                stats.performance as i16,
            )
            .into_packet_data(),
        )
        .await;
    }

    pub async fn update_status(&self, status: ClientChangeAction) {
        let mut status_lock = self.status.write().await;
        *status_lock = status;
    }

    pub async fn get_active_mode(&self) -> OsuMode {
        let status = self.status.read().await;

        if status.mods & 128 > 0 {
            return OsuMode::Relax;
        }

        OsuMode::from_id(status.mode)
    }

    pub async fn stats_packet(&self, redis: &Client) -> Vec<u8> {
        let status = self.status.read().await;
        let stats = self.cached_stats.read().await;

        return UserStats::new(
            self.user.id,
            status.online_status,
            status.description.clone().into(),
            status.beatmap_md5.clone().into(),
            status.mods,
            status.mode,
            status.beatmap_id,
            stats.ranked_score,
            (stats.accuracy * 100.0) as f32,
            stats.playcount,
            stats.total_score,
            get_rank(&redis, &self.user, &self.get_active_mode().await)
                .await
                .unwrap_or(0),
            stats.performance as i16,
        )
        .into_packet_data();
    }

    pub async fn trigger_moderation(&self, payload: &BanchoMessage, _ctx: &Context) -> bool {
        let mut prev_message = self.previous_message.lock().await;
        let mut messages_repeated = self.previous_message_repeated.lock().await;

        if prev_message.to_string() != payload.content {
            *prev_message = payload.content.clone();
            *messages_repeated = 0;

            return false;
        }

        *messages_repeated += 1;
        if *messages_repeated >= 5 {
            //Silencing user for 10 minutes
            let mut silenced_until = self.silenced_until.write().await;
            *silenced_until = (chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp();

            self.enqueue(
                SilenceEnd::new(chrono::Duration::minutes(10).num_seconds() as i32)
                    .into_packet_data(),
            )
            .await;

            // insert_user_punishment(&ctx.pool, "MEDIUM".to_string(), 2, self.user.id, "TIMEOUT".to_string(), true, NaiveDateTime::from_timestamp_opt(*silenced_until).unwrap_or(NaiveDateTime::UNIX_EPOCH), format!("Auto: Spam in {}", payload.target)).await;
            return true;
        }

        return false;
    }
}

impl Presence {
    pub async fn enqueue(&self, packet: Vec<u8>) {
        let mut queue = self.packet_queue.lock().await;
        queue.extend(packet);

        drop(queue);
    }

    pub async fn dequeue(&self) -> Vec<u8> {
        let mut queue = self.packet_queue.lock().await;

        let mut packet = Vec::new();
        std::mem::swap(&mut packet, &mut queue);

        drop(queue);
        packet
    }
}
