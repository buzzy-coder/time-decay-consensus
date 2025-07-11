use crate::decay::{DecayModel, ExponentialDecay, LinearDecay, SteppedDecay};
use crate::trust::TrustEngine;
use crate::vote::{DecayType, SignedVote};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct VoteRecord {
    pub vote_id: String,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct WeightEngine {
    cache: HashMap<String, f64>,
    history: Vec<VoteRecord>,
}

impl WeightEngine {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn calculate_weight(
        &mut self,
        vote: &SignedVote,
        now: DateTime<Utc>,
        trust: Option<&TrustEngine>,
    ) -> f64 {
        if let Some(w) = self.cache.get(&vote.voter_id) {
            return *w;
        }

        let age = (now - vote.timestamp).num_seconds() as f64;

        let mut weight = match vote.decay_model {
            DecayType::Exponential => {
                ExponentialDecay { rate: 0.005 }.compute_weight(vote.original_weight, age)
            }
            DecayType::Linear => {
                LinearDecay { rate: 0.001 }.compute_weight(vote.original_weight, age)
            }
            DecayType::Stepped => SteppedDecay {
                decay_steps: vec![(60.0, 0.8), (180.0, 0.5), (300.0, 0.2)],
            }
            .compute_weight(vote.original_weight, age),
        };

        if let Some(trust_engine) = trust {
            let bonus = trust_engine.get_bonus(&vote.voter_id);
            weight *= bonus;
        }

        self.cache.insert(vote.voter_id.clone(), weight);
        self.history.push(VoteRecord {
            vote_id: vote.voter_id.clone(),
            weight,
            timestamp: now,
        });

        weight
    }

    #[allow(dead_code)]
    pub fn batch_calculate(
        &mut self,
        votes: &[SignedVote],
        now: DateTime<Utc>,
        trust: Option<&TrustEngine>,
    ) -> Vec<f64> {
        votes
            .iter()
            .map(|v| self.calculate_weight(v, now, trust))
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_weight_history(&self) -> &HashMap<String, f64> {
        &self.cache
    }

    pub fn get_history(&self) -> &Vec<VoteRecord> {
        &self.history
    }

    #[allow(dead_code)]
    /// Clears the cached weights and history log
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.history.clear();
        println!("🧹 WeightEngine cache and history cleared.");
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::trust::TrustEngine;
    use crate::vote::{SignedVote, DecayType};
    use ed25519_dalek::SigningKey;
    use signature::Signer;

    // Mocked decay models
    pub struct LinearDecay {
        pub rate: f64,
    }
    pub struct ExponentialDecay {
        pub rate: f64,
    }
    pub struct SteppedDecay {
        pub decay_steps: Vec<(f64, f64)>,
    }

    impl LinearDecay {
        pub fn compute_weight(&self, original: f64, age: f64) -> f64 {
            (original - self.rate * age).max(0.0)
        }
    }
    impl ExponentialDecay {
        pub fn compute_weight(&self, original: f64, age: f64) -> f64 {
            original * (-self.rate * age).exp()
        }
    }
    impl SteppedDecay {
        pub fn compute_weight(&self, original: f64, age: f64) -> f64 {
            for (threshold, factor) in &self.decay_steps {
                if age <= *threshold {
                    return original * *factor;
                }
            }
            0.0
        }
    }

    fn mock_signed_vote(decay: DecayType) -> SignedVote {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let voter_id = "validator_001".to_string();
        let proposal_id = "proposal_001".to_string();
        let timestamp = Utc::now() - chrono::Duration::seconds(120);
        let original_weight = 1.0;

        let msg = format!("{}:{}:{}", voter_id, proposal_id, timestamp);
        let signature = signing_key.sign(msg.as_bytes());

        SignedVote {
            voter_id,
            proposal_id,
            timestamp,
            original_weight,
            decay_model: decay,
            signature,
            public_key: signing_key.verifying_key(),
        }
    }

    #[test]
    fn test_calculate_weight_linear() {
        let mut engine = WeightEngine::new();
        let vote = mock_signed_vote(DecayType::Linear);
        let now = Utc::now();

        let weight = engine.calculate_weight(&vote, now, None);
        assert!(weight >= 0.0, "Weight should be non-negative");
        assert!(engine.cache.contains_key(&vote.voter_id));
        assert_eq!(engine.history.len(), 1);
    }

    #[test]
    fn test_calculate_weight_with_trust_bonus() {
        let mut engine = WeightEngine::new();
        let vote = mock_signed_vote(DecayType::Linear);
        let now = Utc::now();

        let trust = TrustEngine::new();

        let weight_with_trust = engine.calculate_weight(&vote, now, Some(&trust));
        assert!(weight_with_trust >= 1.0, "Trusted validator weight should increase");
    }

    #[test]
    fn test_batch_calculate() {
        let mut engine = WeightEngine::new();
        let now = Utc::now();

        let votes = vec![
            mock_signed_vote(DecayType::Linear),
            mock_signed_vote(DecayType::Exponential),
            mock_signed_vote(DecayType::Stepped),
        ];

        let weights = engine.batch_calculate(&votes, now, None);
        assert_eq!(weights.len(), votes.len());
        assert_eq!(engine.history.len(), votes.len());
    }

    #[test]
    fn test_clear_cache() {
        let mut engine = WeightEngine::new();
        let vote = mock_signed_vote(DecayType::Linear);
        let now = Utc::now();

        engine.calculate_weight(&vote, now, None);
        assert!(!engine.cache.is_empty());
        assert!(!engine.history.is_empty());

        engine.clear_cache();

        assert!(engine.cache.is_empty());
        assert!(engine.history.is_empty());
    }
}