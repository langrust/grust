use crate::functions::alarms_processing;
use crate::typedefs::Alarm;
pub struct AlarmManagerNbRaisedInput {
    pub alarms: [Alarm; 10usize],
}
pub struct AlarmManagerNbRaisedState {}
impl AlarmManagerNbRaisedState {
    pub fn init() -> AlarmManagerNbRaisedState {
        AlarmManagerNbRaisedState {}
    }
    pub fn step(&mut self, input: AlarmManagerNbRaisedInput) -> i64 {
        let nb_raised = alarms_processing(input.alarms);
        nb_raised
    }
}
