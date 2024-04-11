use clap::{arg, clap_derive::Parser};

#[derive(Parser, Debug, Clone)]
pub struct RunConfiguration {
    #[arg(long, env)]
    pub app_component: String,
    #[arg(long, env)]
    pub database_dsn: String,
    #[arg(long, env)]
    pub redis_url: String,
    #[arg(long, env)]
    pub alert_discord_webhook: Option<String>,
    #[arg(long, env)]
    pub server_url: String,
    #[arg(long, env)]
    pub token_hmac_secret: String,
    #[arg(long, env)]
    pub port: Option<i16>,
    #[arg(long, env)]
    pub listing_key: Option<String>,
}
