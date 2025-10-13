use crate::typedefs::Priority;
use crate::typedefs::Alarm;
pub fn sum_alarm(sum: i64, alarm: Alarm) -> i64 {
    let is_raiseable = match (alarm) {
        Alarm { priority: Priority::High, raised: raised } => raised,
        _ => false,
    };
    let new_sum = if is_raiseable { sum + 1i64 } else { sum };
    new_sum
}
pub fn alarms_processing(alarms: [Alarm; 10usize]) -> i64 {
    alarms.into_iter().fold(0i64, sum_alarm)
}
