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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trusted_validator_bonus() {
        let engine = TrustEngine::new();

        assert_eq!(engine.get_bonus("validator_001"), 1.2);
        assert_eq!(engine.get_bonus("validator_002"), 1.1);
    }

    #[test]
    fn test_untrusted_validator_bonus() {
        let engine = TrustEngine::new();

        assert_eq!(engine.get_bonus("unknown_validator"), 1.0);
        assert_eq!(engine.get_bonus(""), 1.0);
    }

    #[test]
    fn test_case_sensitivity() {
        let engine = TrustEngine::new();

        // Should be case-sensitive by default
        assert_eq!(engine.get_bonus("VALIDATOR_001"), 1.0);
        assert_eq!(engine.get_bonus("Validator_001"), 1.0);
    }
}