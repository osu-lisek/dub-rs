use chrono::{NaiveDateTime, Utc};

use crate::{
    bancho::presence::Presence,
    utils::{
        beatmap_utils::{announce_beatmap_status, get_beatmap_by_id, rank_to_str},
        general_utils::to_fixed,
        performance_utils::calculate_performance_with_accuracy_list,
        score_utils::{format_mods, parse_mods},
        user_utils::{
            find_user_by_id_or_username, insert_user_punishment, is_restricted, is_user_manager, punishment_alert, remove_ranking, restrict_user, send_bancho_message, send_message_announcement, unrestrict_user
        },
        Punishment,
    },
    web::scores::submission::BeatmapStatus,
};

use super::mio::MioBot;

pub fn roll(_bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    let mut max = 100;

    if let Some(arg) = args.first() {
        if let Ok(num) = arg.parse::<u32>() {
            max = num;
        }
    }

    Some(format!(
        "{} rolled a {}!",
        author.user.username,
        rand::random::<u32>() % max
    ))
}

pub async fn restrict(bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    if !is_user_manager(&author.user) {
        return Some("No permissions.".to_string());
    }

    let username = args.first();

    if username.is_none() {
        return Some("Usage: !restrict <username> [note]".to_string());
    }

    let username = username.unwrap().to_owned();

    let user = find_user_by_id_or_username(&bot.ctx.pool, username).await;

    if user.is_err() {
        return Some("Failed to fetch user".to_string());
    }

    let user = user.unwrap();

    if user.is_none() {
        return Some("Could not find user.".to_string());
    }

    let user = user.unwrap();

    if is_restricted(&user).await {
        unrestrict_user(&bot.ctx.pool, user.id).await;

        let note = args
            .iter()
            .skip(1)
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
            .join(" ");
        punishment_alert(
            &Punishment {
                id: String::new(),
                date: NaiveDateTime::from_timestamp_millis(Utc::now().timestamp())
                    .unwrap_or(NaiveDateTime::UNIX_EPOCH),
                applied_by: author.user.id,
                applied_to: user.id,
                punishment_type: "Unrestriction".to_string(),
                level: "CRITICAL".to_string(),
                expires: false,
                expires_at: None,
                note,
            },
            &user,
            &author.user,
        )
        .await;
    
        //lifting all punishments
        let _ = sqlx::query!(r#"UPDATE "Punishment" SET "expires" = true, "expiresAt" = '1970-01-01T00:00:00+00:00' WHERE "appliedTo" = $1"#, user.id).execute(&*bot.ctx.pool).await;
        send_bancho_message(&user.id, "user:restricted".to_string(), None).await;
    } else {
        restrict_user(&bot.ctx.pool, user.id).await;

        let note = args
            .iter()
            .skip(1)
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
            .join(" ");
        insert_user_punishment(&bot.ctx.pool, "CRITICAL".to_string(), author.user.id, user.id, "RESTRICTION".to_string(), false, None, note).await;
        remove_ranking(&bot.ctx.redis, &user).await;
        send_bancho_message(&user.id, "user:restricted".to_string(), None).await;
    }

    Some("Done".to_string())
}

pub async fn with(bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    let beatmaps = bot.user_beatmaps.lock().await;
    let beatmap = beatmaps.get(&author.user.id);

    if beatmap.is_none() {
        return Some("Please, np beatmap first".to_string());
    }

    let beatmap = beatmap.unwrap();

    let mods_string = args.first();

    if mods_string.is_none() {
        return Some("Usage: !with <MODS>".to_string());
    }

    let mods_string = mods_string.unwrap();

    let mods = parse_mods(mods_string.to_owned());
    let accuracy_list = vec![100.0, 99.0, 98.0];
    let result = calculate_performance_with_accuracy_list(
        &bot.ctx.pool,
        beatmap.beatmap_id as i64,
        accuracy_list,
        Some(mods),
    )
    .await;

    match result {
        Ok(results) => {
            let mut performance_response: Vec<String> = vec![];
            for result in results {
                performance_response.push(format!(
                    "{}%: {}pp",
                    result.accuracy,
                    to_fixed(result.performance, 2)
                ));
            }

            let response = format!(
                "[osu://b/{} {} - {} [{}]] + {} - {}",
                beatmap.beatmap_id,
                beatmap.artist,
                beatmap.title,
                beatmap.version,
                format_mods(mods),
                performance_response.join(" | ")
            );

            Some(response)
        }
        Err(_) => Some("Failed to calculate performance for this beatmap".to_string()),
    }
}

pub async fn acc(bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    let beatmaps = bot.user_beatmaps.lock().await;
    let beatmap = beatmaps.get(&author.user.id);

    if beatmap.is_none() {
        return Some("Please, np beatmap first".to_string());
    }

    let beatmap = beatmap.unwrap();

    let mods_string = args.first();

    if mods_string.is_none() {
        return Some("Usage: !acc <acc>".to_string());
    }

    let acc = mods_string.unwrap().parse::<f64>().unwrap_or(100_f64);

    let status = author.status.read().await;

    let accuracy_list = vec![acc];
    let result = calculate_performance_with_accuracy_list(
        &bot.ctx.pool,
        beatmap.beatmap_id as i64,
        accuracy_list,
        Some(status.mods),
    )
    .await;

    match result {
        Ok(results) => {
            let mut performance_response: Vec<String> = vec![];
            for result in results {
                performance_response.push(format!(
                    "{}%: {}pp",
                    result.accuracy,
                    to_fixed(result.performance, 2)
                ));
            }

            let response = format!(
                "[osu://b/{} {} - {} [{}]] + {} - {}",
                beatmap.beatmap_id,
                beatmap.artist,
                beatmap.title,
                beatmap.version,
                format_mods(status.mods),
                performance_response.join(" | ")
            );

            Some(response)
        }
        Err(_) => Some("Failed to calculate performance for this beatmap".to_string()),
    }
}

pub async fn map(bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    if author.user.permissions & 4 == 0 {
        return Some("Not enough permissions".to_string());
    }
    let beatmaps = bot.user_beatmaps.lock().await;
    let beatmap = beatmaps.get(&author.user.id);

    if beatmap.is_none() {
        return Some("Please, np beatmap first".to_string());
    }

    let beatmap = beatmap.unwrap();

    let ranked_status_string = args.first();
    let usage = "Usage: !map <loved/ranked/unranked> <set/map>";
    let ranked_statuses = ["loved", "ranked", "unranked"];
    let ranking_types = ["set", "map"];

    if ranked_status_string.is_none() {
        return Some(usage.to_string());
    }

    let ranked_status = ranked_status_string.unwrap();
    if !ranked_statuses.contains(&ranked_status.as_str()) {
        return Some(usage.to_string());
    }

    let ranking_type = args.get(1);

    if ranking_type.is_none() {
        return Some(usage.to_string());
    }

    let ranking_type = ranking_type.unwrap();

    if !ranking_types.contains(&ranking_type.as_str()) {
        return Some(usage.to_string());
    }

    let new_beatmap_status = match ranked_status.as_str() {
        "loved" => 5,
        "ranked" => 2,
        "unranked" => 0,
        _ => 0,
    };

    let current_beatmap = get_beatmap_by_id(&bot.ctx.pool, beatmap.beatmap_id as i64).await;

    if let Err(error) = current_beatmap {
        return Some(format!(
            "Looks like beatmap not in database, consider fetching leaderboard again. ({:#?})",
            error
        ));
    }

    let current_beatmap = current_beatmap.unwrap();

    if current_beatmap.is_none() {
        return Some(
            "Looks like beatmap not in database, consider fetching leaderboard again.".to_string(),
        );
    }

    let current_beatmap = current_beatmap.unwrap();

    match ranking_type.as_str() {
        "set" => {
            let _ = sqlx::query!(
                r#"UPDATE "Beatmap" SET "status" = $1, "updatedStatusById" = $3, "lastStatusUpdate" = $4 WHERE "parentId" = $2"#,
                new_beatmap_status,
                beatmap.parent_id,
                author.user.id,
                NaiveDateTime::from_timestamp_millis(Utc::now().timestamp())
            )
            .execute(&*bot.ctx.pool)
            .await;

            let beatmaps = sqlx::query!(
                r#"SELECT "checksum", "status" FROM "Beatmap" WHERE "parentId" = $1"#,
                beatmap.parent_id
            )
            .fetch_all(&*bot.ctx.pool)
            .await;

            if let Ok(records) = beatmaps {
                for record in records {
                    if record.status == 2 {
                        let _ = sqlx::query!(r#"UPDATE "Score" SET "status" = 0 WHERE "beatmapChecksum" = $1 AND "status" = 2"#, record.checksum).execute(&*bot.ctx.pool).await;
                    }
                }
            }

            send_message_announcement(
                format!(
                    "https://c.{}/api/v2/bancho/notification",
                    bot.ctx.config.server_url
                ),
                format!(
                    "[https://{}/users/{} {}] changed status of [https://{}/b/{} {} - {}] from {} to {}",
                    bot.ctx.config.server_url,
                    author.user.id,
                    author.user.username_safe,
                    bot.ctx.config.server_url,
                    beatmap.parent_id,
                    beatmap.artist,
                    beatmap.title,
                    rank_to_str(&BeatmapStatus::from(beatmap.status)),
                    rank_to_str(&BeatmapStatus::from(new_beatmap_status))
                ),
                "chat".to_string(),
                "#announce".to_string(),
                bot.ctx.config.token_hmac_secret.clone(),
            )
            .await;

            announce_beatmap_status(author, beatmap, &BeatmapStatus::from(new_beatmap_status))
                .await;

            Some(format!(
                "Updated status for set {} - {}",
                beatmap.artist, beatmap.title
            ))
        }
        "map" => {
            let _ = sqlx::query!(
                r#"UPDATE "Beatmap" SET "status" = $1, "updatedStatusById" = $3, "lastStatusUpdate" = $4 WHERE "checksum" = $2"#,
                new_beatmap_status,
                beatmap.checksum,
                author.user.id,
                NaiveDateTime::from_timestamp_millis(Utc::now().timestamp())
            )
            .execute(&*bot.ctx.pool)
            .await;

            if current_beatmap.status == 2 {
                let _ = sqlx::query!(r#"UPDATE "Score" SET "status" = 0 WHERE "beatmapChecksum" = $1 AND "status" = 2"#, current_beatmap.checksum).execute(&*bot.ctx.pool).await;
            }

            Some(format!(
                "Updated status for beatmap {} - {}[{}]",
                beatmap.artist, beatmap.title, beatmap.version
            ))
        }
        _ => Some("Unknown".to_string()),
    }
}
