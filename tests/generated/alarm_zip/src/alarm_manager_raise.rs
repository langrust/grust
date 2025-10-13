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
        let combined = {
            let mut iter = itertools::izip!(input.alarms, input.frequences);
            std::array::from_fn(|_| iter.next().unwrap())
        };
        let raise = combined.map(high_priority);
        raise
    }
}
