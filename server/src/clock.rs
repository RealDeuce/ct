//! Mapping from monotonic process time to the authoritative game-second target.

use std::time::{Duration, Instant};

pub const CLOCK_FORMAT_VERSION: u64 = 1;
pub const GAME_SECONDS_PER_RATE_PERIOD: u64 = 28;
pub const RATE_PERIOD: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct LiveClock {
    anchor_instant: Instant,
    anchor_game_second: u64,
}

impl LiveClock {
    pub fn new(anchor_game_second: u64, anchor_instant: Instant) -> Self {
        Self {
            anchor_instant,
            anchor_game_second,
        }
    }

    pub fn now(anchor_game_second: u64) -> Self {
        Self::new(anchor_game_second, Instant::now())
    }

    pub fn target_second(&self, now: Instant) -> u64 {
        let elapsed = now.saturating_duration_since(self.anchor_instant);
        let scaled = elapsed
            .as_nanos()
            .saturating_mul(u128::from(GAME_SECONDS_PER_RATE_PERIOD))
            / RATE_PERIOD.as_nanos();
        self.anchor_game_second
            .saturating_add(u64::try_from(scaled).unwrap_or(u64::MAX))
    }

    pub fn reanchor(&mut self, game_second: u64, now: Instant) {
        self.anchor_game_second = game_second;
        self.anchor_instant = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_rate_is_exact_and_fractional_seconds_do_not_commit_early() {
        let start = Instant::now();
        let clock = LiveClock::new(100, start);
        assert_eq!(clock.target_second(start + Duration::from_millis(20)), 100);
        assert_eq!(clock.target_second(start + Duration::from_secs(1)), 128);
        assert_eq!(clock.target_second(start + Duration::from_secs(60)), 1_780);
    }

    #[test]
    fn reanchor_freezes_elapsed_downtime() {
        let start = Instant::now();
        let mut clock = LiveClock::new(40, start);
        assert_eq!(clock.target_second(start + Duration::from_secs(1)), 68);
        clock.reanchor(68, start + Duration::from_secs(600));
        assert_eq!(clock.target_second(start + Duration::from_secs(600)), 68);
    }
}
