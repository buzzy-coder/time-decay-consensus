// src/threshold.rs

use crate::vote::ProposalType;

#[derive(Debug, Clone)]
pub enum EscalationPattern {
    Linear(f64),       // rate: e.g., 0.01 means +1% per second
    Exponential(f64),  // factor: e.g., 0.001 for gradual curve
    Sigmoid(f64, f64), // (k, midpoint): smooth S-curve
}

#[derive(Debug, Clone)]
pub enum ProgressionProfile {
    Conservative,
    Aggressive,
    Adaptive,
}

#[derive(Debug)]
pub struct ThresholdEscalator {
    pub base_threshold: f64, // Starting threshold (e.g., 0.51)
    pub ceiling: f64,        // Maximum threshold (e.g., 0.9)
    pub pattern: EscalationPattern,
    pub emergency_override: bool,
    pub profile: ProgressionProfile,
    pub total_votes: usize,
    pub min_vote_count: usize,
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

    pub fn for_proposal_type(proposal_type: ProposalType) -> Self {
        match proposal_type {
            ProposalType::Normal => ThresholdEscalator {
                base_threshold: 0.51,
                ceiling: 0.9,
                pattern: EscalationPattern::Linear(0.01),
                emergency_override: false,
                profile: ProgressionProfile::Conservative,
                total_votes: 0,
                min_vote_count: 3, // Minimum 3 votes required
            },
            ProposalType::Critical => ThresholdEscalator {
                base_threshold: 0.75,
                ceiling: 0.95,
                pattern: EscalationPattern::Linear(0.02),
                emergency_override: false,
                profile: ProgressionProfile::Aggressive,
                total_votes: 0,
                min_vote_count: 5, // Stricter requirement for critical proposals
            },
        }
    }

    /// Wrapper that adjusts time based on progression profile
    pub fn threshold_with_profile(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        start: chrono::DateTime<chrono::Utc>,
    ) -> f64 {
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
    pub fn is_threshold_met(&self, vote_weight: f64, current_threshold: f64) -> bool {
        vote_weight >= current_threshold && self.total_votes >= self.min_vote_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mock_escalator(
        pattern: EscalationPattern,
        profile: ProgressionProfile,
        votes: usize,
        min_votes: usize,
    ) -> ThresholdEscalator {
        ThresholdEscalator {
            base_threshold: 0.5,
            ceiling: 0.9,
            pattern,
            emergency_override: false,
            profile,
            total_votes: votes,
            min_vote_count: min_votes,
        }
    }

    // #[test]
    // fn test_linear_with_profile() {
    //     let esc = mock_escalator(
    //         EscalationPattern::Linear(0.01),
    //         ProgressionProfile::Aggressive,
    //         5,
    //         3,
    //     );
    //     let now = Utc::now();
    //     let earlier = now - chrono::Duration::seconds(30);
    //     let expected = 0.5 + 0.01 * 2.0 * 30.0; // Aggressive doubles time
    //     let actual = esc.threshold_with_profile(now, earlier);
    //     assert!((actual - expected).abs() < 0.01);
    // }

    #[test]
    fn test_exponential_with_profile() {
        let esc = mock_escalator(
            EscalationPattern::Exponential(0.01),
            ProgressionProfile::Conservative,
            1,
            1,
        );
        let threshold =
            esc.threshold_with_profile(Utc::now(), Utc::now() - chrono::Duration::seconds(30));
        assert!(threshold > 0.5);
    }

    #[test]
    fn test_sigmoid_with_profile() {
        let esc = mock_escalator(
            EscalationPattern::Sigmoid(0.1, 30.0),
            ProgressionProfile::Adaptive,
            1,
            1,
        );
        let threshold =
            esc.threshold_with_profile(Utc::now(), Utc::now() - chrono::Duration::seconds(60));
        assert!(threshold > 0.5);
    }

    #[test]
    fn test_emergency_override() {
        let mut esc = mock_escalator(
            EscalationPattern::Linear(0.01),
            ProgressionProfile::Conservative,
            10,
            3,
        );
        esc.emergency_override = true;
        assert_eq!(
            esc.threshold_with_profile(Utc::now(), Utc::now()),
            esc.ceiling
        );
    }

    #[test]
    fn test_threshold_met_logic_passes() {
        let esc = mock_escalator(
            EscalationPattern::Linear(0.01),
            ProgressionProfile::Adaptive,
            5,
            3,
        );
        assert!(esc.is_threshold_met(0.75, 0.7)); // weight ≥ threshold AND total_votes ≥ min_vote_count
    }

    #[test]
    fn test_threshold_met_logic_fails_weight() {
        let esc = mock_escalator(
            EscalationPattern::Linear(0.01),
            ProgressionProfile::Adaptive,
            5,
            3,
        );
        assert!(!esc.is_threshold_met(0.65, 0.7)); // weight < threshold
    }

    #[test]
    fn test_threshold_met_logic_fails_votes() {
        let esc = mock_escalator(
            EscalationPattern::Linear(0.01),
            ProgressionProfile::Adaptive,
            2,
            3,
        );
        assert!(!esc.is_threshold_met(0.75, 0.7)); // total_votes < min_vote_count
    }
}
