use std::sync::Arc;

use axum::{extract::Path, http::StatusCode, Extension, Json};
use bancho_packets::{
    server::{Notification, UserLogout},
    BanchoMessage, BanchoPacket, BanchoPacketWrite,
};

use serde::{Deserialize, Serialize};

use crate::{
    api::FailableResponse,
    bancho::{bancho_manager::BanchoManager, channel_manager::ChannelManager},
    context::Context,
    utils::{
        beatmap_utils::{get_beatmap_by_id, PublicBeatmap},
        user_utils::{find_user_by_id_or_username, is_restricted},
    },
};

#[derive(Debug, Deserialize)]
pub struct ServerMessageStruct {
    pub key: String,
    pub user_id: i32,
    pub args: Vec<String>,
    pub method: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageBody {
    pub message: String,
    pub message_type: String,
    pub target: String,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct APIStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beatmap: Option<PublicBeatmap>,
}
#[derive(Debug, Serialize)]
pub struct OnlineUser {
    pub username: String,
    pub status: APIStatus,
}

#[derive(Debug, Serialize)]
pub struct ServerStats {
    pub users: i32,
    pub online: i32,
}

fn status_to_string(status: u8) -> String {
    match status {
        0 => "idle",
        1 => "afk",
        2 => "playing",
        3 => "editing",
        4 => "modding",
        5 => "multiplayer",
        6 => "watching",
        7 => "unknown",
        8 => "testing",
        9 => "submitting",
        10 => "paused",
        11 => "lobby",
        12 => "multiplaying",
        13 => "osu!direct",
        _ => "unkown",
    }
    .to_string()
}

pub async fn get_server_stats(
    Extension(bancho_manager): Extension<Arc<BanchoManager>>,
    Extension(ctx): Extension<Arc<Context>>,
) -> (StatusCode, Json<FailableResponse<ServerStats>>) {
    let online = bancho_manager.get_online().await;
    let users = sqlx::query!(r#"SELECT COUNT(*) as count FROM "User""#)
        .fetch_one(&*ctx.pool)
        .await
        .unwrap();

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(ServerStats {
                users: users.count.unwrap_or(0) as i32,
                online,
            }),
        }),
    )
}

pub async fn get_user_status(
    Extension(bancho_manager): Extension<Arc<BanchoManager>>,
    Extension(ctx): Extension<Arc<Context>>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<FailableResponse<OnlineUser>>) {
    let presence = bancho_manager.get_presence_by_user_id(id).await;

    if presence.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(FailableResponse {
                ok: false,
                message: Some("User doesn't seems to be online right now".to_string()),
                data: None,
            }),
        );
    }

    let presence = presence.unwrap();

    let presence_status = presence.status.read().await;
    let mut beatmap = None;

    if presence_status.beatmap_id != 0 {
        beatmap = get_beatmap_by_id(&ctx.pool, presence_status.beatmap_id as i64)
            .await
            .unwrap_or(None)
    }

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(OnlineUser {
                username: presence.user.username.clone(),
                status: APIStatus {
                    status: status_to_string(presence_status.online_status),
                    beatmap: beatmap.map(|x| x.to_public()),
                },
            }),
        }),
    )
}

pub async fn send_notification(
    Extension(bancho_manager): Extension<Arc<BanchoManager>>,
    Extension(channel_manager): Extension<Arc<ChannelManager>>,
    Extension(ctx): Extension<Arc<Context>>,
    Json(payload): Json<MessageBody>,
) -> (StatusCode, Json<FailableResponse<bool>>) {
    if payload.key != ctx.config.token_hmac_secret {
        return (
            StatusCode::UNAUTHORIZED,
            Json(FailableResponse {
                ok: false,
                message: Some("Unauthorized".to_string()),
                data: None,
            }),
        );
    }

    let bot = bancho_manager.get_bot_presence().await.unwrap();

    match payload.message_type.as_str() {
        "pm" => {
            let user = find_user_by_id_or_username(&ctx.pool, payload.target.to_string()).await;

            if user.is_err() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("Internal server error".to_string()),
                        data: None,
                    }),
                );
            }

            let user = user.unwrap();

            if user.is_none() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("User not found".to_string()),
                        data: None,
                    }),
                );
            }

            let user = user.unwrap();

            let presence = bancho_manager.get_presence_by_user_id(user.id).await;

            if presence.is_none() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("User not online.".to_string()),
                        data: None,
                    }),
                );
            }

            let presence = presence.unwrap();

            channel_manager
                .handle_private_message(
                    &bot,
                    &BanchoMessage {
                        content: payload.message,
                        sender: bot.user.username.to_string(),
                        sender_id: bot.user.id,
                        target: presence.user.username.to_string(),
                    },
                )
                .await;

            (
                StatusCode::OK,
                Json(FailableResponse {
                    ok: true,
                    message: None,
                    data: Some(true),
                }),
            )
        }

        "chat" => {
            if !payload.target.to_string().starts_with('#') {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("Invalid target".to_string()),
                        data: None,
                    }),
                );
            }

            channel_manager
                .handle_public_message(
                    &bot,
                    &BanchoMessage {
                        content: payload.message,
                        sender: bot.user.username.to_string(),
                        sender_id: bot.user.id,
                        target: payload.target.to_string(),
                    },
                )
                .await;

            (
                StatusCode::OK,
                Json(FailableResponse {
                    ok: true,
                    message: None,
                    data: Some(true),
                }),
            )
        }
        "notification" => {
            let user = find_user_by_id_or_username(&ctx.pool, payload.target.to_string()).await;

            if user.is_err() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("Internal server error".to_string()),
                        data: None,
                    }),
                );
            }

            let user = user.unwrap();

            if user.is_none() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("User not found".to_string()),
                        data: None,
                    }),
                );
            }

            let user = user.unwrap();

            let presence = bancho_manager.get_presence_by_user_id(user.id).await;

            if presence.is_none() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("User not online.".to_string()),
                        data: None,
                    }),
                );
            }

            let presence = presence.unwrap();

            presence
                .enqueue(Notification::new(payload.message.into()).into_packet_data())
                .await;

            (
                StatusCode::OK,
                Json(FailableResponse {
                    ok: true,
                    message: None,
                    data: Some(true),
                }),
            )
        }

        _ => (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some("Invalid message type".to_string()),
                data: None,
            }),
        ),
    }
}

pub async fn refresh_user(
    Extension(bancho_manager): Extension<Arc<BanchoManager>>,
    Extension(channel_manager): Extension<Arc<ChannelManager>>,
    Extension(ctx): Extension<Arc<Context>>,
    Json(payload): Json<ServerMessageStruct>,
) -> (StatusCode, Json<FailableResponse<String>>) {
    if payload.key != ctx.config.token_hmac_secret {
        return (
            StatusCode::UNAUTHORIZED,
            Json(FailableResponse {
                ok: false,
                message: Some("Unauthorized".to_string()),
                data: None,
            }),
        );
    }

    match payload.method.as_str() {
        "user:refresh" => {
            let presence = bancho_manager
                .get_presence_by_user_id(payload.user_id)
                .await;

            if presence.is_none() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("Presence not found".to_string()),
                        data: None,
                    }),
                );
            }

            let presence = presence.unwrap();

            presence.refresh_stats(&ctx.pool, &ctx.redis).await;
            bancho_manager
                .broadcast_packet(presence.stats_packet(&ctx.redis).await.into_packet())
                .await;

            (
                StatusCode::OK,
                Json(FailableResponse {
                    ok: true,
                    message: None,
                    data: Some("Sent refresh packet.".to_string()),
                }),
            )
        }
        "user:restricted" => {
            let presence = bancho_manager
                .get_presence_by_user_id(payload.user_id)
                .await;

            if presence.is_none() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(FailableResponse {
                        ok: false,
                        message: Some("Presence not found".to_string()),
                        data: None,
                    }),
                );
            }

            let presence = presence.unwrap();

            if is_restricted(&presence.user).await {
                channel_manager
                    .handle_private_message(
                        &bancho_manager
                            .get_bot_presence()
                            .await
                            .expect("Failed to get bot"),
                        &BanchoMessage {
                            sender: "Mio".to_string(),
                            content: "We have lifted your punishment, relog to take effect."
                                .to_string(),
                            target: presence.user.username.to_string(),
                            sender_id: 1,
                        },
                    )
                    .await;
                return (
                    StatusCode::OK,
                    Json(FailableResponse {
                        ok: true,
                        message: None,
                        data: Some("Sent restricted packet.".to_string()),
                    }),
                );
            }
            presence.refresh_stats(&ctx.pool, &ctx.redis).await;
            bancho_manager
                .broadcast_packet(presence.stats_packet(&ctx.redis).await.into_packet())
                .await;

            channel_manager.handle_private_message(&bancho_manager.get_bot_presence().await.expect("Failed to get bot"), &BanchoMessage { sender: "Mio".to_string(), content: "Your account currently in restricted state, more details you can get from \"Account standing\" page on the website".to_string(), target: presence.user.username.to_string(), sender_id: 1 }).await;

            //Removing from online panel for other users
            bancho_manager
                .broadcast_packet(UserLogout::new(presence.user.id).into_packet_data())
                .await;

            presence.spectators.lock().await.clear();
            *presence.spectating.lock().await = None;

            (
                StatusCode::OK,
                Json(FailableResponse {
                    ok: true,
                    message: None,
                    data: Some("Sent restricted packet.".to_string()),
                }),
            )
        }
        _ => (
            StatusCode::OK,
            Json(FailableResponse {
                ok: false,
                message: Some("Unknown method".to_string()),
                data: None,
            }),
        ),
    }
}
