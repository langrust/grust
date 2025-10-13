use crate::functions::prioritize;
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
        let raise = {
            let mut x = input.alarms.clone();
            let slice = x.as_mut();
            slice
                .sort_by(|a, b| {
                    let compare = prioritize(*a, *b);
                    if compare < 0 {
                        std::cmp::Ordering::Less
                    } else if compare > 0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            x
        };
        raise
    }
}
