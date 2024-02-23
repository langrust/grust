use crate::typedefs::VehiculeSpeedLevel;
use crate::india_over_speed_condition_speed_condition_holds::*;
pub struct IndiaOverSpeedLowSpeedConditionsSpeedConditionInput {
    pub speed_kmh: i64,
    pub prev_alert: VehiculeSpeedLevel,
    pub time_ms: i64,
}
pub struct IndiaOverSpeedLowSpeedConditionsSpeedConditionState {
    india_over_speed_condition_speed_condition_holds: IndiaOverSpeedConditionSpeedConditionHoldsState,
}
impl IndiaOverSpeedLowSpeedConditionsSpeedConditionState {
    pub fn init() -> IndiaOverSpeedLowSpeedConditionsSpeedConditionState {
        IndiaOverSpeedLowSpeedConditionsSpeedConditionState {
            india_over_speed_condition_speed_condition_holds: IndiaOverSpeedConditionSpeedConditionHoldsState::init(),
        }
    }
    pub fn step(
        &mut self,
        input: IndiaOverSpeedLowSpeedConditionsSpeedConditionInput,
    ) -> bool {
        let speed_condition_2 = input.prev_alert == VehiculeSpeedLevel::Level3
            && input.speed_kmh <= 118i64;
        let x = move |s: i64| -> bool { 80i64 < s && s <= 120i64 };
        let speed_condition_1 = self
            .india_over_speed_condition_speed_condition_holds
            .step(IndiaOverSpeedConditionSpeedConditionHoldsInput {
                condition: x,
                speed_kmh: input.speed_kmh,
                time_ms: input.time_ms,
            });
        let speed_condition = speed_condition_1 || speed_condition_2;
        speed_condition
    }
}
