use std::sync::Arc;

use axum::{extract::DefaultBodyLimit, middleware, routing::get, Extension, Router};
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::{context::Context, health_check};

use self::auth::middleware::auth;

pub mod auth;
pub mod beatmaps;
pub mod listing;
pub mod rankings;
pub mod users;

#[derive(Debug, Serialize)]
pub struct FailableResponse<T> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

pub async fn serve_api(ctx: Context) {
    let ctx = Arc::new(ctx);

    let layer_ctx = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .include_headers(true)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        )
        .layer(Extension(ctx))
        .layer(middleware::from_fn(auth));

    let router = Router::new()
        .merge(crate::api::users::router())
        .merge(crate::api::auth::router())
        .merge(crate::api::listing::router())
        .nest("/api/v2/beatmaps", crate::api::beatmaps::router())
        .nest("/api/v2/rankings", crate::api::rankings::router())
        .route("/health", get(health_check))
        .layer(layer_ctx)
        .layer(DefaultBodyLimit::max(1024 * 8));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
