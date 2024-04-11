use std::{ops::BitAnd, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, Query},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Extension, Router,
};

use serde::Deserialize;
use string_builder::Builder;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{debug, error};

use crate::{
    context::Context,
    utils::{
        beatmap_utils::{get_beatmap_by_hash, get_online_beatmap_by_checksum},
        http_utils::{OsuMode, ScoreRequestQuery},
        score_utils::{get_beatmap_leaderboard, get_user_best},
        user_utils::{get_user_id, validate_auth},
    },
    web::scores::submission::ScoreStatus,
};

pub mod submission;

#[derive(Deserialize)]
pub struct ReplayRequestQuery {
    pub c: i32,
    pub m: i32,
    pub u: String,
    pub h: String,
}

async fn get_score_replay(
    Extension(ctx): Extension<Arc<Context>>,
    Query(query): Query<ReplayRequestQuery>,
) -> Vec<u8> {
    if !validate_auth(&ctx.redis, &ctx.pool, query.u.clone(), query.h.to_string()).await {
        return "error: pass".to_string().into();
    }

    let score_id = query.c;

    let opened_file = File::open(format!("data/replays/{}.osr_frames", score_id)).await;
    if let Err(_) = opened_file {
        return "error: no".to_string().into();
    }

    let mut opened_file = opened_file.unwrap();

    let mut buf = vec![0; opened_file.metadata().await.unwrap().len() as usize];
    let bytes = opened_file.read(&mut buf).await;
    if let Err(_) = bytes {
        return "error: fail".into();
    }

    buf
}

async fn update_beatmap(Path(file): Path<String>) -> Response {
    //Ensuring maybe there is update for this beatmap

    let response = reqwest::get(format!("https://osu.ppy.sh/web/maps/{}", file)).await;

    match response {
        Ok(response) => {
            let bytes = response.text().await.unwrap();
            return Response::builder()
                .header(
                    "content-disposition",
                    format!(r#"attachment; filename="{}""#, file),
                )
                .body(Body::from(bytes))
                .unwrap();
        }
        Err(_) => {
            return Response::builder()
                .status(404)
                .body(Body::from(""))
                .unwrap();
        }
    }
}

async fn get_scores(
    Extension(ctx): Extension<Arc<Context>>,
    Query(query): Query<ScoreRequestQuery>,
) -> Response {
    let ScoreRequestQuery {
        username,
        password,
        leaderboard_version,
        beatmap_hash,
        mode,
        mods,
        leaderboard_type,
        set_id: _,
        filename,
    } = query;

    if !validate_auth(&ctx.redis, &ctx.pool, username.clone(), password).await {
        return Response::builder().body(Body::from("error: pass")).unwrap();
    }

    if leaderboard_version != 4 {
        return Response::builder().body(Body::from("error: pass")).unwrap();
    }

    let user_id = get_user_id(&ctx.redis, &ctx.pool, username.clone()).await;

    if user_id.is_none() {
        return Response::builder().body(Body::from("error: pass")).unwrap();
    }

    let user_id = user_id.unwrap();

    let playmode = OsuMode::from_id(
        128.bitand(mods.clone())
            .eq(&128)
            .then(|| 4)
            .unwrap_or(mode.clone() as u8),
    );

    let mut beatmap = get_beatmap_by_hash(&ctx.pool, beatmap_hash.clone()).await;

    //Ensuring maybe there is update for this beatmap

    if beatmap.clone().is_err() || beatmap.clone().unwrap().is_none() {
        let online_beatmap = get_online_beatmap_by_checksum(beatmap_hash.clone()).await;

        if let Err(error) = online_beatmap {
            error!("Beatmap error: {:#?}", error);

            let response = reqwest::get(format!("https://osu.ppy.sh/web/maps/{}", filename)).await;

            match response {
                Ok(response) => {
                    let bytes = response.bytes().await.unwrap();

                    if bytes.len() == 0 {
                        return Response::builder().body(Body::from("-1|false")).unwrap();
                    }
                    //Getting hash
                    let hash = format!("{:x}", md5::compute(bytes));

                    if hash != beatmap_hash {
                        return Response::builder().body(Body::from("1|false")).unwrap();
                    }
                }
                Err(_) => {}
            }

            return Response::builder().body(Body::from("-1|false")).unwrap();
        }

        let online_beatmap = online_beatmap.unwrap();

        online_beatmap.clone().insert_in_db(&ctx.pool).await;

        beatmap = Ok(Some(online_beatmap.clone()));
    }

    let beatmap = beatmap.clone().unwrap().unwrap();

    let status = ScoreStatus::find_suitable_best_status_for_beatmap(beatmap.clone().status.into());
    let user_best = get_user_best(
        &ctx.pool,
        beatmap_hash.clone(),
        user_id,
        playmode.clone(),
        mods,
        leaderboard_type,
        Some(status.to_db()),
    )
    .await;

    if let Err(error) = user_best {
        error!("User best error: {:#?}", error);
        return Response::builder().body(Body::from("error: no")).unwrap();
    }

    let user_best = user_best.unwrap();

    debug!("{:#?}", user_best);

    let mut response = Builder::default();
    let leaderboard = get_beatmap_leaderboard(
        &ctx.pool,
        beatmap_hash.clone(),
        playmode.clone(),
        mods,
        Some(
            ScoreStatus::find_suitable_best_status_for_beatmap(beatmap.clone().status.into())
                .to_db(),
        ),
    )
    .await;

    if let Err(error) = leaderboard {
        error!("Leaderboard error: {:#?}", error);
        return Response::builder().body(Body::from("error: no")).unwrap();
    }

    let mut leaderboard = leaderboard.unwrap();

    response.append(format!(
        "{}|false|{}|{}|0\n0\n",
        beatmap.status, beatmap.beatmap_id, beatmap.parent_id
    ));
    response.append(format!("{} - {}\n", beatmap.artist, beatmap.title));

    if beatmap.status <= 0 {
        leaderboard.clear();
    }

    response.append(format!("{}\n", leaderboard.len()));

    match user_best {
        Some(best) => response.append(best.to_osu(best.score.rank) + "\n"),
        None => {
            response.append("\n");
        }
    }

    for score in leaderboard {
        response.append(score.to_osu(score.score.rank) + "\n");
    }
    Response::builder()
        .body(Body::from(response.string().unwrap()))
        .unwrap()
}

pub fn serve() -> Router {
    Router::new()
        .route("/web/maps/:file", get(update_beatmap))
        .route("/web/osu-osz2-getscores.php", get(get_scores))
        .route(
            "/web/osu-submit-modular-selector.php",
            post(submission::submit_score),
        )
        .route("/web/osu-getreplay.php", get(get_score_replay))
}
