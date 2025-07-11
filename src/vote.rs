use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};


#[derive(Debug, Clone, Copy)]
pub enum DecayType {
    Linear,
    Exponential,
    Stepped,
}

#[derive(Clone)]
pub enum ProposalType {
    Normal,
    Critical,
}

#[derive(Debug, Clone)]
pub struct SignedVote {
    pub voter_id: String,
    pub proposal_id: String,
    pub timestamp: DateTime<Utc>,
    pub original_weight: f64,
    pub decay_model: DecayType,
    pub signature: Signature,
    pub public_key: VerifyingKey,    
}

pub fn sign_vote(voter_id: String, signing_key: &SigningKey, timestamp: DateTime<Utc>) -> Signature {
    let message = format!("{}{}", voter_id, timestamp.to_rfc3339());
    signing_key.sign(message.as_bytes())
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ed25519_dalek::{Verifier};

    #[test]
    fn test_sign_vote_validity() {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();

        let voter_id = "voter1".to_string();
        let timestamp = Utc::now();
        let sig = sign_vote(voter_id.clone(), &signing_key, timestamp);

        let message = format!("{}{}", voter_id, timestamp.to_rfc3339());
        assert!(verifying_key.verify(message.as_bytes(), &sig).is_ok());
    }

    #[test]
    fn test_signed_vote_fields() {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();

        let voter_id = "voter123".to_string();
        let proposal_id = "proposalABC".to_string();
        let timestamp = Utc::now();
        let weight = 1.0;
        let decay = DecayType::Linear;

        let message = format!("{}{}", voter_id, timestamp.to_rfc3339());
        let signature = signing_key.sign(message.as_bytes());

        let vote = SignedVote {
            voter_id: voter_id.clone(),
            proposal_id: proposal_id.clone(),
            timestamp,
            original_weight: weight,
            decay_model: decay,
            signature: signature.clone(),
            public_key: verifying_key,
        };

        // check fields
        assert_eq!(vote.voter_id, voter_id);
        assert_eq!(vote.proposal_id, proposal_id);
        assert_eq!(vote.original_weight, weight);

        // verify signature
        assert!(vote.public_key.verify(message.as_bytes(), &signature).is_ok());
    }

    #[test]
    fn test_decay_and_proposal_types() {
        let _linear = DecayType::Linear;
        let _exp = DecayType::Exponential;
        let _stepped = DecayType::Stepped;

        let _normal = ProposalType::Normal;
        let _critical = ProposalType::Critical;
    }
}