use std::env;
use std::str::FromStr;
use std::sync::Arc;

use clap::Parser;
use config::RunConfiguration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::ConnectOptions;
use tracing::{error, info, Level};

use crate::api::serve_api;
use crate::bancho::serve_bancho;
use crate::clean::run_cleanup;
use crate::context::Context;
use crate::recalculate::recalculate_terminal;
use crate::web::serve as serve_web;

mod api;
mod bancho;
mod clean;
mod config;
mod context;
mod db;
mod managers;
mod recalculate;
mod utils;
mod web;

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::FmtSubscriber::builder()
        .with_thread_names(true)
        .with_target(false)
        .with_max_level(
            Level::from_str(env::var("LOG_LEVEL").unwrap_or("INFO".to_string()).as_str())
                .unwrap_or(Level::DEBUG),
        )
        .compact()
        .init();
    let run_configuration = RunConfiguration::parse();

    dotenvy::dotenv().unwrap();

    let connection_options = PgConnectOptions::from_str(&run_configuration.database_dsn)
        .unwrap()
        .log_statements(tracing::log::LevelFilter::Debug)
        .log_slow_statements(
            tracing::log::LevelFilter::Warn,
            std::time::Duration::from_secs(1),
        );
    let pool = PgPoolOptions::new().connect_with(connection_options).await;

    if let Err(error) = pool {
        error!("Failed to connect to db: {:#?}", error);
        return;
    }

    info!(name: "db", "Connected to database!");

    let pool = pool.unwrap();

    let redis = redis::Client::open(run_configuration.clone().redis_url);
    if let Err(error) = redis {
        error!("Error while connecting to redis: {}", error);
        return;
    }

    let redis = redis.unwrap();
    let context = Context {
        pool: Arc::new(pool),
        config: Arc::new(run_configuration.clone()),
        redis: Arc::new(redis),
    };

    match run_configuration.app_component.as_str() {
        "web" => serve_web(context).await,
        "bancho" => serve_bancho(context).await,
        "api" => serve_api(context).await,
        "recalculation-terminal" => recalculate_terminal(context).await,
        "cleanup" => run_cleanup(context).await,
        _ => {
            error!("Unknown component.");
        }
    };
}
