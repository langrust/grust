pub struct AlarmManagerNbRaisedInput {
    pub alarms: [Alarm; 10usize],
}
pub struct AlarmManagerNbRaisedState {}
impl AlarmManagerNbRaisedState {
    pub fn init() -> AlarmManagerNbRaisedState {
        AlarmManagerNbRaisedState {}
    }
    pub fn step(&mut self, input: AlarmManagerNbRaisedInput) -> i64 {
        let nb_raised = input.alarms.into_iter().fold(0i64, sum_alarm);
        nb_raised
    }
}
