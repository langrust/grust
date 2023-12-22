use crate::functions::sorting;
pub struct AlarmManagerRaiseInput {
    pub alarms: [Alarm; 10],
}
pub struct AlarmManagerRaiseState {}
impl AlarmManagerRaiseState {
    pub fn init() -> AlarmManagerRaiseState {
        AlarmManagerRaiseState {}
    }
    pub fn step(&mut self, input: AlarmManagerRaiseInput) -> [Alarm; 10] {
        let raise = sorting(input.alarms);
        raise
    }
}
