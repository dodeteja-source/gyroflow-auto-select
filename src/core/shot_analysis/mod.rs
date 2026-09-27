// SPDX-License-Identifier: GPL-3.0-or-later

mod motion;
mod scoring;
mod segments;

pub use motion::{MotionMetrics, MotionSample, analyze_motion};
pub use scoring::{ScoreConfig, ShotScore, score_window};
pub use segments::{GoodSegment, SegmentConfig, build_segments};

use crate::gyro_source::FileMetadata;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnalysisConfig {
    pub window_ms: f64,
    pub step_ms: f64,
    pub score: ScoreConfig,
    pub segments: SegmentConfig,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            window_ms: 1000.0,
            step_ms: 500.0,
            score: ScoreConfig::default(),
            segments: SegmentConfig::default(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ShotAnalysis {
    pub scores: Vec<ShotScore>,
    pub good_segments: Vec<GoodSegment>,
}

pub fn analyze(metadata: &FileMetadata, config: &AnalysisConfig) -> ShotAnalysis {
    let samples = metadata.raw_imu.iter()
        .filter_map(|s| s.gyro.map(|gyro| MotionSample {
            timestamp_ms: s.timestamp_ms,
            gyro,
            accel: s.accl,
        }))
        .collect::<Vec<_>>();

    let metrics = analyze_motion(&samples, config.window_ms, config.step_ms);
    let scores = metrics.iter().map(|m| score_window(m, &config.score)).collect::<Vec<_>>();
    let good_segments = build_segments(&scores, &config.segments);

    ShotAnalysis { scores, good_segments }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_one_second_window() {
        let cfg = AnalysisConfig::default();
        assert_eq!(cfg.window_ms, 1000.0);
        assert_eq!(cfg.step_ms, 500.0);
    }
}
