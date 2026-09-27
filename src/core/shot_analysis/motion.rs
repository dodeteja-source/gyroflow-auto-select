// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Clone, Copy, Debug)]
pub struct MotionSample {
    pub timestamp_ms: f64,
    pub gyro: [f64; 3],
    pub accel: Option<[f64; 3]>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MotionMetrics {
    pub start_ms: f64,
    pub end_ms: f64,
    pub rotation: f64,
    pub angular_acceleration: f64,
    pub jitter: f64,
    pub smoothness: f64,
}

fn magnitude(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 }
}

fn variance(values: &[f64], avg: f64) -> f64 {
    if values.len() < 2 { return 0.0; }
    values.iter().map(|v| {
        let d = *v - avg;
        d * d
    }).sum::<f64>() / (values.len() - 1) as f64
}

/// Converts normalized gyro samples into overlapping analysis windows.
pub fn analyze_motion(samples: &[MotionSample], window_ms: f64, step_ms: f64) -> Vec<MotionMetrics> {
    if samples.len() < 2 || window_ms <= 0.0 || step_ms <= 0.0 {
        return Vec::new();
    }

    let first = samples.first().unwrap().timestamp_ms;
    let last = samples.last().unwrap().timestamp_ms;
    let mut start = first;
    let mut output = Vec::new();

    while start < last {
        let end = start + window_ms;
        let window = samples.iter()
            .filter(|s| s.timestamp_ms >= start && s.timestamp_ms < end)
            .collect::<Vec<_>>();

        if window.len() >= 2 {
            let rates = window.iter().map(|s| magnitude(s.gyro)).collect::<Vec<_>>();
            let mean_rate = mean(&rates);

            let mut accels = Vec::with_capacity(window.len().saturating_sub(1));
            for pair in window.windows(2) {
                let dt = (pair[1].timestamp_ms - pair[0].timestamp_ms) / 1000.0;
                if dt > 0.0 {
                    let delta = [
                        pair[1].gyro[0] - pair[0].gyro[0],
                        pair[1].gyro[1] - pair[0].gyro[1],
                        pair[1].gyro[2] - pair[0].gyro[2],
                    ];
                    accels.push(magnitude(delta) / dt);
                }
            }

            let jitter = variance(&rates, mean_rate).sqrt();
            let angular_acceleration = mean(&accels);
            let smoothness = 1.0 / (1.0 + jitter + angular_acceleration * 0.02);

            // Use the analysis-window boundary rather than the last sample timestamp.
            // This keeps a 1 s window represented as 1 s in the resulting timeline.
            output.push(MotionMetrics {
                start_ms: start,
                end_ms: end.min(last),
                rotation: mean_rate,
                angular_acceleration,
                jitter,
                smoothness,
            });
        }

        start += step_ms;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_gyro_has_low_jitter() {
        let samples = (0..20).map(|i| MotionSample {
            timestamp_ms: i as f64 * 50.0,
            gyro: [1.0, 0.0, 0.0],
            accel: None,
        }).collect::<Vec<_>>();

        let m = analyze_motion(&samples, 500.0, 500.0);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].start_ms, 0.0);
        assert_eq!(m[0].end_ms, 500.0);
        assert!(m[0].jitter < 0.001);
        assert!(m[0].angular_acceleration < 0.001);
        assert!(m[0].smoothness > 0.99);
    }
}
