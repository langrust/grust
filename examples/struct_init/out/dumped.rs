#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Priority {
    #[default]
    Hight,
    Medium,
    Low,
}
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Alarm {
    pub prio: Priority,
    pub data: i64,
}
pub struct AlarmFilteringInput {
    pub alarm: Alarm,
}
pub struct AlarmFilteringState {
    last_temp: Alarm,
}
impl grust::core::Component for AlarmFilteringState {
    type Input = AlarmFilteringInput;
    type Output = Alarm;
    fn init() -> AlarmFilteringState {
        AlarmFilteringState {
            last_temp: Alarm {
                prio: Priority::Low,
                data: 0i64,
            },
        }
    }
    fn step(&mut self, input: AlarmFilteringInput) -> Alarm {
        let delayed = self.last_temp;
        let temp = input.alarm;
        self.last_temp = temp;
        delayed
    }
}
