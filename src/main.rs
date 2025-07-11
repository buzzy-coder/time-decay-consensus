use chrono::{DateTime, Utc};
use std::io;

mod decay;
mod threshold;
mod trust;
mod verify;
mod vote;
mod weight_engine;

use threshold::{EscalationPattern, ProgressionProfile, ThresholdEscalator};
use trust::TrustEngine;
use vote::{DecayType, ProposalType, SignedVote};
use weight_engine::WeightEngine;

fn main() {
    // Step 1: Generate a keypair (in real use case, would be per-validator)
    let signing_key = SignedVote::generate_keypair();
    let verify_key = signing_key.verifying_key();

    // Step 2: Collect dynamic input from user
    let mut input = String::new();
    println!("Enter your voter ID:");
    io::stdin().read_line(&mut input).unwrap();
    let voter_id = input.trim().to_string();

    input.clear();
    println!("Enter proposal ID:");
    io::stdin().read_line(&mut input).unwrap();
    let proposal_id = input.trim().to_string();

    input.clear();
    println!("Enter original vote weight (e.g., 1.0):");
    io::stdin().read_line(&mut input).unwrap();
    let original_weight: f64 = input.trim().parse().unwrap_or(1.0);

    input.clear();
    println!("Enter decay model (linear / exponential / stepped):");
    io::stdin().read_line(&mut input).unwrap();
    let decay_model = match input.trim().to_lowercase().as_str() {
        "linear" => DecayType::Linear,
        "stepped" => DecayType::Stepped,
        _ => DecayType::Exponential,
    };

    input.clear();
    println!("Proposal type (normal / critical):");
    io::stdin().read_line(&mut input).unwrap();
    let proposal_type = match input.trim().to_lowercase().as_str() {
        "critical" => ProposalType::Critical,
        _ => ProposalType::Normal,
    };

    let now = Utc::now();
    let vote = SignedVote::new(
        voter_id.clone(),
        proposal_id,
        original_weight,
        now, // 👈 Pass this to maintain consistency
        decay_model,
        &signing_key,
    );

    // Step 3: Verify vote signature
    if vote.verify(300).is_ok() {
        println!("✅ Signature verification successful.");
    } else {
        println!("❌ Invalid vote signature. Exiting.");
        return;
    }

    // Step 4: Compute weight with trust and decay
    let mut weight_engine = WeightEngine::new();
    let trust_engine = TrustEngine::new();
    let weight = weight_engine.calculate_weight(&vote, now, Some(&trust_engine));

    println!(
        "🧮 Final vote weight after decay & trust bonus: {:.4}",
        weight
    );

    // Step 5: Threshold evaluation based on proposal type
    let threshold_engine = match proposal_type {
        ProposalType::Critical => ThresholdEscalator {
            base_threshold: 0.75,
            ceiling: 0.95,
            pattern: EscalationPattern::Linear(0.015),
            emergency_override: false,
            profile: ProgressionProfile::Aggressive,
            total_votes: 3,
        },
        ProposalType::Normal => ThresholdEscalator {
            base_threshold: 0.51,
            ceiling: 0.9,
            pattern: EscalationPattern::Linear(0.01),
            emergency_override: false,
            profile: ProgressionProfile::Conservative,
            total_votes: 3,
        },
    };

    let current_threshold = threshold_engine.threshold_with_profile(now, vote.timestamp);
    println!(
        "🔢 Required threshold at this time: {:.2}%",
        current_threshold * 100.0
    );

    if threshold_engine.is_threshold_met(weight, current_threshold) {
        println!("✅ Vote passes threshold and minimum vote count");
    } else {
        println!("❌ Vote rejected: weight or participation too low");
    }

    // Optional: Print weight history log
    println!("\n📜 Weight History Log:");
    for record in weight_engine.get_history() {
        println!(
            "- {} -> {:.4} at {:?}",
            record.vote_id, record.weight, record.timestamp
        );
    }
}
