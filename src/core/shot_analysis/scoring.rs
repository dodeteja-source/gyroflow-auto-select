// SPDX-License-Identifier: GPL-3.0-or-later

use super::motion::MotionMetrics;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScoreConfig {
    pub max_rotation_deg_s: f64,
    pub max_acceleration_deg_s2: f64,
    pub max_jitter_deg_s: f64,
    pub min_score: f64,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            max_rotation_deg_s: 180.0,
            max_acceleration_deg_s2: 500.0,
            max_jitter_deg_s: 25.0,
            min_score: 70.0,
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct ShotScore {
    pub start_ms: f64,
    pub end_ms: f64,
    pub smoothness: f64,
    pub jitter: f64,
    pub acceleration: f64,
    pub rotation: f64,
    pub total_score: f64,
}

fn component(value: f64, limit: f64) -> f64 {
    if limit <= 0.0 { return 0.0; }
    (1.0 - value / limit).clamp(0.0, 1.0) * 100.0
}

pub fn score_window(metrics: &MotionMetrics, config: &ScoreConfig) -> ShotScore {
    // Smoothness is deliberately weighted most heavily: a slow, stable camera
    // move should score better than a stationary shot containing high-frequency jitter.
    let rotation_score = component(metrics.rotation, config.max_rotation_deg_s);
    let acceleration_score = component(metrics.angular_acceleration, config.max_acceleration_deg_s2);
    let jitter_score = component(metrics.jitter, config.max_jitter_deg_s);

    let total = (
        rotation_score * 0.25 +
        acceleration_score * 0.30 +
        jitter_score * 0.35 +
        metrics.smoothness.clamp(0.0, 1.0) * 100.0 * 0.10
    ).clamp(0.0, 100.0);

    ShotScore {
        start_ms: metrics.start_ms,
        end_ms: metrics.end_ms,
        smoothness: metrics.smoothness,
        jitter: metrics.jitter,
        acceleration: metrics.angular_acceleration,
        rotation: metrics.rotation,
        total_score: total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_motion_scores_higher_than_jitter() {
        let stable = MotionMetrics {
            start_ms: 0.0, end_ms: 1000.0,
            rotation: 20.0, angular_acceleration: 10.0,
            jitter: 1.0, smoothness: 0.95,
        };
        let jitter = MotionMetrics {
            start_ms: 0.0, end_ms: 1000.0,
            rotation: 20.0, angular_acceleration: 450.0,
            jitter: 30.0, smoothness: 0.1,
        };
        assert!(score_window(&stable, &ScoreConfig::default()).total_score >
                score_window(&jitter, &ScoreConfig::default()).total_score);
    }
}
