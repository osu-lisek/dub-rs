use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query},
    response::Response,
    Extension,
};
use reqwest::StatusCode;
use serde::Deserialize;
use url_builder::URLBuilder;

use crate::{
    context::Context,
    utils::{beatmap_utils::OnlineBeatmap, http_utils::OsuMode, user_utils::validate_auth},
    web::scores::submission::BeatmapStatus,
};

#[derive(Deserialize, Clone)]
pub struct BeatmapsQuery {
    /// username
    u: String,
    /// hashed password
    h: String,
    /// ranked status
    r: u8,
    /// query
    q: String,
    /// mode
    m: i8,
    /// page_num
    p: i32,
}

#[derive(Deserialize, Clone)]
pub struct BeatmapSetQuery {
    /// username
    u: String,
    /// hashed password
    h: String,
    /// beatmap set id
    s: Option<i32>,
    /// beatmap id
    b: Option<i32>,
    /// beatmap checksum
    c: Option<String>,
}

pub struct SearchRequestParameters {
    amount: i16,
    offset: i32,
    query: Option<String>,
    mode: Option<OsuMode>,
    status: Option<BeatmapStatus>,
}

#[derive(Deserialize, Clone)]
pub struct DirectBeatmapSet {
    id: i64,
    title: String,
    title_unicode: String,
    artist: String,
    artist_unicode: String,
    creator: String,
    source: String,
    tags: String,
    ranked: i8,
    submitted_date: String,
    approved_date: Option<String>,
    last_updated: String,
    beatmaps: Vec<OnlineBeatmap>,
}

fn normalize_direct_name(input: String) -> String {
    return input.replace("@", "").replace("|", "-");
}

fn ranked_status_to_string(status: u8) -> &'static str {
    match status {
        0 => "ranked",
        2 => "pending",
        3 => "qualified",
        // 4 - All
        5 => "graveyard", // Graveyard
        7 => "ranked",    // Ranked (Played)
        8 => "loved",
        _ => "",
    }
}

pub async fn search_beatmaps(
    Extension(ctx): Extension<Arc<Context>>,
    Query(query): Query<BeatmapsQuery>,
) -> Response {
    if !validate_auth(&ctx.redis, &ctx.pool, query.u, query.h).await {
        return Response::builder().body(Body::from("error: pass")).unwrap();
    }

    let params = SearchRequestParameters {
        amount: 100,
        offset: query.p * 100,
        mode: None,
        query: None,
        status: None,
    };

    let mut ub = URLBuilder::new();

    ub.set_protocol("https")
        .set_host("mirror.lisek.cc")
        .add_route("api")
        .add_route("v1")
        .add_route("search")
        .add_param("limit", params.amount.to_string().as_str())
        .add_param("offset", params.offset.to_string().as_str());

    if !vec!["Newest", "Top+Rated", "Most+Played"].contains(&query.q.as_str()) {
        ub.add_param("query", query.q.as_str());
    }

    if query.m >= 0 {
        ub.add_param("modes[0]", &OsuMode::from_id(query.m as u8).to_lazer_name());
    }

    if query.r != 4 {
        ub.add_param("statuses[0]", ranked_status_to_string(query.r));
    }

    let url = &ub.build();
    let request = reqwest::get(url).await.unwrap();

    if request.status() != StatusCode::OK {
        return Response::builder().body(Body::from("-1|Unable to fetch beatmaps.")).unwrap();
    }

    let result: serde_json::Value = request.json().await.unwrap();
    let json: Vec<DirectBeatmapSet> = serde_json::from_value(result).unwrap();

    let page_size = if json.iter().len() > 99 {
        101
    } else {
        json.iter().len()
    };

    let mut body = format!("{page_size}\n");

    for beatmap in json {
        let mut diffs: Vec<String> = vec![];

        for diff in beatmap.beatmaps {
            diffs.push(format!(
                "[{:.2}*] {}@{}",
                diff.difficulty_rating,
                normalize_direct_name(diff.version),
                diff.mode_int
            ))
        }

        body += format!(
            "{}.osz|{}|{}|{}|{}|0.0|{}|{}|0|0|0|0|0|{}\n",
            beatmap.id,
            normalize_direct_name(beatmap.artist),
            normalize_direct_name(beatmap.title),
            beatmap.creator,
            beatmap.ranked,
            beatmap.last_updated,
            beatmap.id,
            diffs.join(", ")
        )
        .as_str();
    }

    return Response::builder().body(Body::from(body)).unwrap();
}

pub async fn search_beatmap_set(
    Extension(ctx): Extension<Arc<Context>>,
    Query(query): Query<BeatmapSetQuery>,
) -> Response {
    if !validate_auth(&ctx.redis, &ctx.pool, query.u, query.h).await {
        return Response::builder().body(Body::from("error: pass")).unwrap();
    }

    let mut ub = URLBuilder::new();

    ub.set_protocol("https")
        .set_host("mirror.lisek.cc")
        .add_route("api")
        .add_route("v1");

    if query.s.is_some() {
        ub.add_route("beatmapsets")
            .add_route(query.s.unwrap().to_string().as_str());
    }

    if query.b.is_some() {
        ub.add_route("beatmapsets")
            .add_route("beatmap")
            .add_route(query.b.unwrap().to_string().as_str());
    }

    if query.c.is_some() {
        ub.add_route("beatmaps")
            .add_route("md5")
            .add_route(query.c.unwrap().to_string().as_str());
    }

    let request = reqwest::get(&ub.build()).await.unwrap();

    if request.status() != StatusCode::OK {
        return Response::builder().body(Body::from("-1|Unable to fetch beatmap")).unwrap();
    }

    let json: serde_json::Value = request.json().await.unwrap();
    let beatmap_set: DirectBeatmapSet = serde_json::from_value(json).unwrap();

    let mut body = format!("1\n"); // format -> String
    let mut diffs: Vec<String> = vec![];

    for diff in beatmap_set.beatmaps {
        diffs.push(format!(
            "[{:.2}*] {}@{}",
            diff.difficulty_rating,
            normalize_direct_name(diff.version),
            diff.mode_int
        ))
    }

    body += format!(
        "{}.osz|{}|{}|{}|{}|0|{}|{}|0|0|0|0|0|{}\n",
        beatmap_set.id,
        beatmap_set.artist,
        beatmap_set.title,
        beatmap_set.creator,
        beatmap_set.ranked,
        beatmap_set.last_updated,
        beatmap_set.id,
        diffs.join(", ")
    )
    .as_str();

    return Response::builder().body(Body::from(body)).unwrap();
}

pub async fn download_osz(Path(mut osz_id): Path<String>) -> Response {
    if osz_id.contains("n") {
        osz_id = osz_id.split_once("n").unwrap().0.to_string();
    }

    return Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header(
            "Location",
            format!("https://mirror.lisek.cc/api/v1/download/{osz_id}").as_str(),
        )
        .body(Body::from(()))
        .unwrap();
}
