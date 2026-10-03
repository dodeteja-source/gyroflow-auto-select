// SPDX-License-Identifier: GPL-3.0-or-later

use super::scoring::ShotScore;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SegmentConfig {
    pub min_score: f64,
    pub min_duration_ms: f64,
    pub merge_gap_ms: f64,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            min_score: 70.0,
            min_duration_ms: 1500.0,
            merge_gap_ms: 750.0,
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct GoodSegment {
    pub start_ms: f64,
    pub end_ms: f64,
    pub score: f64,
}

pub fn build_segments(scores: &[ShotScore], config: &SegmentConfig) -> Vec<GoodSegment> {
    let mut segments: Vec<GoodSegment> = Vec::new();

    for score in scores.iter().filter(|s| s.total_score >= config.min_score) {
        if let Some(last) = segments.last_mut() {
            if score.start_ms <= last.end_ms + config.merge_gap_ms {
                let previous_duration = (last.end_ms - last.start_ms).max(0.0);
                let added_duration = (score.end_ms - score.start_ms).max(0.0);
                let total_duration = previous_duration + added_duration;

                last.end_ms = last.end_ms.max(score.end_ms);
                if total_duration > 0.0 {
                    last.score = (
                        last.score * previous_duration +
                        score.total_score * added_duration
                    ) / total_duration;
                } else {
                    last.score = score.total_score;
                }
                continue;
            }
        }

        segments.push(GoodSegment {
            start_ms: score.start_ms,
            end_ms: score.end_ms,
            score: score.total_score,
        });
    }

    segments.retain(|s| s.end_ms - s.start_ms >= config.min_duration_ms);
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(start: f64, end: f64, score: f64) -> ShotScore {
        ShotScore {
            start_ms: start, end_ms: end, total_score: score,
            smoothness: 0.0, jitter: 0.0, acceleration: 0.0, rotation: 0.0,
        }
    }

    #[test]
    fn merges_nearby_good_windows() {
        let scores = vec![s(0.0, 1000.0, 80.0), s(1000.0, 2000.0, 90.0)];
        let cfg = SegmentConfig { min_score: 70.0, min_duration_ms: 1000.0, merge_gap_ms: 100.0 };
        let out = build_segments(&scores, &cfg);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].start_ms, 0.0);
        assert_eq!(out[0].end_ms, 2000.0);
        assert_eq!(out[0].score, 85.0);
    }
}
