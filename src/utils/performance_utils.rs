use akatsuki_pp::{osu_2019::OsuPP, AnyPP, Beatmap, GameMode};
use sqlx::{Pool, Postgres};
use tracing::debug;

use crate::utils::{beatmap_utils::get_beatmap_file, general_utils::to_fixed};

use super::{
    beatmap_utils::get_beatmap_by_id,
    http_utils::OsuMode,
    score_utils::{OsuServerError, UserScoreWithBeatmap},
    user_utils::is_verified,
};

#[derive(Debug)]
pub struct CalculationResult {
    pub accuracy: f64,
    pub performance: f64,
    pub stars: f64,
}

pub async fn calculate_performance_with_accuracy_list(
    connection: &Pool<Postgres>,
    id: i64,
    accuracy: Vec<f64>,
    mods: Option<u32>,
) -> Result<Vec<CalculationResult>, OsuServerError> {
    let beatmap = get_beatmap_by_id(connection, id).await;

    let mut result: Vec<CalculationResult> = vec![];

    match beatmap {
        Err(_) => {
            return Err(OsuServerError::BeatmapProcessingFailed(
                "Failed to process beatmap.".to_string(),
            ));
        }
        Ok(beatmap) => {
            if let Some(online_beatmap) = beatmap {
                debug!("Got beatmap: {:?}", online_beatmap);
                let file = get_beatmap_file(online_beatmap.beatmap_id as i64).await;

                match file {
                    Ok(beatmap_bytes) => match beatmap_bytes {
                        None => {}
                        Some(beatmap_bytes) => {
                            let beatmap = Beatmap::from_bytes(&beatmap_bytes);

                            match beatmap {
                                Err(_) => {
                                    return Err(OsuServerError::BeatmapProcessingFailed(
                                        "Failed to process beatmap.".to_string(),
                                    ));
                                }
                                Ok(beatmap) => {
                                    for acc in accuracy {
                                        if beatmap.mode == GameMode::Osu
                                            && mods.unwrap_or(0) & 128 == 128
                                        {
                                            let calc = OsuPP::new(&beatmap)
                                                .mods(mods.unwrap_or(0))
                                                .accuracy(acc as f32);

                                            let attrs = calc.calculate();

                                            // return attrs.pp;
                                            result.push(CalculationResult {
                                                accuracy: acc,
                                                performance: to_fixed(attrs.pp, 2),
                                                stars: 0.0,
                                            });
                                            continue;
                                        }

                                        let calc = AnyPP::new(&beatmap)
                                            .accuracy(acc)
                                            .mods(mods.unwrap_or(0))
                                            .mode(beatmap.mode);

                                        let attrs = calc.calculate();

                                        // return attrs.pp();
                                        result.push(CalculationResult {
                                            accuracy: acc,
                                            performance: to_fixed(attrs.pp(), 2),
                                            stars: attrs.stars(),
                                        });
                                    }
                                }
                            }
                        }
                    },
                    Err(_) => {
                        return Err(OsuServerError::BeatmapProcessingFailed(
                            "Failed to process beatmap.".to_string(),
                        ));
                    }
                }
            }
        }
    }

    Ok(result)
}

fn convert_mode(mode: OsuMode) -> GameMode {
    match mode {
        OsuMode::Osu => GameMode::Osu,
        OsuMode::Taiko => GameMode::Taiko,
        OsuMode::Fruits => GameMode::Catch,
        OsuMode::Mania => GameMode::Mania,
        OsuMode::Relax => GameMode::Osu,
    }
}

pub async fn calculate_performance_safe(
    beatmap_id: i64,
    mods: u32,
    n300: usize,
    n100: usize,
    n50: usize,
    n_geki: usize,
    n_katu: usize,
    n_miss: usize,
    combo: usize,
    mode: OsuMode,
) -> f64 {
    let beatmap_file = get_beatmap_file(beatmap_id).await;

    match beatmap_file {
        Ok(beatmap_file) => {
            if beatmap_file.is_none() {
                return 0.0;
            }

            let beatmap_file = beatmap_file.unwrap();

            match Beatmap::from_bytes(&beatmap_file) {
                Ok(beatmap) => {
                    if mode == OsuMode::Relax {
                        let calc = OsuPP::new(&beatmap)
                            .mods(mods)
                            .combo(combo)
                            .n300(n300)
                            .n100(n100)
                            .n50(n50)
                            .misses(n_miss);

                        return calc.calculate().pp;
                    }

                    let calc = AnyPP::new(&beatmap)
                        .mods(mods)
                        .combo(combo)
                        .n300(n300)
                        .n_geki(n_geki)
                        .n100(n100)
                        .n_katu(n_katu)
                        .n50(n50)
                        .n_misses(n_miss)
                        .mode(convert_mode(mode));

                    let attrs = calc.calculate();

                    attrs.pp()
                }
                Err(_) => 0.0,
            }
        }
        Err(_) => 0.0,
    }
}

pub fn is_cap_reached(score: &UserScoreWithBeatmap) -> bool {
    if is_verified(&score.user) {
        return false;
    }

    return score.score.performance > get_pp_cap(score.score.playmode);
    // match score.score.playmode {
    //     0 => score.score.performance >= 727.0,
    //     1 => score.score.performance >= 800.0,
    //     2 => score.score.performance >= 2300.0,
    //     3 => score.score.performance >= 1200.0,
    //     4 => score.score.performance >= 1800.0,
    //     _ => false,
    // }
}

pub fn get_pp_cap(play_mode: i32) -> f64 {
    match play_mode {
        0 => 727.0,
        1 => 800.0,
        2 => 2300.0,
        3 => 1200.0,
        4 => 1800.0,
        _ => 9999.0,
    }
}
