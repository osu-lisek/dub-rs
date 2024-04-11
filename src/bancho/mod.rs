use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};

use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, Level};

use crate::{context::Context, health_check};

use self::{
    bancho_manager::BanchoManager,
    bot::mio::MioBot,
    channel_manager::ChannelManager,
    handler::api::{get_server_stats, get_user_status, refresh_user, send_notification},
};

pub mod bancho_manager;
pub mod bot;
pub mod channel_manager;
pub mod client;
pub mod handler;
pub mod presence;

pub async fn serve_bancho(ctx: Context) {
    let ctx = Arc::new(ctx);

    let manager = Arc::new(BanchoManager::init(ctx.clone()));
    if !manager.init_bot().await {
        error!("Failed to start bancho");
        return;
    }

    let channel_manager = Arc::new(ChannelManager::new(manager.clone(), ctx.clone()));
    if let Err(error) = channel_manager.load_channels_from_db().await {
        error!("Failed to load channels: {:#?}", error);
        error!("Failed to start bancho");
        return;
    }

    let bot_presence = manager.get_bot_presence().await;

    if let None = bot_presence {
        error!("Failed to start bancho: No bot presence found");
        return;
    }

    let bot_presence = bot_presence.unwrap();

    let mut bot = MioBot::new(
        ctx.clone(),
        bot_presence,
        manager.clone(),
        channel_manager.clone(),
    );

    bot.register_commands();

    let layer_ctx = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::DEBUG)
                        .include_headers(true)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        )
        .layer(Extension(ctx))
        .layer(Extension(manager))
        .layer(Extension(channel_manager))
        .layer(Extension(Arc::new(Mutex::new(bot))));

    let router = Router::new()
        .merge(crate::bancho::handler::serve())
        .route("/api/v2/bancho/notification", post(send_notification))
        .route("/api/v2/bancho/user/:id", get(get_user_status))
        .route("/api/v2/bancho/stats", get(get_server_stats))
        .route("/api/v2/bancho/update", post(refresh_user))
        .route("/health", get(health_check))
        .layer(layer_ctx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
