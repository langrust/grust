use crate::functions::alarms_processing;
use crate::functions::high_priority;
use crate::typedefs::Alarm;
pub struct AlarmManagerRaiseInput {
    pub alarms: [Alarm; 10usize],
    pub frequences: [i64; 10usize],
}
pub struct AlarmManagerRaiseState {}
impl AlarmManagerRaiseState {
    pub fn init() -> AlarmManagerRaiseState {
        AlarmManagerRaiseState {}
    }
    pub fn step(&mut self, input: AlarmManagerRaiseInput) -> [bool; 10usize] {
        let raise = alarms_processing(input.alarms, input.frequences).map(high_priority);
        raise
    }
}
