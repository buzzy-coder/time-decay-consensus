use chrono::{Utc, Duration};
use crate::vote::{SignedVote, DecayType, ProposalType};
use crate::verify::{VerificationError};
use crate::threshold::{ThresholdEscalator, EscalationPattern, ProgressionProfile};
use crate::trust::TrustEngine;
use crate::weight_engine::WeightEngine;
use crate::history::{VoteRecord, HistoryAnalyzer};
use ed25519_dalek::{Signer};

pub fn run_simulation() {
    let now = Utc::now();
    let trust_engine = TrustEngine::new();
    let mut weight_engine = WeightEngine::new();
    let mut history = HistoryAnalyzer::default();

    let voters = vec!["alice", "bob", "carol", "dave", "eve"];
    let decay_models = vec![DecayType::Linear, DecayType::Exponential, DecayType::Stepped];
    let proposal_type = ProposalType::Critical;

    // Initialize threshold engine
    let mut threshold_engine = ThresholdEscalator::for_proposal_type(proposal_type.clone());
    threshold_engine.total_votes = voters.len();

    for (i, voter) in voters.iter().enumerate() {
        let keypair = SignedVote::generate_keypair();
        let decay = &decay_models[i % decay_models.len()];

        // Stagger timestamps: simulate votes at different times
        let timestamp = now - Duration::seconds((i * 30) as i64);

        let vote = SignedVote {
            voter_id: voter.to_string(),
            proposal_id: "proposal_sim".to_string(),
            timestamp,
            original_weight: 1.0,
            decay_model: decay.clone(),
            signature: keypair.sign(format!("{}:{}:{}", voter, "proposal_sim", timestamp).as_bytes()),
            public_key: keypair.verifying_key(),
        };

        match vote.verify(300) {
            Ok(_) => {
                let weight = weight_engine.calculate_weight(&vote, now, Some(&trust_engine));
                let current_threshold = threshold_engine.threshold_with_profile(now, vote.timestamp);
                let passed = threshold_engine.is_threshold_met(weight, current_threshold);

                let record = VoteRecord {
                    vote_id: vote.voter_id.clone(),
                    weight,
                    threshold: current_threshold,
                    passed,
                    timestamp: now,
                };
                history.record_vote(record);

                println!(
                    "✅ {}: weight={:.4}, threshold={:.2}, passed={}",
                    vote.voter_id, weight, current_threshold * 100.0, passed
                );
            }
            Err(e) => println!("❌ {}: verification failed ({})", voter, e),
        }
    }

    println!("\n📊 Simulation Results (History Log):");
    history.print_history();
}
