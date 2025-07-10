use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, Copy)]
pub enum WindowType {
    Short,       // 5 minutes
    Medium,      // 30 minutes
    Long,        // 2 hours
    Custom(u64), // in seconds
}

pub struct VotingWindow {
    pub start_time: DateTime<Utc>,
    pub duration_secs: u64,
    pub grace_secs: u64,
}

impl VotingWindow {
    pub fn new(start_time: DateTime<Utc>, window_type: WindowType, grace_secs: u64) -> Self {
        let duration_secs = match window_type {
            WindowType::Short => 300,
            WindowType::Medium => 1800,
            WindowType::Long => 7200,
            WindowType::Custom(secs) => secs,
        };
        VotingWindow {
            start_time,
            duration_secs,
            grace_secs,
        }
    }

    pub fn is_open(&self, now: DateTime<Utc>) -> bool {
        let deadline =
            self.start_time + Duration::seconds((self.duration_secs + self.grace_secs) as i64);
        now <= deadline
    }

    pub fn time_left(&self, now: DateTime<Utc>) -> i64 {
        let deadline = self.start_time + Duration::seconds(self.duration_secs as i64);
        (deadline - now).num_seconds()
    }

    pub fn should_extend(
        &self,
        now: DateTime<Utc>,
        current_weight: f64,
        current_threshold: f64,
    ) -> bool {
        let time_left = self.time_left(now);
        let close_enough = current_weight >= 0.9 * current_threshold;
        time_left <= 20 && close_enough
    }

    pub fn extend(&mut self, extra_secs: u64) {
        self.duration_secs += extra_secs;
        println!("⏳ Voting window extended by {} seconds!", extra_secs);
    }
}
