use std::collections::HashMap;

use super::beatmap_utils::Beatmap;

pub struct Chart {
    pub chart_id: String,
    pub chart_url: String,
    pub chart_name: String,
    pub achievements: String,
    pub score_id: i32,
    pub rank_before: i32,
    pub rank_after: i32,
    pub accruacy_before: f64,
    pub accuracy_after: f64,
    pub ranked_score_before: i64,
    pub ranked_score_after: i64,
    pub combo_before: i32,
    pub combo_after: i32,
    pub total_score_before: i64,
    pub total_score_after: i64,
    pub performance_before: f64,
    pub performance_after: f64,
}

impl Chart {
    fn to_string(&self) -> String {
        let mut data = HashMap::new();

        data.insert("chartId", self.chart_id.to_string());
        data.insert("chartUrl", self.chart_url.to_string());
        data.insert("chartName", self.chart_name.to_string());
        data.insert("rankBefore", self.rank_before.to_string());
        data.insert("rankAfter", self.rank_after.to_string());
        data.insert("maxComboBefore", self.combo_before.to_string());
        data.insert("maxComboAfter", self.combo_after.to_string());
        data.insert("accuracyBefore", self.accruacy_before.to_string());
        data.insert("accuracyAfter", self.accuracy_after.to_string());
        data.insert("rankedScoreBefore", self.ranked_score_before.to_string());
        data.insert("rankedScoreAfter", self.ranked_score_after.to_string());
        data.insert("totalScoreBefore", self.total_score_before.to_string());
        data.insert("totalScoreAfter", self.total_score_after.to_string());
        data.insert("ppBefore", self.performance_before.to_string());
        data.insert("ppAfter", self.performance_after.to_string());
        data.insert("achievements-new", "".to_string()); //TODO: Medals
        data.insert("onlineScoreId", self.score_id.to_string()); //TODO: Medals

        let mut result = Vec::new();

        for (key, value) in data {
            result.push(format!("{}:{}", key, value))
        }

        result.join("|")
    }

    pub fn build(beatmap: &Beatmap, beatmap_chart: Self, overall: Self) -> String {
        format!("beatmapId:{}|beatmapSetId:{}|beatmapPlaycount:0|beatmapPasscount:0|approvedDate:\n\n{}\n{}", beatmap.beatmap_id, beatmap.parent_id, beatmap_chart.to_string(), overall.to_string())
    }
}
