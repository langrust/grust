use crate::during_result::*;
use crate::typedefs::VehiculeSpeedLevel;
pub struct ArabicCountiresOverSpeedWarningAlertInput {
    pub speed_kmh: i64,
    pub dt_ms: i64,
}
pub struct ArabicCountiresOverSpeedWarningAlertState {
    during_result: DuringResultState,
}
impl ArabicCountiresOverSpeedWarningAlertState {
    pub fn init() -> ArabicCountiresOverSpeedWarningAlertState {
        ArabicCountiresOverSpeedWarningAlertState {
            during_result: DuringResultState::init(),
        }
    }
    pub fn step(
        &mut self,
        input: ArabicCountiresOverSpeedWarningAlertInput,
    ) -> VehiculeSpeedLevel {
        let x_1 = 3000i64;
        let x = 120i64 < input.speed_kmh;
        let alert_on = self
            .during_result
            .step(DuringResultInput {
                condition: x,
                duration_ms: x_1,
                dt_ms: input.dt_ms,
            });
        let alert_off = input.speed_kmh <= 118i64;
        let alert = match (alert_off, alert_on) {
            (true, _) => VehiculeSpeedLevel::Level0,
            (_, _) => VehiculeSpeedLevel::Level1,
        };
        alert
    }
}
