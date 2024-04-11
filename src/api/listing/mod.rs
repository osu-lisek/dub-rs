use axum::{routing::get, Router};
use serde::Deserialize;

use self::routes::handle_vote;

pub mod routes;

#[derive(Deserialize, Debug)]
pub struct GetBackQuery {
    pub username: String,
    pub key: String,
}

pub fn router() -> Router {
    Router::new().route("/api/v2/listing/getback", get(handle_vote))
}
