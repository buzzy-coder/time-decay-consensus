use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, VerifyingKey};


pub enum DecayType {
    Linear,
    Exponential,
    Stepped,
}

pub struct SignedVote {
    pub voter_id: String,
    pub proposal_id: String,
    pub timestamp: DateTime<Utc>,
    pub original_weight: f64,
    pub decay_model: DecayType,
    pub signature: Signature,
    pub public_key: VerifyingKey,    
}
