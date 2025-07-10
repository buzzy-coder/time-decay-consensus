// src/threshold.rs

#[derive(Debug, Clone)]
pub enum EscalationPattern {
    Linear(f64),          // rate: e.g., 0.01 means +1% per second
    Exponential(f64),     // factor: e.g., 0.001 for gradual curve
    Sigmoid(f64, f64),    // (k, midpoint): smooth S-curve
}

#[derive(Debug, Clone)]
pub enum ProgressionProfile {
    Conservative,
    Aggressive,
    Adaptive,
}

#[derive(Debug)]
pub struct ThresholdEscalator {
    pub base_threshold: f64,     // Starting threshold (e.g., 0.51)
    pub ceiling: f64,            // Maximum threshold (e.g., 0.9)
    pub pattern: EscalationPattern,
    pub emergency_override: bool,
    pub profile: ProgressionProfile,
    pub total_votes: usize,
}

impl ThresholdEscalator {
    /// Base threshold calculation without progression profile
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

    /// Wrapper that adjusts time based on progression profile
    pub fn threshold_with_profile(&self, now: chrono::DateTime<chrono::Utc>, start: chrono::DateTime<chrono::Utc>) -> f64 {
        if self.emergency_override {
            return self.ceiling;
        }

        let elapsed_secs = (now - start).num_seconds().max(0) as u64;
        let adjusted_secs = match self.profile {
            ProgressionProfile::Conservative => elapsed_secs,
            ProgressionProfile::Aggressive => elapsed_secs * 2,
            ProgressionProfile::Adaptive => {
                if self.total_votes < 3 {
                    elapsed_secs * 3
                } else {
                    elapsed_secs
                }
            }
        };

        self.current_threshold(adjusted_secs)
    }

    /// Multi-dimensional threshold check: weight + vote count
    pub fn is_threshold_met(&self, weight: f64, required: f64) -> bool {
        weight >= required && self.total_votes >= 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mock_escalator(pattern: EscalationPattern, profile: ProgressionProfile, votes: usize) -> ThresholdEscalator {
        ThresholdEscalator {
            base_threshold: 0.5,
            ceiling: 0.9,
            pattern,
            emergency_override: false,
            profile,
            total_votes: votes,
        }
    }

    #[test]
    fn test_linear_with_profile() {
        let esc = mock_escalator(EscalationPattern::Linear(0.01), ProgressionProfile::Aggressive, 5);
        assert!((esc.threshold_with_profile(Utc::now(), Utc::now() - chrono::Duration::seconds(30)) - 0.5 - 0.01 * 60.0).abs() < 0.01);
    }

    #[test]
    fn test_exponential_with_profile() {
        let esc = mock_escalator(EscalationPattern::Exponential(0.01), ProgressionProfile::Conservative, 1);
        let threshold = esc.threshold_with_profile(Utc::now(), Utc::now() - chrono::Duration::seconds(30));
        assert!(threshold > 0.5);
    }

    #[test]
    fn test_sigmoid_with_profile() {
        let esc = mock_escalator(EscalationPattern::Sigmoid(0.1, 30.0), ProgressionProfile::Adaptive, 1);
        let threshold = esc.threshold_with_profile(Utc::now(), Utc::now() - chrono::Duration::seconds(60));
        assert!(threshold > 0.5);
    }

    #[test]
    fn test_emergency_override() {
        let mut esc = mock_escalator(EscalationPattern::Linear(0.01), ProgressionProfile::Conservative, 10);
        esc.emergency_override = true;
        assert_eq!(esc.threshold_with_profile(Utc::now(), Utc::now()), esc.ceiling);
    }

    #[test]
    fn test_threshold_met_logic() {
        let esc = mock_escalator(EscalationPattern::Linear(0.01), ProgressionProfile::Adaptive, 3);
        assert!(esc.is_threshold_met(0.75, 0.7));
        assert!(!esc.is_threshold_met(0.65, 0.7));
        assert!(!esc.is_threshold_met(0.75, 0.7) == false);
    }
}
