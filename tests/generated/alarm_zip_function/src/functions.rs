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
pub fn alarms_processing(
    alarms: [Alarm; 10usize],
    frequences: [i64; 10usize],
) -> [(Alarm, i64); 10usize] {
    {
        let mut iter = itertools::izip!(alarms, frequences);
        std::array::from_fn(|_| iter.next().unwrap())
    }
}
