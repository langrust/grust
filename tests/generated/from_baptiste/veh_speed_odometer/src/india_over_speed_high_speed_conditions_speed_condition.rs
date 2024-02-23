use crate::india_over_speed_condition_speed_condition_holds::*;
pub struct IndiaOverSpeedHighSpeedConditionsSpeedConditionInput {
    pub speed_kmh: i64,
    pub time_ms: i64,
}
pub struct IndiaOverSpeedHighSpeedConditionsSpeedConditionState {
    india_over_speed_condition_speed_condition_holds: IndiaOverSpeedConditionSpeedConditionHoldsState,
}
impl IndiaOverSpeedHighSpeedConditionsSpeedConditionState {
    pub fn init() -> IndiaOverSpeedHighSpeedConditionsSpeedConditionState {
        IndiaOverSpeedHighSpeedConditionsSpeedConditionState {
            india_over_speed_condition_speed_condition_holds: IndiaOverSpeedConditionSpeedConditionHoldsState::init(),
        }
    }
    pub fn step(
        &mut self,
        input: IndiaOverSpeedHighSpeedConditionsSpeedConditionInput,
    ) -> bool {
        let x = move |s: i64| -> bool { 120i64 < s };
        let speed_condition = self
            .india_over_speed_condition_speed_condition_holds
            .step(IndiaOverSpeedConditionSpeedConditionHoldsInput {
                condition: x,
                speed_kmh: input.speed_kmh,
                time_ms: input.time_ms,
            });
        speed_condition
    }
}
