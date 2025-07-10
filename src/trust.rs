use std::collections::HashMap;

pub struct TrustEngine {
    trusted_validators: HashMap<String, f64>, // validator_id -> bonus multiplier
}

impl TrustEngine {
    pub fn new() -> Self {
        let mut trusted = HashMap::new();
        trusted.insert("validator_001".to_string(), 1.2); // +20%
        trusted.insert("validator_002".to_string(), 1.1); // +10%
        Self {
            trusted_validators: trusted,
        }
    }

    pub fn get_bonus(&self, validator_id: &str) -> f64 {
        self.trusted_validators.get(validator_id).cloned().unwrap_or(1.0)
    }
}
