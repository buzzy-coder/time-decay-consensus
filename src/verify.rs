// src/verify.rs

use chrono::{DateTime, Utc};
use ed25519_dalek::{SECRET_KEY_LENGTH, Signer, SigningKey, Verifier};
use rand::RngCore;
use rand::rngs::OsRng;
use thiserror::Error;

use crate::vote::SignedVote;

#[derive(Error, Debug, PartialEq)]
pub enum VerificationError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Timestamp is too old")]
    TimestampExpired,
    #[error("Timestamp is in the future")]
    TimestampInFuture,
}

impl SignedVote {
    /// Generate a new signed vote
pub fn new(
    voter_id: String,
    proposal_id: String,
    original_weight: f64,
    timestamp: DateTime<Utc>, // ✅ take from caller
    decay_model: crate::vote::DecayType,
    signing_key: &SigningKey,
) -> Self {
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
    pub fn verify(&self, max_age_secs: i64) -> Result<(), VerificationError> {
        let message = format!("{}:{}:{}", self.voter_id, self.proposal_id, self.timestamp);
        let now = Utc::now();
        let age_secs = (now - self.timestamp).num_seconds();

        // Reject if timestamp is too old or in the future (±5 seconds allowed)
        if age_secs < -5 {
            return Err(VerificationError::TimestampInFuture);
        }
        if age_secs > max_age_secs {
            return Err(VerificationError::TimestampExpired);
        }

        self.public_key
            .verify(message.as_bytes(), &self.signature)
            .map_err(|_| VerificationError::InvalidSignature)
    }

    /// Utility function to generate a validator keypair
    pub fn generate_keypair() -> SigningKey {
        let mut rng = OsRng;
        let mut secret = [0u8; SECRET_KEY_LENGTH];
        rng.fill_bytes(&mut secret);
        SigningKey::from_bytes(&secret)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::vote::{SignedVote, DecayType};
    use chrono::{Utc, Duration};

    fn mock_signed_vote(offset_secs: i64) -> SignedVote {
        let signing_key = SignedVote::generate_keypair();
        let timestamp = Utc::now() + Duration::seconds(offset_secs);
        SignedVote::new(
            "voter1".to_string(),
            "proposal1".to_string(),
            1.0,
            timestamp,
            DecayType::Linear, // use tuple variant syntax if Linear is defined as Linear(f64)
            &signing_key,
        )
    }

    #[test]
    fn test_valid_vote() {
        let vote = mock_signed_vote(0);
        assert_eq!(vote.verify(10), Ok(()));
    }

    #[test]
    fn test_vote_too_old() {
        let vote = mock_signed_vote(-20);
        let result = vote.verify(10);
        assert_eq!(result, Err(VerificationError::TimestampExpired));
    }

    #[test]
    fn test_vote_in_future() {
        let vote = mock_signed_vote(10);
        let result = vote.verify(5);
        assert_eq!(result, Err(VerificationError::TimestampInFuture));
    }

    #[test]
    fn test_invalid_signature() {
        let mut vote = mock_signed_vote(0);
        // Corrupt the signature bytes
        vote.signature = ed25519_dalek::Signature::try_from([0u8; 64])
    .expect("Failed to create dummy signature");
        let result = vote.verify(10);
        assert_eq!(result, Err(VerificationError::InvalidSignature));
    }
} 