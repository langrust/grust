use crate::typedefs::Alarm;
use crate::functions::alarms_processing;
pub struct AlarmManagerNbRaisedInput {
    pub alarms: [Alarm; 10],
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
