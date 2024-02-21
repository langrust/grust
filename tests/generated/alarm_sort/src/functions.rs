use crate::typedefs::Priority;
pub fn priority_order(priority: Priority, other: Priority) -> i64 {
    let higher_priority = match (priority) {
        Priority::High => {
            match (other) {
                Priority::High => 0i64,
                _ => 1i64,
            }
        }
        Priority::Medium => {
            match (other) {
                Priority::High => -1i64,
                Priority::Medium => 0i64,
                Priority::Low => 1i64,
            }
        }
        Priority::Low => {
            match (other) {
                Priority::Low => 1i64,
                _ => -1i64,
            }
        }
    };
    higher_priority
}
use crate::typedefs::Alarm;
pub fn prioritize(alarm: Alarm, other: Alarm) -> i64 {
    let raise_difference = alarm.raised != other.raised;
    let priority_ordering = priority_order(alarm.priority, other.priority);
    if raise_difference {
        (if alarm.raised { 1i64 } else { -1i64 })
    } else {
        priority_ordering
    }
}
