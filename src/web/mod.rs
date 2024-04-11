pub mod other;
pub mod scores;

use std::sync::Arc;

use axum::{routing::get, Extension, Router};
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::{context::Context, health_check};

pub async fn serve(ctx: Context) {
    let port = ctx.config.port.unwrap_or(3000);
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
        .layer(Extension(ctx));

    let router = Router::new()
        .merge(crate::web::scores::serve())
        .merge(crate::web::other::serve())
        .route("/health", get(health_check))
        .layer(layer_ctx);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}
