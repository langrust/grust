use crate::during_result::*;
pub struct IndiaOverSpeedConditionSpeedConditionHoldsInput<F: Fn(i64) -> bool> {
    pub condition: F,
    pub speed_kmh: i64,
    pub dt_ms: i64,
}
pub struct IndiaOverSpeedConditionSpeedConditionHoldsState {
    during_result: DuringResultState,
}
impl IndiaOverSpeedConditionSpeedConditionHoldsState {
    pub fn init() -> IndiaOverSpeedConditionSpeedConditionHoldsState {
        IndiaOverSpeedConditionSpeedConditionHoldsState {
            during_result: DuringResultState::init(),
        }
    }
    pub fn step<F: Fn(i64) -> bool>(
        &mut self,
        input: IndiaOverSpeedConditionSpeedConditionHoldsInput<F>,
    ) -> bool {
        let x = 1000i64;
        let speed_condition = (input.condition)(input.speed_kmh);
        let speed_condition_holds = self
            .during_result
            .step(DuringResultInput {
                condition: speed_condition,
                duration_ms: x,
                dt_ms: input.dt_ms,
            });
        speed_condition_holds
    }
}
