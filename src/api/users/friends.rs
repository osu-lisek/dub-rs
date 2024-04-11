use std::sync::Arc;

use axum::{extract::Path, http::StatusCode, Extension, Json};
use axum_macros::debug_handler;
use tracing::error;

use crate::{
    api::FailableResponse,
    context::Context,
    db::user::User,
    utils::user_utils::{
        add_friend, find_user_by_id_or_username, get_user_followers, get_user_relationships,
        is_donator, is_user_manager, remove_friend,
    },
};

use super::Friend;

#[debug_handler]
pub async fn get_friends(
    Extension(ctx): Extension<Arc<Context>>,
    Extension(user): Extension<Option<User>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<FailableResponse<Vec<Friend>>>) {
    if let Err(e) = id.parse::<i32>() {
        return (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some(e.to_string()),
                data: None,
            }),
        );
    }

    if user.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(FailableResponse {
                ok: false,
                message: Some("Not logged in".to_string()),
                data: None,
            }),
        );
    }

    let user = user.unwrap();
    let id = id.parse::<i32>().unwrap();

    if user.id != id && !is_user_manager(&user) {
        return (
            StatusCode::FORBIDDEN,
            Json(FailableResponse {
                ok: false,
                message: Some("Not authorized".to_string()),
                data: None,
            }),
        );
    }

    let friends = get_user_relationships(&ctx.pool, &id).await;

    if friends.is_err() {
        error!("Failed to fetch friends: {:#?}", friends.unwrap_err());
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some("Internal server error".to_string()),
                data: None,
            }),
        );
    }
    let friends = friends.unwrap();

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(
                friends
                    .iter()
                    .map(|x| Friend {
                        id: x.friend_id,
                        is_mutual: x.is_mutual,
                        is_online: false,
                        username: x.username.clone(),
                        banner: x.banner.clone(),
                        country: x.country.clone(),
                    })
                    .collect(),
            ),
        }),
    )
}

pub async fn change_friend_status(
    Extension(ctx): Extension<Arc<Context>>,
    Extension(user): Extension<Option<User>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<FailableResponse<bool>>) {
    if let None = user {
        return (
            StatusCode::UNAUTHORIZED,
            Json(FailableResponse {
                ok: false,
                message: Some("Not logged in".to_string()),
                data: None,
            }),
        );
    }

    let user = user.unwrap();

    let requested_user = find_user_by_id_or_username(&ctx.pool, id)
        .await
        .unwrap_or_default();

    if let None = requested_user {
        return (
            StatusCode::NOT_FOUND,
            Json(FailableResponse {
                ok: false,
                message: Some("User not found".to_string()),
                data: None,
            }),
        );
    }

    let requested_user = requested_user.unwrap();

    if requested_user.id == user.id {
        return (
            StatusCode::BAD_REQUEST,
            Json(FailableResponse {
                ok: false,
                message: Some("Cannot add yourself as friend".to_string()),
                data: None,
            }),
        );
    }
    let is_friend = get_user_relationships(&ctx.pool, &user.id)
        .await
        .unwrap()
        .iter()
        .any(|x| x.friend_id == requested_user.id);

    if is_friend {
        remove_friend(&ctx.pool, &user.id, &requested_user.id).await;
    } else {
        add_friend(&ctx.pool, &user.id, &requested_user.id).await;
    }

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(true),
        }),
    )
}

pub async fn get_followers(
    Extension(ctx): Extension<Arc<Context>>,
    Extension(authorized_user): Extension<Option<User>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<FailableResponse<Vec<Friend>>>) {
    let user = find_user_by_id_or_username(&ctx.pool, id).await.unwrap();

    if let None = user {
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

    if let None = authorized_user {
        return (
            StatusCode::UNAUTHORIZED,
            Json(FailableResponse {
                ok: false,
                message: Some("Not logged in".to_string()),
                data: None,
            }),
        );
    }

    let authorized_user = authorized_user.unwrap();
    if authorized_user.id != user.id && !is_user_manager(&user) {
        return (
            StatusCode::FORBIDDEN,
            Json(FailableResponse {
                ok: false,
                message: Some("Not authorized".to_string()),
                data: None,
            }),
        );
    }

    if !is_donator(&authorized_user) {
        return (
            StatusCode::FORBIDDEN,
            Json(FailableResponse {
                ok: false,
                message: Some("You need to be a donator, to use this feature".to_string()),
                data: None,
            }),
        );
    }

    let followers = get_user_followers(&ctx.pool, &user.id).await;

    if let Err(error) = followers {
        error!("Failed to fetch followers: {:#?}", error);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FailableResponse {
                ok: false,
                message: Some("Internal server error".to_string()),
                data: None,
            }),
        );
    }

    let followers = followers.unwrap();

    (
        StatusCode::OK,
        Json(FailableResponse {
            ok: true,
            message: None,
            data: Some(
                followers
                    .iter()
                    .map(|x| Friend {
                        id: x.friend_id,
                        is_mutual: x.is_mutual,
                        is_online: false,
                        username: x.username.clone(),
                        banner: x.banner.clone(),
                        country: x.country.clone(),
                    })
                    .collect(),
            ),
        }),
    )
}
