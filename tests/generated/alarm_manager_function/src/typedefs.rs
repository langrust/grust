pub enum Priority {
    Low,
    Medium,
    High,
}
pub struct Alarm {
    pub priority: Priority,
    pub raised: bool,
}
