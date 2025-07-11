mod decay;
mod threshold;
mod verify;
mod vote;
mod window;
mod weight_engine;
mod trust;
mod history;
mod simulation; 
mod blockchain;

use threshold::ThresholdEscalator;
use vote::{SignedVote, DecayType, ProposalType};
use weight_engine::WeightEngine;
use trust::TrustEngine;
use history::{VoteRecord, HistoryAnalyzer};
use simulation::run_simulation;
use blockchain::Blockchain;
use chrono::Utc;

#[allow(unused_imports)]
use decay::DecayModel;
#[allow(unused_imports)]
use verify::VerificationError;
#[allow(unused_imports)]
use blockchain::Block;



fn main() {
    // Run simulation directly
    run_simulation();

    // Blockchain demonstration
    println!("
--- Blockchain Demonstration ---");
    let mut blockchain = Blockchain::new();
    blockchain.add_block("Transaction 1 Data".to_string());
    blockchain.add_block("Transaction 2 Data".to_string());
    blockchain.add_block("Transaction 3 Data".to_string());

    println!("Blockchain is valid: {}", blockchain.is_valid());

    for block in &blockchain.blocks {
        println!("Block {}: {}", block.id, block.hash);
    }
    println!("--------------------------------");

    #[warn(unused_variables)]
    // Step 1: Generate a keypair (validator)
    let signing_key = SignedVote::generate_keypair();
    let _verify_key = signing_key.verifying_key();

    // Step 2: Hardcoded input
    let voter_id = "hardcoded_voter".to_string();
    let proposal_id = "hardcoded_proposal".to_string();
    let original_weight: f64 = 0.75;
    let decay_model = DecayType::Linear;
    let proposal_type = ProposalType::Normal;

    // Step 3: Create vote
    let now = Utc::now();
    let vote = SignedVote::new(
        voter_id.clone(),
        proposal_id,
        original_weight,
        now,
        decay_model.clone(),
        &signing_key,
    );

    // Step 4: Verify vote
    match vote.verify(300) {
        Ok(_) => println!("✅ Signature verification successful."),
        Err(e) => {
            println!("❌ Verification failed: {:?}", e);
            return;
        }
    }

    // Step 5: Compute decayed & trust-adjusted weight
    let mut weight_engine = WeightEngine::new();
    let trust_engine = TrustEngine::new();
    let weight = weight_engine.calculate_weight(&vote, now, Some(&trust_engine));
    println!("🧮 Final vote weight after decay & trust bonus: {:.4}", weight);

    // Step 6: Threshold logic
    let mut threshold_engine = ThresholdEscalator::for_proposal_type(proposal_type.clone());
    threshold_engine.total_votes = 3; // Simulated count
    let current_threshold = threshold_engine.threshold_with_profile(now, vote.timestamp);
    println!("🔢 Required threshold at this time: {:.2}%", current_threshold * 100.0);

    let passed = threshold_engine.is_threshold_met(weight, current_threshold);
    if passed {
        println!("✅ Vote passes threshold and minimum vote count");
    } else {
        println!("❌ Vote rejected: weight or participation too low");
    }

    // Step 7: Historical record
    let mut history = HistoryAnalyzer::default();
    let record = VoteRecord {
        vote_id: vote.voter_id.clone(),
        weight,
        threshold: current_threshold,
        passed,
        timestamp: now,
    };
    history.record_vote(record);

    // Logs
    println!("
📊 Historical Vote Log:");
    history.print_history();

    println!("
📜 Weight History Log:");
    for record in weight_engine.get_history() {
        println!(
            "- {} -> {:.4} at {:?}",
            record.vote_id, record.weight, record.timestamp
        );
    }

}


#[cfg(test)]
mod tests {
    use chrono::{Utc, Duration};
    use ed25519_dalek::SigningKey;

    use crate::trust::TrustEngine;
    use crate::vote::{DecayType, SignedVote};
    use crate::weight_engine::WeightEngine;
    use crate::threshold::{ThresholdEscalator, ProgressionProfile};
    use crate::history::{HistoryAnalyzer, VoteRecord};
    use crate::vote::ProposalType;

    #[test]
    fn test_signed_vote_verification() {
        let signing_key = SignedVote::generate_keypair();
        let now = Utc::now();

        let vote = SignedVote::new(
            "voter_123".to_string(),
            "proposal_abc".to_string(),
            1.0,
            now,
            DecayType::Linear,
            &signing_key,
        );

        assert!(vote.verify(300).is_ok(), "Signature should verify within allowed time");

        // simulate a future timestamp — should fail
        let bad_vote = SignedVote::new(
            "voter_123".to_string(),
            "proposal_abc".to_string(),
            1.0,
            now + Duration::seconds(10),
            DecayType::Linear,
            &signing_key,
        );
        assert!(bad_vote.verify(300).is_err(), "Future timestamp should fail");
    }

    #[test]
    fn test_weight_engine_with_trust() {
        let signing_key = SignedVote::generate_keypair();
        let now = Utc::now();

        let vote = SignedVote::new(
            "validator_001".to_string(),
            "proposal_abc".to_string(),
            2.0,
            now - Duration::seconds(60),
            DecayType::Exponential,
            &signing_key,
        );

        let mut weight_engine = WeightEngine::new();
        let trust_engine = TrustEngine::new();

        let weight = weight_engine.calculate_weight(&vote, now, Some(&trust_engine));
        assert!(weight > 0.0, "Weight should be positive");
        assert!(weight > 2.0 * 0.5, "Weight with trust bonus should be reasonable");
    }

    #[test]
    fn test_threshold_escalator() {
        let mut escalator = ThresholdEscalator::for_proposal_type(ProposalType::Normal);
        escalator.total_votes = 10;
        escalator.min_vote_count = 5;

        let now = Utc::now();
        let threshold = escalator.threshold_with_profile(now, now);
        let passed = escalator.is_threshold_met(0.6, threshold);

        assert!(threshold >= 0.0 && threshold <= 1.0, "Threshold should be between 0 and 1");
        assert!(
            passed == (0.6 >= threshold),
            "Pass condition should match weight vs. threshold"
        );
    }

    #[test]
    fn test_history_analyzer() {
        let mut history = HistoryAnalyzer::default();
        let now = Utc::now();

        let record = VoteRecord {
            vote_id: "voter_1".to_string(),
            weight: 1.0,
            threshold: 0.5,
            passed: true,
            timestamp: now,
        };

        history.record_vote(record.clone());
        assert_eq!(history.records.len(), 1, "History should have one record");
        assert_eq!(history.records[0].vote_id, "voter_1", "Recorded voter ID should match");
    }
}