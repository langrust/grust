use crate::typedefs::Priority;
use crate::typedefs::Alarm;
pub fn high_priority(alarm_frequence: (Alarm, i64)) -> bool {
    let alarm = alarm_frequence.0;
    let frequence = alarm_frequence.1;
    let is_raiseable = frequence > 3i64
        && match (alarm) {
            Alarm { priority: Priority::High, raised: raised } => raised,
            _ => false,
        };
    is_raiseable
}
