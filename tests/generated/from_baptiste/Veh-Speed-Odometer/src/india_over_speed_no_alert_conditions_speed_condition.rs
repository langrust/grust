pub struct IndiaOverSpeedNoAlertConditionsSpeedConditionInput {
    pub speed_kmh: i64,
}
pub struct IndiaOverSpeedNoAlertConditionsSpeedConditionState {}
impl IndiaOverSpeedNoAlertConditionsSpeedConditionState {
    pub fn init() -> IndiaOverSpeedNoAlertConditionsSpeedConditionState {
        IndiaOverSpeedNoAlertConditionsSpeedConditionState {
        }
    }
    pub fn step(
        &mut self,
        input: IndiaOverSpeedNoAlertConditionsSpeedConditionInput,
    ) -> bool {
        let speed_condition = input.speed_kmh <= 78i64;
        speed_condition
    }
}
