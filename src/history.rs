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


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc};

    fn sample_vote(vote_id: &str, weight: f64, threshold: f64, passed: bool) -> VoteRecord {
        VoteRecord {
            vote_id: vote_id.to_string(),
            weight,
            threshold,
            passed,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_record_and_history() {
        let mut analyzer = HistoryAnalyzer::default();

        let vote1 = sample_vote("vote1", 0.6, 0.5, true);
        analyzer.record_vote(vote1.clone());

        assert_eq!(analyzer.records.len(), 1);
        assert_eq!(analyzer.records[0].vote_id, "vote1");
        assert_eq!(analyzer.records[0].passed, true);

        analyzer.print_history(); // Should not panic
    }

    #[test]
    fn test_average_margin() {
        let mut analyzer = HistoryAnalyzer::default();

        analyzer.record_vote(sample_vote("v1", 0.6, 0.5, true));  // margin +0.1
        analyzer.record_vote(sample_vote("v2", 0.4, 0.5, false)); // margin -0.1

        let avg_margin = analyzer.average_margin();
        assert!((avg_margin - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_suggested_base_threshold() {
        let mut analyzer = HistoryAnalyzer::default();

        // Mostly passes
        analyzer.record_vote(sample_vote("v1", 0.7, 0.5, true));
        analyzer.record_vote(sample_vote("v2", 0.6, 0.5, true));
        assert_eq!(analyzer.suggested_base_threshold(), 0.50);

        // Mostly fails
        let mut failing_analyzer = HistoryAnalyzer::default();
        failing_analyzer.record_vote(sample_vote("v3", 0.4, 0.5, false));
        failing_analyzer.record_vote(sample_vote("v4", 0.3, 0.5, false));
        assert_eq!(failing_analyzer.suggested_base_threshold(), 0.55);
    }

    #[test]
    fn test_empty_history() {
        let analyzer = HistoryAnalyzer::default();

        // Should handle empty gracefully
        assert_eq!(analyzer.average_margin(), 0.0);
        assert_eq!(analyzer.suggested_base_threshold(), 0.50);
        analyzer.print_history(); // Should not panic
    }
}