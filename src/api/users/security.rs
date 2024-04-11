use std::sync::Arc;

use axum::{extract::Path, http::StatusCode, Extension, Json};
use chrono::NaiveDateTime;
use serde::Serialize;
use tracing::{error, info};

use crate::{
    api::FailableResponse,
    context::Context,
    utils::user_utils::{find_user_by_id_or_username, get_user_recent_vilations, is_restricted},
};

#[derive(Debug, Serialize)]
pub struct AccountViolation {
    pub violation_type: String,
    pub received_at: NaiveDateTime,
    pub reason: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct AccountStanding {
    pub good: bool,
    pub vilations: Vec<AccountViolation>,
}

pub async fn get_user_account_standing(
    Extension(ctx): Extension<Arc<Context>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<FailableResponse<AccountStanding>>) {
    let user = find_user_by_id_or_username(&ctx.pool, id).await;

    if let Err(error) = user {
        info!("Error getting user: {:?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Internal server error.")),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    if let None = user {
        return (
            StatusCode::NOT_FOUND,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("Not found")),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    if is_restricted(&user).await {
        return (
            StatusCode::FORBIDDEN,
            Json(FailableResponse {
                ok: false,
                message: Some(String::from("This profile is unaccessable.")),
                data: None,
            }),
        );
    }

    let punishments = get_user_recent_vilations(&ctx.pool, &user).await;

    if let Err(error) = punishments {
        error!("{:#?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                data: None,
                message: Some("Internal server error".to_string()),
            }),
        );
    }

    let punishments = punishments.unwrap();

    if punishments.is_empty() {
        return (
            StatusCode::OK,
            Json(FailableResponse {
                ok: true,
                data: Some(AccountStanding {
                    good: true,
                    vilations: Vec::new(),
                }),
                message: None,
            }),
        );
    }

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            data: Some(AccountStanding {
                good: false,
                vilations: punishments
                    .iter()
                    .map(|punishment| AccountViolation {
                        expires_at: punishment.expires_at.unwrap_or(NaiveDateTime::UNIX_EPOCH),
                        reason: punishment.note.clone(),
                        received_at: punishment.date,
                        violation_type: punishment.punishment_type.clone(),
                    })
                    .collect(),
            }),
            message: None,
        }),
    )
}
