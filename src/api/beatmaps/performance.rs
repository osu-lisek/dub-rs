use akatsuki_pp::{Beatmap, BeatmapExt};
use axum::{extract, Json};
use serde::{Deserialize, Serialize};

use crate::utils::beatmap_utils::get_beatmap_file;

#[derive(Deserialize)]
pub struct CalculationRequestBody {
    pub beatmap_id: i64,
    pub mods: u32,
    pub accuracy: f64,
    pub misses: usize,
    pub count_100: usize,
    pub count_50: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CalculationResult {
    pub performance: Option<f64>,
    pub star_rating: Option<f64>,
    pub ok: bool,
}

pub async fn _calculate_performance(
    extract::Json(body): extract::Json<CalculationRequestBody>,
) -> Json<CalculationResult> {
    let beatmap_file = get_beatmap_file(body.beatmap_id).await;

    match beatmap_file {
        Ok(beatmap_file) => {
            if beatmap_file.is_none() {
                return Json(CalculationResult {
                    performance: None,
                    star_rating: None,
                    ok: false,
                });
            }

            let beatmap_file = beatmap_file.unwrap();

            match Beatmap::from_bytes(&beatmap_file) {
                Ok(beatmap) => {
                    let performance = beatmap
                        .pp()
                        .mods(body.mods)
                        .accuracy(body.accuracy)
                        .n100(body.count_100)
                        .n50(body.count_50)
                        .n_misses(body.misses)
                        .calculate();

                    let stars = beatmap.stars().mods(body.mods).calculate().stars();

                    Json(CalculationResult {
                        performance: Some(performance.pp()),
                        star_rating: Some(stars),
                        ok: true,
                    })
                }
                Err(_) => Json(CalculationResult {
                    performance: None,
                    star_rating: None,
                    ok: false,
                }),
            }
        }
        Err(_) => {
            return Json(CalculationResult {
                performance: None,
                star_rating: None,
                ok: false,
            })
        }
    }
}
