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
