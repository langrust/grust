use crate::during_holds_during::*;
use crate::typedefs::VehiculeSpeedLevel;
pub struct ArabicCountiresOverSpeedWarningAlertInput {
    pub speed_kmh: i64,
    pub time_ms: i64,
}
pub struct ArabicCountiresOverSpeedWarningAlertState {
    during_holds_during: DuringHoldsDuringState,
}
impl ArabicCountiresOverSpeedWarningAlertState {
    pub fn init() -> ArabicCountiresOverSpeedWarningAlertState {
        ArabicCountiresOverSpeedWarningAlertState {
            during_holds_during: DuringHoldsDuringState::init(),
        }
    }
    pub fn step(
        &mut self,
        input: ArabicCountiresOverSpeedWarningAlertInput,
    ) -> VehiculeSpeedLevel {
        let x_1 = 3000i64;
        let x = 120i64 < input.speed_kmh;
        let alert_on = self
            .during_holds_during
            .step(DuringHoldsDuringInput {
                condition: x,
                duration_ms: x_1,
                time_ms: input.time_ms,
            });
        let alert_off = input.speed_kmh <= 118i64;
        let alert = match (alert_off, alert_on) {
            (true, _) => VehiculeSpeedLevel::Level0,
            (_, true) => VehiculeSpeedLevel::Level1,
        };
        alert
    }
}
