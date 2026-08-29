use data::TimerData;

/// Client-side semantics of the shared [`TimerData`] wire type.
pub trait TimerDataExt {
    /// Whether the scoreboard is currently frozen.
    fn is_frozen(&self) -> bool;

    /// A negative placeholder used before the first timer update arrives:
    /// keeps the countdown gate closed so the scoreboard never mounts while
    /// the contest has not started.
    fn fake() -> TimerData;
}

impl TimerDataExt for TimerData {
    fn is_frozen(&self) -> bool {
        self.current_time >= self.score_freeze_time * 60
    }

    fn fake() -> TimerData {
        TimerData::new(-1, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_is_negative_so_the_board_stays_gated() {
        let fake = TimerData::fake();
        assert!(fake.current_time < 0);
        assert!(!fake.is_frozen());
    }
}
