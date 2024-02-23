use crate::during_holds_during::*;
pub struct IndiaOverSpeedConditionSpeedConditionHoldsInput<F: Fn(i64) -> bool> {
    pub condition: F,
    pub speed_kmh: i64,
    pub time_ms: i64,
}
pub struct IndiaOverSpeedConditionSpeedConditionHoldsState {
    during_holds_during: DuringHoldsDuringState,
}
impl IndiaOverSpeedConditionSpeedConditionHoldsState {
    pub fn init() -> IndiaOverSpeedConditionSpeedConditionHoldsState {
        IndiaOverSpeedConditionSpeedConditionHoldsState {
            during_holds_during: DuringHoldsDuringState::init(),
        }
    }
    pub fn step<F: Fn(i64) -> bool>(
        &mut self,
        input: IndiaOverSpeedConditionSpeedConditionHoldsInput<F>,
    ) -> bool {
        let x = 1000i64;
        let speed_condition = (input.condition)(input.speed_kmh);
        let speed_condition_holds = self
            .during_holds_during
            .step(DuringHoldsDuringInput {
                condition: speed_condition,
                duration_ms: x,
                time_ms: input.time_ms,
            });
        speed_condition_holds
    }
}
