pub fn high_priority(alarm: Alarm) -> bool {
    let is_raiseable = match alarm {
        Alarm { priority: Priority::High, raised: raised } => raised,
        _ => false,
    };
    is_raiseable
}
pub fn alarms_processing(alarms: [Alarm; 10usize]) -> [bool; 10usize] {
    alarms.map(high_priority)
}
