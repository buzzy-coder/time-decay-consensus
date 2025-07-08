
pub trait decay_model {
    fn compute_weight(&self, original_weight: f64, elapsed_time: f64) -> f64;
}

pub struct LinearDecay {
    pub rate: f64,
}

impl decay_model for LinearDecay {
    fn compute_weight(&self, original_weight: f64, elapsed_time: f64) -> f64 {
        let decayed = original_weight - self.rate * elapsed_time as f64;
        decayed.max(0.1 * original_weight)
    }
}

pub struct ExponentialDecay {
    pub rate: f64,
}

impl decay_model for ExponentialDecay {
    fn compute_weight(&self, original_weight: f64, elapsed_time: f64) -> f64 {
        let decayed = original_weight * (-self.rate * elapsed_time as f64).exp();
        decayed.max(0.1 * original_weight)
    }
}

pub struct SteppedDecay {
    pub decay_steps: Vec<(f64, f64)>, // (time, weight)
}

impl decay_model for SteppedDecay {
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
