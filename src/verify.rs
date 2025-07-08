// src/verify.rs

use chrono::{DateTime, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;

use crate::vote::SignedVote;

impl SignedVote {
    /// Generate a new signed vote
    pub fn new(
        voter_id: String,
        proposal_id: String,
        original_weight: f64,
        decay_model: crate::vote::DecayType,
        signing_key: &SigningKey,
    ) -> Self {
        let timestamp = Utc::now();
        let message = format!("{}:{}:{}", voter_id, proposal_id, timestamp);
        let signature = signing_key.sign(message.as_bytes());
        let public_key = signing_key.verifying_key();

        Self {
            voter_id,
            proposal_id,
            timestamp,
            original_weight,
            decay_model,
            signature,
            public_key,
        }
    }

    /// Verify the vote signature and timestamp
    pub fn verify(&self, max_age_secs: i64) -> bool {
        let message = format!("{}:{}:{}", self.voter_id, self.proposal_id, self.timestamp);
        let now = Utc::now();
        let age_secs = (now - self.timestamp).num_seconds();

        // Reject if timestamp is too old or in the future (±5 seconds allowed)
        if age_secs < -5 || age_secs > max_age_secs {
            eprintln!("Vote timestamp out of bounds");
            return false;
        }

        self.public_key.verify(message.as_bytes(), &self.signature).is_ok()
    }
}

/// Utility function to generate a validator keypair
pub fn generate_keypair() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}
