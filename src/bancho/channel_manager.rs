use std::{collections::HashMap, sync::Arc};

use bancho_packets::{
    server::{ChannelInfo, ChannelJoin, ChannelKick, SendMessage},
    BanchoMessage, BanchoPacket,
};
use sqlx::prelude::FromRow;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

use crate::{
    context::Context,
    utils::{
        channel_utils::fetch_channels, score_utils::OsuServerError, user_utils::is_restricted,
    },
};

use super::{bancho_manager::BanchoManager, presence::Presence};

#[derive(FromRow, Debug)]
pub struct Message {
    pub id: i64,
    pub channel_id: i64,
    pub author_id: i32,
    pub content: String,
    pub deleted: String,
    pub created_at: sqlx::types::chrono::NaiveDateTime,
}

pub struct Channel {
    pub id: i32,
    pub channel_type: String,
    pub name: String,
    pub description: String,
    pub users: Mutex<Vec<i32>>,
    pub messages: RwLock<Vec<Message>>,
}

pub struct ChannelManager {
    //ID:channel
    pub channels: RwLock<HashMap<i32, Arc<Channel>>>,
    bancho_manager: Arc<BanchoManager>,
    context: Arc<Context>,
}

impl ChannelManager {
    pub fn new(bancho_manager: Arc<BanchoManager>, context: Arc<Context>) -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            bancho_manager,
            context,
        }
    }

    pub async fn dispose_presence(&self, presence: &Presence) {
        for channel in self.channels.read().await.values() {
            if channel.users.lock().await.contains(&presence.user.id) {
                self.part(presence, channel.name.clone()).await;
            }
        }
    }
    pub async fn load_channels_from_db(&self) -> Result<(), OsuServerError> {
        let channels = fetch_channels(&self.context.pool).await;

        match channels {
            Err(error) => Err(error),
            Ok(channels) => {
                let mut locked_channels = self.channels.write().await;
                info!("Loading {} channels", channels.len());

                for channel in channels {
                    locked_channels.insert(
                        channel.id,
                        Arc::new(Channel {
                            id: channel.id,
                            channel_type: channel.channel_type,
                            name: channel.name,
                            description: channel.description,
                            users: Mutex::new(Vec::new()),
                            messages: RwLock::new(Vec::new()),
                        }),
                    );
                }

                Ok(())
            }
        }
    }

    pub async fn get_channel_by_name(&self, channel_name: &str) -> Option<Arc<Channel>> {
        let locked_channels = self.channels.read().await;

        for (_id, channel) in locked_channels.iter() {
            if channel.name == channel_name {
                return Some(Arc::clone(channel));
            }
        }

        None
    }

    pub async fn join_channel(&self, channel_name: &str, presence: &Presence) {
        if let Some(channel) = self.get_channel_by_name(channel_name).await {
            let mut users = channel.users.lock().await;

            if users.contains(&presence.user.id) {
                presence
                .enqueue(ChannelJoin::new((format!("{}", channel_name)).into()).into_packet_data())
                .await; // Sometimes osu doesn't know about channel that it joined

                warn!("{} tried to join channel #{}, but presence is already in this channel.", presence.user.username, channel_name);
                return;
            }
            users.push(presence.user.id);
            //Sending to presence packet that he joined channel
            presence
                .enqueue(ChannelJoin::new((format!("{}", channel_name)).into()).into_packet_data())
                .await;
            info!(
                "User {} joined channel {}",
                presence.user.username, channel_name
            );

            debug!("Users of channel #{} updated: {:#?}", channel_name, users);
        }
    }

    pub async fn join_channel_with_friendly_name(
        &self,
        channel_name: &str,
        presence: &Presence,
        friendly_name: String,
    ) {
        if let Some(channel) = self.get_channel_by_name(channel_name).await {
            channel.users.lock().await.push(presence.user.id);
            //Sending to presence packet that he joined channel
            presence
                .enqueue(ChannelJoin::new((format!("{}", friendly_name)).into()).into_packet_data())
                .await;
            info!(
                "User {} joined channel {}",
                presence.user.username, channel_name
            );
        }
    }

    pub async fn send_channel_info(&self, presence: &Presence) {
        let channels = self.channels.read().await;

        for channel in channels.values() {
            if channel.channel_type == "public" {
                presence
                    .enqueue(
                        ChannelInfo::new(
                            channel.name.clone().into(),
                            channel.description.clone().into(),
                            channel.users.lock().await.len() as i16,
                        )
                        .into_packet_data(),
                    )
                    .await;
            }
        }
    }

    pub async fn part(&self, presence: &Presence, channel_name: String) {
        if let Some(channel) = self.get_channel_by_name(channel_name.as_str()).await {

            let mut users = channel.users.lock().await;
            let index = users
                .iter()
                .position(|&x| x == presence.user.id)
                .unwrap();
            users.remove(index);
            //Sending to presence packet that he joined channel
            presence
                .enqueue(ChannelKick::new((format!("{}", channel_name)).into()).into_packet_data())
                .await;

            info!(
                "User {} parted from channel {}",
                presence.user.username, channel_name
            );
        }
    }

    pub async fn create_private_channel(&self, id: i32, channel_name: String) {
        if let None = self.get_channel_by_name(&channel_name).await {
            let mut channels = self.channels.write().await;
            channels.insert(
                id,
                Arc::new(Channel {
                    channel_type: "private_temp".to_string(),
                    id: id,
                    description: "Temp channel".to_string(),
                    name: channel_name.to_string(),
                    messages: RwLock::new(Vec::new()),
                    users: Mutex::new(Vec::new()),
                }),
            );
        }
    }

    pub async fn handle_public_message(&self, presence: &Presence, payload: &BanchoMessage) {
        if is_restricted(&presence.user).await {
            info!(
                "User {} tried to write, while user is restricted",
                presence.user.username
            );
            return;
        }

        let mut channel_name = payload.target.clone();

        if channel_name == "#spectator" {
            if let Some(spectating_presence) = presence.spectating.lock().await.to_owned() {
                channel_name = format!("#spec_{}", spectating_presence.user.id);
            }

            let spectators = presence.spectators.lock().await;

            if spectators.len() > 0 {
                channel_name = format!("#spec_{}", presence.user.id);
            }
        }

        let channel = self.get_channel_by_name(&channel_name).await;

        if let None = channel {
            info!(
                "User {} tried to write in channel {} that does not exist",
                presence.user.username, payload.target
            );
            return;
        }

        let channel = channel.unwrap();

        if !channel.users.lock().await.contains(&presence.user.id) && presence.user.id != 1 {
            info!(
                "User {} tried to write in channel {} that he is not in",
                presence.user.username, payload.target
            );
            return;
        }

        //Sending it to channel
        for &user in channel.users.lock().await.iter() {
            if user != presence.user.id {
                let other_presence = self.bancho_manager.get_presence_by_user_id(user).await;

                if let None = other_presence {
                    warn!(
                        "User {} in channel {} is offline/does not exists.",
                        user,
                        payload.clone().target
                    );
                    continue;
                }

                let payload = payload.clone();

                if let Some(other_presence) = other_presence {
                    other_presence
                        .enqueue(
                            SendMessage::new(
                                presence.user.username.to_string().into(),
                                payload.content.into(),
                                payload.target.into(),
                                presence.user.id,
                            )
                            .into_packet_data(),
                        )
                        .await;
                }
            }
        }

        info!(
            "{} -> {}: {}",
            presence.user.username, payload.target, payload.content
        );
    }
    pub async fn handle_private_message(&self, presence: &Presence, payload: &BanchoMessage) {
        if is_restricted(&presence.user).await {
            info!(
                "User {} tried to write, while user is restricted",
                presence.user.username
            );
            return;
        }

        let target = self
            .bancho_manager
            .get_presence_by_username(payload.target.to_string())
            .await;

        if let None = target {
            info!(
                "User {} tried to write in user {} that does not exist or offline",
                presence.user.username, payload.target
            );
            return;
        }

        let target = target.unwrap();

        if is_restricted(&target.user).await && payload.sender_id != 1 {
            info!(
                "User {} tried to write in user {} that is restricted",
                presence.user.username,
                payload.clone().target
            );
            return;
        }

        target
            .enqueue(
                SendMessage::new(
                    payload.clone().sender.into(),
                    payload.clone().content.into(),
                    payload.clone().target.into(),
                    payload.clone().sender_id,
                )
                .into_packet_data(),
            )
            .await;

        info!(
            "{} -> {}: {}",
            presence.user.username, payload.target, payload.content
        );
    }
}
