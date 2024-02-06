pub struct AlarmManagerRaiseInput {
    pub alarms: [Alarm; 10usize],
}
pub struct AlarmManagerRaiseState {}
impl AlarmManagerRaiseState {
    pub fn init() -> AlarmManagerRaiseState {
        AlarmManagerRaiseState {}
    }
    pub fn step(&mut self, input: AlarmManagerRaiseInput) -> [bool; 10usize] {
        let raise = input.alarms.map(high_priority);
        raise
    }
}
