pub mod api;

use std::{sync::Arc, time::SystemTime};

use axum::{
    body::{to_bytes, Body},
    http::Response,
    routing::{get, post},
    Router,
};
use bancho_packets::{
    server::{BanchoRestart, LoginReply, Notification, UserSilenced},
    BanchoMessage, BanchoPacket, BanchoPacketRead, ClientChangeAction, PacketBuilder, PacketReader,
    PayloadReader,
};
use chrono::Utc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use webhook::client::WebhookClient;

use crate::{
    bancho::{bot::mio::MioBot, channel_manager::ChannelManager, client::ClientData},
    context::Context,
    utils::{
        ip_utils::{get_ip_info, Country},
        user_utils::{
            find_hwids, get_user_by_id, get_user_id, update_user_country, update_user_hwid,
            validate_auth,
        },
    },
};

use super::bancho_manager::BanchoManager;

async fn bancho_get(req: axum::http::Request<Body>) -> Response<Body> {
    let manager = req.extensions().get::<Arc<BanchoManager>>();

    if let None = manager {
        error!("No bancho manager found");
        return Response::builder()
            .body(Body::from("not found manager"))
            .unwrap();
    }

    let manager = manager.unwrap();

    let online = manager.get_online().await;

    Response::builder()
        .body(Body::from(format!("Total online: {}", online)))
        .unwrap()
}

fn login_failed(host: String, message: String) -> Response<Body> {
    Response::builder()
        .header("cho-token", "nicht")
        .body(Body::from(
            PacketBuilder::default()
                .add(Notification::new(format!("{}: {}", host, message).into()))
                .add(LoginReply::failed_invalid_credentials())
                .build(),
        ))
        .unwrap()
}

fn login_server_error() -> Response<Body> {
    Response::builder()
        .header("cho-token", "nicht")
        .body(Body::from(
            PacketBuilder::default()
                .add(LoginReply::failed_server_error())
                .build(),
        ))
        .unwrap()
}

fn reconnect(delay: i32, host: String, message: String) -> Response<Body> {
    Response::builder()
        .header("cho-token", "nicht")
        .body(Body::from(
            PacketBuilder::default()
                .add(Notification::new(format!("{}: {}", host, message).into()))
                .add(BanchoRestart::new(delay))
                .build(),
        ))
        .unwrap()
}

async fn bancho_post(req: axum::http::Request<Body>) -> Response<Body> {
    // let started_at = SystemTime::now();
    let (parts, body) = req.into_parts();
    let manager = parts.extensions.get::<Arc<BanchoManager>>();
    let channel_manager = parts.extensions.get::<Arc<ChannelManager>>();
    let bot = parts.extensions.get::<Arc<Mutex<MioBot>>>().unwrap();
    let ctx = parts.extensions.get::<Arc<Context>>().unwrap();
    let host = parts.headers.get("host").unwrap().to_str().unwrap();

    if let None = manager {
        error!("No bancho manager found");
        return Response::builder()
            .body(Body::from("not found manager"))
            .unwrap();
    }

    if let None = channel_manager {
        error!("No channel manager found");
        return Response::builder()
            .body(Body::from("not found channel manager"))
            .unwrap();
    }

    let channel_manager = channel_manager.unwrap();
    let manager = manager.unwrap();

    let is_login_request = parts.headers.get("osu-token").is_some();

    if !is_login_request {
        //TODO: LOGIN
        let login_body = to_bytes(body, usize::MAX).await;

        if let Err(e) = login_body {
            error!("Failed to read body: {}", e);
            return Response::builder()
                .header("cho-token", "nicht")
                .body(Body::from(
                    PacketBuilder::default()
                        .add(LoginReply::new(bancho_packets::LoginResult::Failed(
                            bancho_packets::LoginFailedReason::ServerError,
                        )))
                        .build(),
                ))
                .unwrap();
        }

        let login_body = login_body.unwrap();
        let login_body = login_body.to_vec();
        //Converting it to string
        let login_body = String::from_utf8(login_body).unwrap();
        let mut lines = login_body.lines();

        let username = lines.next();
        let password = lines.next();

        let username = username.unwrap();
        let password = password.unwrap();

        let is_auth_ok = validate_auth(&ctx.redis, &ctx.pool, username, password).await;

        if !is_auth_ok {
            return login_failed(host.to_string(), "Invalid credentials".to_string());
        }

        let user_id = get_user_id(&ctx.redis, &ctx.pool, username).await;
        if let None = user_id {
            return login_failed(host.to_string(), "Invalid credentials".to_string());
        }

        let user_id = user_id.unwrap();

        let user = get_user_by_id(&ctx.pool, user_id).await.unwrap().unwrap();

        let client_data = lines.next();
        let client_data = ClientData::from(client_data.unwrap().to_string());

        let users_with_current_hwid = find_hwids(&ctx.pool, &client_data.hwid).await;

        if let Err(e) = users_with_current_hwid {
            error!("Failed to find user: {:#?}", e);
            return login_server_error();
        }

        let users_with_current_hwid = users_with_current_hwid.unwrap();

        if users_with_current_hwid.len() > 1 && ctx.config.alert_discord_webhook.clone().is_some() {
            let formatted_report = users_with_current_hwid
                .iter()
                .map(|entry| {
                    format!(
                        "{} -> https://{}/users/{}",
                        entry.user.username, ctx.config.server_url, entry.user.id
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");
            //Reporting it to alerts
            let client =
                WebhookClient::new(ctx.config.alert_discord_webhook.clone().unwrap().as_str());
            let is_report_ok = client
                .send(|message| {
                    message
                        .content("Multiple users with the same HWID found!")
                        .embed(|embed| {
                            embed
                                .author(
                                    user.username.as_str(),
                                    Some(format!(
                                        "https://{}/users/{}",
                                        ctx.config.server_url, user.id
                                    )),
                                    Some(format!(
                                        "https://a.{}/{}",
                                        ctx.config.server_url, user.id
                                    )),
                                )
                                .description(format!("{}", formatted_report.as_str()).as_str())
                                .footer(format!("Host: {}", host).as_str(), None)
                        })
                })
                .await;
            if let Err(e) = is_report_ok {
                error!("Report of hwid was unsuccessful: {}", e);
                return login_server_error();
            }
        }

        update_user_hwid(&ctx.pool, &user, &client_data.hwid).await;
        let ip_header = parts.headers.get("Cf-Connecting-Ip");
        let mut ip: Option<String> = None;

        if ip_header.is_some() {
            ip = Some(ip_header.unwrap().to_str().unwrap().to_string());
        }

        if let Some(ref completed_ip) = ip {
            if completed_ip == "127.0.0.1" {
                ip = None;
            }
        }
        let country = get_ip_info(ip).await;

        let mut code = 0 as u8;
        let mut lat = 0.0;
        let mut lon = 0.0;
        if let Some(country) = country {
            if user.country == "XX" {
                update_user_country(&ctx.pool, user.id, country.clone().code).await;
            }

            lat = country.lat;
            lon = country.lon;
            code = Country::from_code(&country.code).unwrap_or(Country::XX.to_byte());
        }

        let presence = manager
            .init_user(user.id, client_data, code, lat, lon, channel_manager)
            .await;

        if let None = presence {
            return login_server_error();
        }

        let token = presence.unwrap();

        return Response::builder()
            .header("cho-token", token)
            .body(Body::from(
                PacketBuilder::default()
                    // .add(Notification::new(
                    //     format!(
                    //         "Login took: {}ms",
                    //         started_at.elapsed().unwrap().as_millis()
                    //     )
                    //     .into(),
                    // ))
                    .add(LoginReply::success(user.id))
                    .build(),
            ))
            .unwrap();
    }

    //@TODO: Remake it
    for presence in manager.get_presences().await {
        if presence.user.id != 1
            && presence.last_ping.lock().await.timestamp() < (Utc::now().timestamp() - 60)
        {
            manager
                .dispose_presence(presence.token.clone(), &channel_manager)
                .await;
        }
    }

    let token = parts
        .headers
        .get("osu-token")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let presence = manager.get_presence_by_token(token).await;

    if let None = presence {
        return Response::builder()
            .header("cho-token", "nicht")
            .body(Body::from(
                PacketBuilder::default().add(BanchoRestart::new(1)).build(),
            ))
            .unwrap();
    }

    let presence = presence.unwrap();

    *presence.last_ping.lock().await = Utc::now();

    //Handling all packets
    let body = to_bytes(body, usize::MAX).await;

    if let Err(e) = body {
        error!("{}", e);
        return reconnect(
            1,
            host.to_string(),
            "Internal server error occured, you will be reconnected.".to_string(),
        );
    }
    let body = body.unwrap();

    let mut reader = PacketReader::new(&body);

    while let Some(packet) = reader.next() {
        let id = packet.id;
        let mut payload_reader = PayloadReader::new(packet.payload.unwrap_or_default());

        match id {
            bancho_packets::PacketId::OSU_USER_REQUEST_STATUS_UPDATE => {
                presence.refresh_stats(&ctx.pool, &ctx.redis).await;
            }
            bancho_packets::PacketId::OSU_USER_CHANGE_ACTION => {
                let data = ClientChangeAction::read(&mut payload_reader);

                if let Some(status) = data.clone() {
                    presence.update_status(status.clone()).await;
                    presence.refresh_stats(&ctx.pool, &ctx.redis).await;

                    //Broadcasting to everyone about new status
                    manager
                        .broadcast_packet(presence.stats_packet(&ctx.redis).await)
                        .await;
                }
            }
            bancho_packets::PacketId::OSU_USER_LOGOUT => {
                manager
                    .dispose_presence(presence.token.to_owned(), &channel_manager)
                    .await;
            }
            bancho_packets::PacketId::OSU_PING => {}
            bancho_packets::PacketId::OSU_USER_CHANNEL_PART => {
                let name = payload_reader.read::<String>().unwrap();
                info!("Trying to part from: {}", name);
                channel_manager.part(&presence, name).await;
            }
            bancho_packets::PacketId::OSU_USER_CHANNEL_JOIN => {
                let name = payload_reader.read::<String>().unwrap();
                channel_manager.join_channel(name.as_str(), &presence).await;
            }
            bancho_packets::PacketId::OSU_SEND_PUBLIC_MESSAGE => {
                let payload = BanchoMessage::read(&mut payload_reader);

                if payload.is_none() {
                    continue;
                }

                let payload = Arc::new(payload.unwrap());
                if presence.trigger_moderation(&payload, ctx).await {
                    manager
                        .broadcast_packet(UserSilenced::new(presence.user.id).into_packet_data())
                        .await;
                }

                channel_manager
                    .handle_public_message(&presence, &payload)
                    .await;

                bot.lock().await.handle_command(&presence, &payload).await;
            }
            bancho_packets::PacketId::OSU_SEND_PRIVATE_MESSAGE => {
                let payload = BanchoMessage::read(&mut payload_reader);

                if let None = payload {
                    continue;
                }

                let payload = payload.unwrap();

                channel_manager
                    .handle_private_message(&presence, &payload)
                    .await;

                if payload.target == "Mio" {
                    bot.lock()
                        .await
                        .handle_command_dms(&presence, &payload)
                        .await;
                }
            }
            bancho_packets::PacketId::OSU_USER_STATS_REQUEST => {
                let users = payload_reader.read::<Vec<i32>>();

                if let None = users {
                    continue;
                }

                let users = users.unwrap();

                for user in users {
                    if user == presence.user.id {
                        continue;
                    }

                    let user_presence = manager.get_presence_by_user_id(user).await;

                    if let None = user_presence {
                        continue;
                    }

                    let user_presence = user_presence.unwrap();

                    presence
                        .enqueue(user_presence.stats_packet(&ctx.redis).await)
                        .await;
                }
            }
            bancho_packets::PacketId::OSU_SPECTATE_START => {
                let payload = payload_reader.read::<i32>();
                if let Some(id) = payload {
                    let other = manager.get_presence_by_user_id(id).await;

                    if let Some(other) = other {
                        presence
                            .start_spectating(&other, &channel_manager, manager)
                            .await;
                    }
                }
            }
            bancho_packets::PacketId::OSU_SPECTATE_STOP => {
                presence.stop_spectating(manager).await;
            }
            bancho_packets::PacketId::OSU_SPECTATE_FRAMES => {
                let data = payload_reader.payload();
                presence.spectate_frames(data.to_vec(), manager).await;
            }
            id => {
                warn!("Unhandled packet: {}", id);
            }
        }
    }

    let packets = presence.dequeue().await;
    debug!("Sending back buffer with: {} bytes", packets.len());
    Response::builder().body(Body::from(packets)).unwrap()
}

pub fn serve() -> Router {
    Router::new()
        .route("/", post(bancho_post))
        .route("/", get(bancho_get))
}
