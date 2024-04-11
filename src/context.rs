use std::sync::Arc;

use redis::Client;
use sqlx::{Pool, Postgres};

use crate::config::RunConfiguration;

pub struct Context {
    pub pool: Arc<Pool<Postgres>>,
    pub config: Arc<RunConfiguration>,
    pub redis: Arc<Client>,
}
