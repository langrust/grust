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
pub struct DelayedAlarmInput {
    pub alarm: Alarm,
}
pub struct DelayedAlarmOutput {
    pub delayed: Alarm,
}
pub struct DelayedAlarmState {
    last_temp: Alarm,
}
impl grust::core::Component for DelayedAlarmState {
    type Input = DelayedAlarmInput;
    type Output = DelayedAlarmOutput;
    fn init() -> DelayedAlarmState {
        DelayedAlarmState {
            last_temp: Alarm {
                prio: Priority::Low,
                data: 0i64,
            },
        }
    }
    fn step(&mut self, input: DelayedAlarmInput) -> DelayedAlarmOutput {
        let delayed = self.last_temp;
        let temp = input.alarm;
        self.last_temp = temp;
        DelayedAlarmOutput { delayed }
    }
}
