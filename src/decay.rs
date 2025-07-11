
pub trait DecayModel {
    fn compute_weight(&self, original_weight: f64, elapsed_time: f64) -> f64;
}

pub struct LinearDecay {
    pub rate: f64,
}

impl DecayModel for LinearDecay {
    fn compute_weight(&self, original_weight: f64, elapsed_time: f64) -> f64 {
        let decayed = original_weight - self.rate * elapsed_time as f64;
        decayed.max(0.1 * original_weight)
    }
}

pub struct ExponentialDecay {
    pub rate: f64,
}

impl DecayModel for ExponentialDecay {
    fn compute_weight(&self, original_weight: f64, elapsed_time: f64) -> f64 {
        let decayed = original_weight * (-self.rate * elapsed_time as f64).exp();
        decayed.max(0.1 * original_weight)
    }
}

pub struct SteppedDecay {
    pub decay_steps: Vec<(f64, f64)>, // (time, weight)
}

impl DecayModel for SteppedDecay {
    fn compute_weight(&self, original_weight: f64, elapsed_time: f64) -> f64 {
        let mut multiplier = 1.0;
        for &(threshold, factor) in &self.decay_steps {
            if elapsed_time >= threshold {
                multiplier = factor;
            }
        }
        (original_weight * multiplier).max(0.1 * original_weight)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_decay() {
        let model = LinearDecay { rate: 2.0 };
        let w0 = 100.0;

        // No time passed
        assert_eq!(model.compute_weight(w0, 0.0), w0);

        // Some decay
        let w = model.compute_weight(w0, 10.0);
        assert_eq!(w, 100.0 - 2.0 * 10.0);

        // Below 10% of original should clamp
        let w = model.compute_weight(w0, 100.0);
        assert_eq!(w, 0.1 * w0);
    }

    #[test]
    fn test_exponential_decay() {
        let model = ExponentialDecay { rate: 0.1 };
        let w0 = 100.0;

        // No time passed
        let w = model.compute_weight(w0, 0.0);
        assert_eq!(w, w0);

        // After some time
        let w = model.compute_weight(w0, 10.0);
        let expected = w0 * (-0.1_f64 * 10.0).exp();
        assert!((w - expected).abs() < 1e-6);

        // Should not drop below 10% of original
        let w = model.compute_weight(w0, 100.0);
        assert!(w >= 0.1 * w0);
    }

    #[test]
    fn test_stepped_decay() {
        let model = SteppedDecay {
            decay_steps: vec![
                (5.0, 0.8),
                (10.0, 0.5),
                (20.0, 0.2),
            ],
        };
        let w0 = 100.0;

        // Before first step
        let w = model.compute_weight(w0, 2.0);
        assert_eq!(w, w0);

        // After first step
        let w = model.compute_weight(w0, 5.0);
        assert_eq!(w, w0 * 0.8);

        // After second step
        let w = model.compute_weight(w0, 15.0);
        assert_eq!(w, w0 * 0.5);

        // After third step
        let w = model.compute_weight(w0, 25.0);
        assert_eq!(w, w0 * 0.2);

        // Should not drop below 10% of original
        let w = model.compute_weight(w0, 100.0);
        assert!(w >= 0.1 * w0);
    }
}