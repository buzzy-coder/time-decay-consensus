mod vote;
mod decay;
mod verify;

use chrono::{DateTime, Utc};
use vote::{Vote, DecayType};
use decay::{ExponentialDecay, LinearDecay, SteppedDecay, decay_model};

fn get_vote_weight(vote: &Vote, current_time: DateTime<Utc>) -> f64 {
    let age = (current_time - vote.timestamp).num_seconds() as u64;

    match vote.decay_model {
        DecayType::Exponential => {
            let model = ExponentialDecay { rate: 0.005 };
            model.compute_weight(vote.original_weight, age as f64)
        }
        DecayType::Linear => {
            let model = LinearDecay { rate: 0.001 };
            model.compute_weight(vote.original_weight, age as f64)
        }
        DecayType::Stepped => {
            let model = SteppedDecay {
                decay_steps: vec![(60.0, 0.8), (180.0, 0.5), (300.0, 0.2)],
            };
            model.compute_weight(vote.original_weight, age as f64)
        }
    }
}

fn main() {
    let vote = Vote {
        voter_id: "alice".into(),
        proposal_id: "prop1".into(),
        timestamp: Utc::now() - chrono::Duration::seconds(200), // 200 seconds ago
        original_weight: 1.0,
        decay_model: DecayType::Stepped,
    };

    let weight = get_vote_weight(&vote, Utc::now());
    println!("Effective vote weight: {:.4}", weight);
}

