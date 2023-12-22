pub fn high_priority(alarm: Alarm) -> bool {
    let is_raiseable = match alarm {
        Alarm { priority: Priority::High, raised } => raised,
        _ => false,
    };
    is_raiseable
}
