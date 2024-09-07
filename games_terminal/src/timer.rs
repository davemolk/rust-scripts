use std::time::{Instant, Duration};

pub struct Timer {
    end_time: Instant,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Timer {
            end_time: Instant::now() + duration,
        }
    }
    pub fn has_expired(&self) -> bool {
        Instant::now() >= self.end_time
    }
}