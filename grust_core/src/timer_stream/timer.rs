use std::time::{Duration, Instant};

/// Timing trait.
pub trait Timing {
    fn get_duration(&self) -> Duration;
    fn do_reset(&self) -> bool;
}

/// Timer.
///
/// A timer has a `kind`, which is its identifier (period_component_c, timeout_event_e, etc).
/// It also has a deadline, to which it should tick.
#[derive(Debug, PartialEq)]
pub struct Timer<T> {
    deadline: Instant,
    kind: T,
}
impl<T> Timer<T>
where
    T: Timing,
{
    /// Initiate a new timer.
    pub fn init(kind: T, now: Instant) -> Timer<T> {
        Timer {
            deadline: now + kind.get_duration(),
            kind,
        }
    }
}
impl<T> Timer<T> {
    /// Get timer's kind.
    pub fn get_kind(&self) -> &T {
        &self.kind
    }
    /// Get timer's deadline.
    pub fn get_deadline(&self) -> &Instant {
        &self.deadline
    }
    /// Get timer's kind and deadline.
    pub fn get_kind_and_deadline(self) -> (T, Instant) {
        (self.kind, self.deadline)
    }
    /// Create a timer from deadline.
    pub fn from_deadline(deadline: Instant, kind: T) -> Self {
        Timer { deadline, kind }
    }
    /// Create a timer from millis.
    #[cfg(test)]
    pub fn from_millis(millis: u64, kind: T, now: Instant) -> Self {
        Timer {
            deadline: now + Duration::from_millis(millis),
            kind,
        }
    }
}
