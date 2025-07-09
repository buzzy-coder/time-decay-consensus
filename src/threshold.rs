// src/threshold.rs

#[derive(Debug, Clone)]
pub enum EscalationPattern {
    Linear(f64),          // rate: e.g., 0.01 means +1% per second
    Exponential(f64),     // factor: e.g., 0.001 for gradual curve
    Sigmoid(f64, f64),    // (k, midpoint): smooth S-curve
}

#[derive(Debug)]
pub struct ThresholdEscalator {
    pub base_threshold: f64,     // Starting threshold (e.g., 0.51)
    pub ceiling: f64,            // Maximum threshold (e.g., 0.9)
    pub pattern: EscalationPattern,
    pub emergency_override: bool,
}

impl ThresholdEscalator {
    /// Compute the current threshold based on elapsed time
    pub fn current_threshold(&self, elapsed_secs: u64) -> f64 {
        if self.emergency_override {
            return self.ceiling; // Max threshold for critical situations
        }

        match self.pattern {
            EscalationPattern::Linear(rate) => {
                let increase = rate * elapsed_secs as f64;
                (self.base_threshold + increase).min(self.ceiling)
            }
            EscalationPattern::Exponential(factor) => {
                let increase = 1.0 - (-factor * elapsed_secs as f64).exp();
                (self.base_threshold + increase).min(self.ceiling)
            }
            EscalationPattern::Sigmoid(k, midpoint) => {
                let x = elapsed_secs as f64;
                let sigmoid = 1.0 / (1.0 + (-k * (x - midpoint)).exp());
                let threshold_range = self.ceiling - self.base_threshold;
                self.base_threshold + sigmoid * threshold_range
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_escalation() {
        let esc = ThresholdEscalator {
            base_threshold: 0.51,
            ceiling: 0.9,
            pattern: EscalationPattern::Linear(0.01),
            emergency_override: false,
        };
        assert!((esc.current_threshold(10) - 0.61).abs() < 0.001);
        assert!((esc.current_threshold(100) - 0.9).abs() < 0.001); // capped
    }

    #[test]
    fn test_exponential_escalation() {
        let esc = ThresholdEscalator {
            base_threshold: 0.5,
            ceiling: 0.9,
            pattern: EscalationPattern::Exponential(0.01),
            emergency_override: false,
        };
        let t30 = esc.current_threshold(30);
        let t60 = esc.current_threshold(60);
        assert!(t60 > t30);
    }

    #[test]
    fn test_sigmoid_escalation() {
        let esc = ThresholdEscalator {
            base_threshold: 0.5,
            ceiling: 0.9,
            pattern: EscalationPattern::Sigmoid(0.1, 30.0),
            emergency_override: false,
        };
        let low = esc.current_threshold(0);
        let mid = esc.current_threshold(30);
        let high = esc.current_threshold(60);
        assert!(low < mid && mid < high);
    }

    #[test]
    fn test_emergency_override() {
        let esc = ThresholdEscalator {
            base_threshold: 0.51,
            ceiling: 0.9,
            pattern: EscalationPattern::Linear(0.01),
            emergency_override: true,
        };
        assert_eq!(esc.current_threshold(0), 0.9);
        assert_eq!(esc.current_threshold(100), 0.9);
    }
}
