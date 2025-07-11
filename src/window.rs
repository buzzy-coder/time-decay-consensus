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


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Duration};

    #[test]
    fn test_window_creation() {
        let now = Utc::now();
        let vw_short = VotingWindow::new(now, WindowType::Short, 10);
        assert_eq!(vw_short.duration_secs, 300);

        let vw_medium = VotingWindow::new(now, WindowType::Medium, 10);
        assert_eq!(vw_medium.duration_secs, 1800);

        let vw_long = VotingWindow::new(now, WindowType::Long, 10);
        assert_eq!(vw_long.duration_secs, 7200);

        let vw_custom = VotingWindow::new(now, WindowType::Custom(42), 10);
        assert_eq!(vw_custom.duration_secs, 42);
    }

    #[test]
    fn test_is_open() {
        let now = Utc::now();
        let vw = VotingWindow::new(now, WindowType::Short, 10);

        // Immediately after start, should be open
        assert!(vw.is_open(now));

        // After window + grace, should be closed
        let after_deadline =
            now + Duration::seconds((vw.duration_secs + vw.grace_secs + 1) as i64);
        assert!(!vw.is_open(after_deadline));
    }

    #[test]
    fn test_time_left() {
        let now = Utc::now();
        let vw = VotingWindow::new(now, WindowType::Short, 10);

        // At start, full time left
        assert_eq!(vw.time_left(now), vw.duration_secs as i64);

        // Halfway
        let halfway = now + Duration::seconds((vw.duration_secs / 2) as i64);
        assert!((vw.time_left(halfway) - (vw.duration_secs as i64 / 2)).abs() <= 1);

        // After deadline
        let after = now + Duration::seconds((vw.duration_secs + 1) as i64);
        assert!(vw.time_left(after) < 0);
    }

    #[test]
    fn test_should_extend() {
        let now = Utc::now();
        let mut vw = VotingWindow::new(now, WindowType::Short, 10);

        // Move close to end
        let near_end =
            now + Duration::seconds((vw.duration_secs - 15) as i64);
        let threshold = 100.0;

        // weight below 90% of threshold: should not extend
        assert!(!vw.should_extend(near_end, 80.0, threshold));

        // weight above 90% of threshold and within 20s: should extend
        assert!(vw.should_extend(near_end, 95.0, threshold));
    }

    #[test]
    fn test_extend() {
        let now = Utc::now();
        let mut vw = VotingWindow::new(now, WindowType::Short, 10);

        let original_duration = vw.duration_secs;
        vw.extend(60);
        assert_eq!(vw.duration_secs, original_duration + 60);
    }
}