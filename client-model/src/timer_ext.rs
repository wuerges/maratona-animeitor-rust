use data::TimerData;

/// Client-side semantics of the shared [`TimerData`] wire type.
pub trait TimerDataExt {
    /// Whether the scoreboard is currently frozen.
    fn is_frozen(&self) -> bool;

    /// A timer state that is already frozen, used before the first timer
    /// update arrives.
    fn fake() -> TimerData;
}

impl TimerDataExt for TimerData {
    fn is_frozen(&self) -> bool {
        self.current_time >= self.score_freeze_time * 60
    }

    fn fake() -> TimerData {
        TimerData::new(86399, 86399 + 1)
    }
}
