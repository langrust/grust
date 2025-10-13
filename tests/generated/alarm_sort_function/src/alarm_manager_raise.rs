use crate::functions::sorting;
use crate::typedefs::Alarm;
pub struct AlarmManagerRaiseInput {
    pub alarms: [Alarm; 10usize],
}
pub struct AlarmManagerRaiseState {}
impl AlarmManagerRaiseState {
    pub fn init() -> AlarmManagerRaiseState {
        AlarmManagerRaiseState {}
    }
    pub fn step(&mut self, input: AlarmManagerRaiseInput) -> [Alarm; 10usize] {
        let raise = sorting(input.alarms);
        raise
    }
}
