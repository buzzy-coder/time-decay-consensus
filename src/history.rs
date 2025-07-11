use chrono::{DateTime, Utc};

/// Stores the result of an individual vote
#[derive(Debug, Clone)]
pub struct VoteRecord {
    pub vote_id: String,
    pub weight: f64,
    pub threshold: f64,
    pub passed: bool,
    pub timestamp: DateTime<Utc>,
}

/// Collects vote history and provides analysis
#[derive(Default)]
pub struct HistoryAnalyzer {
    pub records: Vec<VoteRecord>,
}

impl HistoryAnalyzer {
    /// Add a vote record after evaluating a vote
    pub fn record_vote(&mut self, record: VoteRecord) {
        self.records.push(record);
    }

    /// Average margin of success or failure
    pub fn average_margin(&self) -> f64 {
        let total_margin: f64 = self
            .records
            .iter()
            .map(|r| r.weight - r.threshold)
            .sum();

        total_margin / self.records.len().max(1) as f64
    }

    /// Suggest adjusted base threshold based on past results
    pub fn suggested_base_threshold(&self) -> f64 {
        let avg_margin = self.average_margin();

        if avg_margin < 0.0 {
            0.55 // Raise slightly if votes usually fail
        } else {
            0.50 // Maintain or lower if votes usually pass
        }
    }

    /// Display vote history
    pub fn print_history(&self) {
        println!("\n📊 Historical Vote Log:");
        for r in &self.records {
            println!(
                "- {}: weight={:.4}, threshold={:.4}, passed={}, at {}",
                r.vote_id,
                r.weight,
                r.threshold,
                r.passed,
                r.timestamp
            );
        }
    }
}
