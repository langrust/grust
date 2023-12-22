pub struct AlarmManagerRaiseInput {
    pub alarms: [Alarm; 10],
}
pub struct AlarmManagerRaiseState {}
impl AlarmManagerRaiseState {
    pub fn init() -> AlarmManagerRaiseState {
        AlarmManagerRaiseState {}
    }
    pub fn step(&mut self, input: AlarmManagerRaiseInput) -> [bool; 10] {
        let raise = input.alarms.map(high_priority);
        raise
    }
}
