use tracing::info;

use crate::{
    context::Context,
    utils::{
        http_utils::OsuMode,
        performance_utils::calculate_performance_safe,
        score_utils::{get_user_best_scores, SortMode},
        user_utils::recalculate_user_stats,
    },
};

use tracing::error;

use super::CalculationQueue;

pub async fn process_command(
    _command: String,
    _arguments: Vec<String>,
    ctx: &Context,
    queue: &mut CalculationQueue,
) {
    info!("Processing preview: ");

    let mut scores_to_calculate = queue.score.clone();

    for user in queue.users.clone() {
        for mode in [
            OsuMode::Osu,
            OsuMode::Taiko,
            OsuMode::Fruits,
            OsuMode::Mania,
            OsuMode::Relax,
        ] {
            let scores = get_user_best_scores(
                &ctx.pool,
                &user,
                Some(10000),
                None,
                mode.clone(),
                Some(SortMode::Performance),
            )
            .await;

            if let Ok(scores) = scores {
                info!(
                    "Added {} scores from {} to queue with mode {:#?}",
                    scores.len(),
                    user.username,
                    mode
                );
                scores_to_calculate.extend(scores);
            }
        }
    }

    for score in scores_to_calculate {
        let recalculation_result = calculate_performance_safe(
            score.beatmap.beatmap_id as i64,
            score.score.mods as u32,
            score.score.count_300 as usize,
            score.score.count_100 as usize,
            score.score.count_50 as usize,
            score.score.count_geki as usize,
            score.score.count_katu as usize,
            score.score.count_miss as usize,
            score.score.max_combo as usize,
            OsuMode::from_id(score.score.playmode as u8),
        )
        .await;

        let result = sqlx::query!(
            r#"UPDATE "Score" SET "performance" = $1 WHERE "id" = $2"#,
            recalculation_result,
            score.score.id
        )
        .execute(&*ctx.pool)
        .await;

        if let Err(error) = result {
            error!("Error while updating performance: {}", error);
            continue;
        }

        info!(
            "Recalculated score with id {} by {} on beatmap {} - {} to {} -> {}",
            score.score.id,
            score.user.username,
            score.beatmap.artist,
            score.beatmap.title,
            score.score.performance,
            recalculation_result
        );
    }

    for user in queue.users.clone() {
        for mode in [
            OsuMode::Osu,
            OsuMode::Taiko,
            OsuMode::Fruits,
            OsuMode::Mania,
            OsuMode::Relax,
        ] {
            recalculate_user_stats(&ctx.pool, &ctx.redis, &user, &mode).await;
        }
    }
}
