use crate::{
    bancho::presence::Presence,
    utils::{
        beatmap_utils::get_beatmap_by_id,
        general_utils::to_fixed,
        performance_utils::calculate_performance_with_accuracy_list,
        score_utils::{format_mods, parse_mods},
    },
};

use super::mio::MioBot;

pub fn roll(_bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    let mut max = 100;

    if let Some(arg) = args.get(0) {
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

pub async fn with(bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    let beatmaps = bot.user_beatmaps.lock().await;
    let beatmap = beatmaps.get(&author.user.id);

    if let None = beatmap {
        return Some("Please, np beatmap first".to_string());
    }

    let beatmap = beatmap.unwrap();

    let mods_string = args.get(0);

    if let None = mods_string {
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

            return Some(response);
            // bot.handle_response(author.user.username.to_string(), response, author).await
        }
        Err(_) => {
            return Some("Failed to calculate performance for this beatmap".to_string());
            // bot.handle_response(author.user.username.to_string(), , author).await;
        }
    }
}

pub async fn acc(bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    let beatmaps = bot.user_beatmaps.lock().await;
    let beatmap = beatmaps.get(&author.user.id);

    if let None = beatmap {
        return Some("Please, np beatmap first".to_string());
    }

    let beatmap = beatmap.unwrap();

    let mods_string = args.get(0);

    if let None = mods_string {
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

            return Some(response);
            // bot.handle_response(author.user.username.to_string(), response, author).await
        }
        Err(_) => {
            return Some("Failed to calculate performance for this beatmap".to_string());
            // bot.handle_response(author.user.username.to_string(), , author).await;
        }
    }
}

pub async fn map(bot: &mut MioBot, author: &Presence, args: Vec<String>) -> Option<String> {
    if author.user.permissions & 4 == 0 {
        return Some("Not enough permissions".to_string());
    }
    let beatmaps = bot.user_beatmaps.lock().await;
    let beatmap = beatmaps.get(&author.user.id);

    if let None = beatmap {
        return Some("Please, np beatmap first".to_string());
    }

    let beatmap = beatmap.unwrap();

    let ranked_status_string = args.get(0);
    let usage = "Usage: !map <loved/ranked/unranked> <set/map>";
    let ranked_statuses = vec!["loved", "ranked", "unranked"];
    let ranking_types = vec!["set", "map"];

    if let None = ranked_status_string {
        return Some(usage.to_string());
    }

    let ranked_status = ranked_status_string.unwrap();
    if !ranked_statuses.contains(&ranked_status.as_str()) {
        return Some(usage.to_string());
    }

    let ranking_type = args.get(1);

    if let None = ranking_type {
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

    if let None = current_beatmap {
        return Some(
            "Looks like beatmap not in database, consider fetching leaderboard again.".to_string(),
        );
    }

    let current_beatmap = current_beatmap.unwrap();

    match ranking_type.as_str() {
        "set" => {
            let _ = sqlx::query!(
                r#"UPDATE "Beatmap" SET "status" = $1 WHERE "parentId" = $2"#,
                new_beatmap_status,
                beatmap.parent_id
            )
            .execute(&*bot.ctx.pool)
            .await;

            let beatmaps = sqlx::query!(
                r#"SELECT "checksum", "status" FROM "Beatmap" WHERE "parentId" = $1"#,
                beatmap.parent_id
            )
            .fetch_all(&*bot.ctx.pool)
            .await;
            match beatmaps {
                Ok(records) => {
                    for record in records {
                        if record.status == 2 {
                            let _ = sqlx::query!(r#"UPDATE "Score" SET "status" = 0 WHERE "beatmapChecksum" = $1 AND "status" = 2"#, record.checksum).execute(&*bot.ctx.pool).await;
                        }
                    }
                }
                _ => {}
            }

            return Some(format!(
                "Updated status for set {} - {}",
                beatmap.artist, beatmap.title
            ));
        }
        "map" => {
            let _ = sqlx::query!(
                r#"UPDATE "Beatmap" SET "status" = $1 WHERE "checksum" = $2"#,
                new_beatmap_status,
                beatmap.checksum
            )
            .execute(&*bot.ctx.pool)
            .await;

            if current_beatmap.status == 2 {
                let _ = sqlx::query!(r#"UPDATE "Score" SET "status" = 0 WHERE "beatmapChecksum" = $1 AND "status" = 2"#, current_beatmap.checksum).execute(&*bot.ctx.pool).await;
            }

            return Some(format!(
                "Updated status for beatmap {} - {}[{}]",
                beatmap.artist, beatmap.title, beatmap.version
            ));
        }
        _ => return Some("Unknown".to_string()),
    }
}
