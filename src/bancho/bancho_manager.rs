use std::{collections::HashMap, sync::Arc};

use bancho_packets::{
    server::{
        BanchoPrivileges, ChannelInfoEnd, Notification, ProtocolVersion, SilenceEnd, UserLogout,
        UserPresence,
    },
    BanchoMessage, BanchoPacket,
};
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::{
    context::Context,
    utils::{
        http_utils::OsuMode,
        user_utils::{
            get_rank, get_silenced_until, get_user_by_id, is_pending_verification, is_restricted,
            to_safe,
        },
    },
};

use super::{channel_manager::ChannelManager, client::ClientData, presence::Presence};

pub struct BanchoManager {
    presences: RwLock<HashMap<String, Arc<Presence>>>,
    context: Arc<Context>,
}

impl BanchoManager {
    pub fn init(context: Arc<Context>) -> Self {
        Self {
            presences: RwLock::new(HashMap::new()),
            context,
        }
    }

    pub async fn get_bot_presence(&self) -> Option<Arc<Presence>> {
        let mut bot_token = "".to_string();

        for (token, presence) in self.presences.read().await.iter() {
            if presence.user.id == 1 {
                bot_token = token.clone();
                break;
            }
        }

        if bot_token.is_empty() {
            return None;
        }

        self.get_presence_by_token(bot_token).await
    }

    pub async fn init_bot(&self) -> bool {
        let bot = get_user_by_id(&self.context.pool, 1).await;

        if let Err(error) = bot {
            error!("Failed to init bot: {:#?}", error);
            return false;
        }

        let bot = bot.unwrap();

        match bot {
            Some(user) => {
                let presence = Presence::new(user, None, Some(0), None, None);
                self.presences
                    .write()
                    .await
                    .insert(presence.clone().token, Arc::new(presence.clone()));

                info!(
                    "Bot logged in as: {}({})",
                    presence.clone().user.username,
                    presence.clone().user.clone().id
                );
                true
            }
            None => {
                //TODO: Create new user if not
                false
            }
        }
    }

    pub async fn init_user(
        &self,
        user_id: i32,
        data: ClientData,
        country: u8,
        lat: f32,
        lon: f32,
        channel_manager: &ChannelManager,
    ) -> Option<String> {
        let user = get_user_by_id(&self.context.pool, user_id).await;

        if let Err(error) = user {
            error!("Failed to init user: {:#?}", error);
            return None;
        }

        let user = user.unwrap();

        match user {
            Some(user) => {
                let presence = Presence::new(
                    user.clone(),
                    Some(data.clone()),
                    Some(country),
                    Some(lat),
                    Some(lon),
                );
                info!(
                    "User logged in as: {}({})",
                    presence.clone().user.username,
                    presence.clone().user.clone().id
                );
                //Sending general packets
                presence
                    .enqueue(BanchoPrivileges::new(4).into_packet_data())
                    .await;
                presence
                    .enqueue(ProtocolVersion::new(19).into_packet_data())
                    .await;
                presence
                    .enqueue(SilenceEnd::new(0).into_packet_data())
                    .await;
                presence
                    .enqueue(ChannelInfoEnd::new().into_packet_data())
                    .await;
                channel_manager.send_channel_info(&presence).await;
                channel_manager.join_channel("#osu", &presence).await;
                channel_manager.join_channel("#announce", &presence).await;

                let silenced_until = get_silenced_until(&self.context.pool, user.id).await;
                if silenced_until > 0 {
                    let seconds_until = (silenced_until - Utc::now().timestamp()) as i32;
                    presence
                        .enqueue(SilenceEnd::new(seconds_until).into_packet_data())
                        .await;
                    *presence.silenced_until.write().await = silenced_until;
                }

                let user_rank = get_rank(&self.context.redis, &user, &OsuMode::Osu)
                    .await
                    .unwrap_or(0);
                presence
                    .enqueue(
                        UserPresence::new(
                            user.clone().id,
                            user.clone().username.into(),
                            (data.clone().time_offset + 24) as u8,
                            presence.country,
                            0,
                            lon,
                            lat,
                            user_rank,
                        )
                        .into_packet_data(),
                    )
                    .await;
                presence
                    .refresh_stats(&self.context.pool, &self.context.redis)
                    .await;

                //Sending everyone about new user
                for another_presence in self.presences.write().await.values() {
                    presence
                        .enqueue(
                            UserPresence::new(
                                another_presence.user.clone().id,
                                another_presence.user.clone().username.into(),
                                (another_presence.client_data.time_offset + 24) as u8,
                                another_presence.country,
                                0,
                                lon,
                                lat,
                                user_rank,
                            )
                            .into_packet_data(),
                        )
                        .await;

                    if is_pending_verification(&user) {
                        let _ = sqlx::query!(
                            r#"UPDATE "User" SET permissions = 0, flags = 0 WHERE "id" = $1"#,
                            user.id
                        )
                        .execute(&*self.context.pool)
                        .await;
                        presence
                            .enqueue(
                                Notification::new("You has been verified.".into())
                                    .into_packet_data(),
                            )
                            .await;
                    }

                    if !is_restricted(&user).await {
                        another_presence
                            .enqueue(
                                UserPresence::new(
                                    user.clone().id,
                                    user.clone().username.into(),
                                    (data.clone().time_offset + 24) as u8,
                                    presence.country,
                                    0,
                                    0.0,
                                    0.0,
                                    user_rank,
                                )
                                .into_packet_data(),
                            )
                            .await;
                    }
                }

                let token = presence.clone().token;
                self.presences
                    .write()
                    .await
                    .insert(token.clone(), Arc::new(presence.clone()));

                if is_restricted(&user).await {
                    channel_manager.handle_private_message(&self.get_bot_presence().await.expect("Failed to get bot."), &BanchoMessage {
                        sender: "Mio".into(),
                        content: "Your account currently in restricted state, more details you can get from \"Account standing\" page on the website.".into(),
                        target: presence.user.username.to_string(),
                        sender_id: 1
                    }).await;
                }

                Some(token)
            }
            None => {
                error!("Failed to init user: User not found");
                None
            }
        }
    }

    pub async fn get_online(&self) -> i32 {
        self.presences.read().await.len() as i32
    }

    pub async fn get_presence_by_token(&self, token: String) -> Option<Arc<Presence>> {
        self.presences.read().await.get(&token).cloned()
    }

    pub async fn get_presence_by_user_id(&self, user_id: i32) -> Option<Arc<Presence>> {
        self.presences
            .read()
            .await
            .iter()
            .find(|(_token, presence)| presence.user.id == user_id)
            .map(|presence| Arc::clone(presence.1))
    }

    pub async fn get_presence_by_username(&self, username: String) -> Option<Arc<Presence>> {
        if let Some((_token, presence)) = self
            .presences
            .read()
            .await
            .iter()
            .find(|(_token, presence)| presence.user.username_safe == to_safe(&username))
        {
            Some(Arc::clone(presence))
        } else {
            None
        }
    }

    pub async fn broadcast_packet(&self, packet: Vec<u8>) {
        let presences = self.presences.read().await;
        debug!("Found {} presences", presences.len());
        for presence in presences.values() {
            debug!("Broadcasting packet for: {}", presence.user.username);
            if presence.user.id == 1 {
                continue;
            }
            presence.enqueue(packet.clone()).await;
        }

        drop(presences);
        debug!("Finished broadcast");
    }

    pub async fn dispose_presence(&self, token: String, channel_manager: &ChannelManager) {
        let presence = self.get_presence_by_token(token.clone()).await;

        if presence.is_none() {
            return;
        }

        let presence = presence.unwrap();

        channel_manager.dispose_presence(&presence).await;

        //Deleting it from presences map
        let mut presences = self.presences.write().await;

        presences.remove(&token);

        info!("User {} logged out.", presence.user.username);
        //Broadcasting logout packet
        self.broadcast_packet(UserLogout::new(presence.user.id).into_packet_data())
            .await;
    }

    pub async fn get_presences(&self) -> Vec<Arc<Presence>> {
        self.presences.read().await.values().cloned().collect()
    }
}
