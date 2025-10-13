#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alarm {
    pub priority: Priority,
    pub raised: bool,
}
