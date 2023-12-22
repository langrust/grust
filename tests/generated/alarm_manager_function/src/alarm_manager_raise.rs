use crate::functions::alarms_processing;
pub struct AlarmManagerRaiseInput {
    pub alarms: [Alarm; 10],
}
pub struct AlarmManagerRaiseState {}
impl AlarmManagerRaiseState {
    pub fn init() -> AlarmManagerRaiseState {
        AlarmManagerRaiseState {}
    }
    pub fn step(&mut self, input: AlarmManagerRaiseInput) -> [bool; 10] {
        let raise = alarms_processing(input.alarms);
        raise
    }
}
