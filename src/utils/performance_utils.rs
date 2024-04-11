use akatsuki_pp::{osu_2019::OsuPP, AnyPP, AttributeProvider, Beatmap, BeatmapExt, GameMode, Mods};
use sqlx::{Pool, Postgres};
use tracing::{debug, info};

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
                                        // let performance = beatmap
                                        //     .pp()
                                        //     .accuracy(acc)
                                        //     .mode(beatmap.mode)
                                        //     .mods(mods.unwrap_or(0))
                                        //     .calculate();
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

                                        let calc =
                                            AnyPP::new(&beatmap).accuracy(acc).mode(beatmap.mode);

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
pub async fn caculate_performance_safe(
    beatmap_id: i64,
    mods: u32,
    n300: usize,
    n100: usize,
    n50: usize,
    _n_geki: usize,
    _n_katu: usize,
    nmiss: usize,
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
                    // let performance = beatmap
                    //     .pp()
                    //     .mods(mods)
                    //     .n300(n300)
                    //     .n100(n100)
                    //     .n50(n50)
                    //     .n_geki(n_geki)
                    //     .n_katu(n_katu)
                    //     .passed_objects(n300 + n100 + n50 + nmiss)
                    //     .n_misses(nmiss)
                    //     .mode(convert_mode(mode))
                    //     .calculate();
                    if mode == OsuMode::Osu || mode == OsuMode::Relax {
                        let calc = OsuPP::new(&beatmap)
                            .mods(mods)
                            .n300(n300)
                            .n100(n100)
                            .n50(n50)
                            .passed_objects(n300 + n100 + n50 + nmiss)
                            .misses(nmiss);

                        let attrs = calc.calculate();

                        return attrs.pp;
                    }
                    let calc = AnyPP::new(&beatmap)
                        .mods(mods)
                        .n300(n300)
                        .n100(n100)
                        .n50(n50)
                        .passed_objects(n300 + n100 + n50 + nmiss)
                        .n_misses(nmiss)
                        .mode(convert_mode(mode));

                    let attrs = calc.calculate();

                    return attrs.pp();
                }
                Err(_) => {
                    return 0.0;
                }
            }
        }
        Err(_) => {
            return 0.0;
        }
    }
}

pub fn is_cap_reached(score: &UserScoreWithBeatmap) -> bool {
    if is_verified(&score.user) {
        return false;
    }

    match score.score.playmode {
        0 => score.score.performance >= 727.0,
        1 => score.score.performance >= 800.0,
        2 => score.score.performance >= 2300.0,
        3 => score.score.performance >= 1200.0,
        4 => score.score.performance >= 1800.0,
        _ => false,
    }
}
