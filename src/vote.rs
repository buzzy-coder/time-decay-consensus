use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, VerifyingKey};

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
