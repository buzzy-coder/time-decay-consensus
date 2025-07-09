mod vote;
mod decay;
mod verify;
mod threshold;

use chrono::{DateTime, Utc};
use vote::{SignedVote, DecayType};
use decay::{ExponentialDecay, LinearDecay, SteppedDecay, DecayModel};
use threshold::{ThresholdEscalator, EscalationPattern};

fn get_vote_weight(vote: &SignedVote, current_time: DateTime<Utc>) -> f64 {
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
    // Step 1: Generate validator keypair
    let keypair = SignedVote::generate_keypair();

    // Step 2: Create a new vote
    let vote = SignedVote::new(
        "voter_001".into(),
        "proposal_alpha".into(),
        1.0,
        DecayType::Linear,
        &keypair,
    );

    // Step 3: Verify the vote
    let is_valid = vote.verify(300); // max age = 300s
    println!("🔐 Signature valid: {:?}", is_valid);

    // Step 4: Calculate decayed weight
    let current_time = Utc::now();
    let weight = get_vote_weight(&vote, current_time);
    println!("⚖️ Decayed weight: {:.4}", weight);

    // Step 5: Threshold escalation simulation
    let threshold = ThresholdEscalator {
        base_threshold: 0.51,
        ceiling: 0.9,
        pattern: EscalationPattern::Linear(0.01),
        emergency_override: false,
    };
    let current_threshold = threshold.current_threshold(90); // simulate 90s passed
    println!("📈 Current threshold (after 90s): {:.2}%", current_threshold * 100.0);

    // Step 6: Decision
    if weight >= current_threshold {
        println!("✅ Vote passes threshold");
    } else {
        println!("❌ Vote below threshold");
    }
}

