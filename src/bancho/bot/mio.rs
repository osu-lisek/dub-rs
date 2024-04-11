use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use bancho_packets::BanchoMessage;

use regex::Regex;

use tokio::sync::Mutex;
use tracing::debug;

use crate::{
    bancho::{
        bancho_manager::BanchoManager,
        bot::commands::{acc, map, with},
        channel_manager::ChannelManager,
        presence::Presence,
    },
    context::Context,
    utils::{
        beatmap_utils::{get_beatmap_by_id, Beatmap as DbBeatmap},
        general_utils::to_fixed,
        performance_utils::calculate_performance_with_accuracy_list,
        score_utils::format_mods,
    },
};

use super::commands::roll;

pub type BotCommand = fn(&mut MioBot, &Presence, Vec<String>) -> Option<String>;

pub struct MioBot {
    presence: Arc<Presence>,
    _bancho_manager: Arc<BanchoManager>,
    channel_manager: Arc<ChannelManager>,
    pub ctx: Arc<Context>,
    pub commands: HashMap<String, BotCommand>,
    pub user_beatmaps: Mutex<HashMap<i32, DbBeatmap>>,
    beatmap_regex: Regex,
}

impl MioBot {
    pub fn new(
        ctx: Arc<Context>,
        presence: Arc<Presence>,
        bancho_manager: Arc<BanchoManager>,
        channel_manager: Arc<ChannelManager>,
    ) -> Self {
        Self {
            presence: Arc::clone(&presence),
            ctx,
            commands: HashMap::new(),
            _bancho_manager: Arc::clone(&bancho_manager),
            channel_manager: Arc::clone(&channel_manager),
            beatmap_regex: Regex::new(r#"\/(\d+)\s"#).expect("Failed to parse regex"),
            user_beatmaps: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_command(&mut self, name: &str, command: BotCommand) {
        self.commands.insert(name.to_string(), command);
    }

    pub fn register_commands(&mut self) {
        self.register_command("roll", roll);
    }

    pub async fn handle_command_dms(&mut self, author: &Presence, message: &BanchoMessage) {
        let started_at = SystemTime::now();
        let captures = self.beatmap_regex.captures(&message.content);

        if let Some(captures) = captures {
            //Last one - is beatmap id, what we need
            let beatmap_id = captures.get(1).unwrap().as_str();

            let fetched_beatmap =
                get_beatmap_by_id(&self.ctx.pool, beatmap_id.parse::<i64>().unwrap_or(0)).await;
            if let Err(_) = fetched_beatmap {
                self.channel_manager
                    .handle_private_message(
                        &self.presence,
                        &(BanchoMessage {
                            sender: self.presence.user.username.clone(),
                            content: "Failed to fetch beatmap".to_string(),
                            target: author.user.username.clone(),
                            sender_id: self.presence.user.id,
                        }),
                    )
                    .await;
                return;
            }

            let fetched_beatmap = fetched_beatmap.unwrap();

            if let None = fetched_beatmap {
                self.channel_manager
                    .handle_private_message(
                        &self.presence,
                        &(BanchoMessage {
                            sender: self.presence.user.username.clone(),
                            content: "Failed to fetch beatmap".to_string(),
                            target: author.user.username.clone(),
                            sender_id: self.presence.user.id,
                        }),
                    )
                    .await;
                return;
            }

            let fetched_beatmap = fetched_beatmap.unwrap();

            let mut mods = 0;
            let status = author.status.read().await;

            if status.online_status != 0 {
                mods = status.mods as u32;
            }

            let accuracy_list = vec![100.0, 99.0, 98.0];
            let result = calculate_performance_with_accuracy_list(
                &self.ctx.pool,
                beatmap_id.parse::<i64>().unwrap_or(0),
                accuracy_list,
                Some(mods),
            )
            .await;

            match result {
                Ok(results) => {
                    let mut performance_response: Vec<String> = vec![];
                    for result in results {
                        performance_response.push(format!(
                            "{}%: {}pp",
                            result.accuracy,
                            to_fixed(result.performance, 2)
                        ));
                    }

                    let response = format!(
                        "[osu://b/{} {} - {} [{}]] + {} - {} - took {}ms",
                        beatmap_id,
                        fetched_beatmap.artist,
                        fetched_beatmap.title,
                        fetched_beatmap.version,
                        format_mods(mods),
                        performance_response.join(" | "),
                        started_at
                            .elapsed()
                            .unwrap_or(Duration::from_micros(0))
                            .as_millis()
                    );

                    self.channel_manager
                        .handle_private_message(
                            &self.presence,
                            &(BanchoMessage {
                                sender: self.presence.user.username.clone(),
                                content: response,
                                target: author.user.username.clone(),
                                sender_id: self.presence.user.id,
                            }),
                        )
                        .await;
                }
                Err(_) => {
                    self.channel_manager
                        .handle_private_message(
                            &self.presence,
                            &(BanchoMessage {
                                sender: self.presence.user.username.clone(),
                                content: "Failed to calculate performance for this beatmap"
                                    .to_string(),
                                target: author.user.username.clone(),
                                sender_id: self.presence.user.id,
                            }),
                        )
                        .await;
                }
            }

            self.user_beatmaps
                .lock()
                .await
                .insert(author.user.id, fetched_beatmap);
        }

        self.handle_command(author, message).await;
    }

    pub async fn handle_response(&self, target: String, response: String, _author: &Presence) {
        if target.starts_with("#") {
            self.channel_manager
                .handle_public_message(
                    &self.presence,
                    &(BanchoMessage {
                        sender: self.presence.user.username.clone(),
                        content: response,
                        target: target,
                        sender_id: self.presence.user.id,
                    }),
                )
                .await;
        } else {
            self.channel_manager
                .handle_private_message(
                    &self.presence,
                    &(BanchoMessage {
                        sender: self.presence.user.username.clone(),
                        content: response,
                        target: target,
                        sender_id: self.presence.user.id,
                    }),
                )
                .await;
        }
    }

    pub async fn handle_command(&mut self, author: &Presence, message: &BanchoMessage) {
        let content = message.content.clone();

        debug!("Started handling command: {}", content);
        if !content.starts_with("!") {
            return;
        }

        let command_name = content.split_whitespace().next().unwrap().replace("!", "");
        let args: Vec<String> = content
            .split_whitespace()
            .skip(1)
            .map(|s| s.to_string())
            .collect();

        //TODO: Reword this shit.
        match command_name.as_str() {
            "map" => {
                let response = map(self, author, args.clone()).await;
                if let Some(response) = response {
                    self.handle_response(
                        message
                            .target
                            .to_string()
                            .eq(&self.presence.user.username.clone())
                            .then(|| author.user.username.to_string())
                            .unwrap_or(message.target.to_string()),
                        response,
                        author,
                    )
                    .await;
                }
            }
            "with" => {
                let response = with(self, author, args.clone()).await;
                if let Some(response) = response {
                    self.handle_response(
                        message
                            .target
                            .to_string()
                            .eq(&self.presence.user.username.clone())
                            .then(|| author.user.username.to_string())
                            .unwrap_or(message.target.to_string()),
                        response,
                        author,
                    )
                    .await;
                }
            }
            "acc" => {
                let response = acc(self, author, args.clone()).await;
                if let Some(response) = response {
                    self.handle_response(
                        message
                            .target
                            .to_string()
                            .eq(&self.presence.user.username.clone())
                            .then(|| author.user.username.to_string())
                            .unwrap_or(message.target.to_string()),
                        response,
                        author,
                    )
                    .await;
                }
            }
            _ => {}
        }
        let command = self.commands.get(&command_name);

        debug!(
            "Command: {:?}, args: {:?}, name: {:?}",
            command, args, command_name
        );
        if let Some(command) = command {
            let response = command(self, author, args);
            debug!("Response: {:?}", response);
            if let Some(response) = response {
                if message.target.starts_with("#") {
                    self.channel_manager
                        .handle_public_message(
                            &self.presence,
                            &(BanchoMessage {
                                sender: self.presence.user.username.clone(),
                                content: response,
                                target: message.target.clone(),
                                sender_id: self.presence.user.id,
                            }),
                        )
                        .await;
                } else {
                    self.channel_manager
                        .handle_private_message(
                            &self.presence,
                            &(BanchoMessage {
                                sender: self.presence.user.username.clone(),
                                content: response,
                                target: author.user.username.clone(),
                                sender_id: self.presence.user.id,
                            }),
                        )
                        .await;
                }
            }
        }
    }
}
