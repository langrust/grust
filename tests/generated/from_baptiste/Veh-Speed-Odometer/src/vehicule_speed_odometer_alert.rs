use crate::typedefs::VehiculeSpeedLevel;
use crate::arabic_countires_over_speed_warning_alert::*;
use crate::india_over_speed_warning_alert::*;
use crate::typedefs::Config;
pub struct VehiculeSpeedOdometerAlertInput {
    pub vehicule_config: Config,
    pub speed_kmh: i64,
    pub time_ms: i64,
}
pub struct VehiculeSpeedOdometerAlertState {
    arabic_countires_over_speed_warning_alert: ArabicCountiresOverSpeedWarningAlertState,
    india_over_speed_warning_alert: IndiaOverSpeedWarningAlertState,
}
impl VehiculeSpeedOdometerAlertState {
    pub fn init() -> VehiculeSpeedOdometerAlertState {
        VehiculeSpeedOdometerAlertState {
            arabic_countires_over_speed_warning_alert: ArabicCountiresOverSpeedWarningAlertState::init(),
            india_over_speed_warning_alert: IndiaOverSpeedWarningAlertState::init(),
        }
    }
    pub fn step(
        &mut self,
        input: VehiculeSpeedOdometerAlertInput,
    ) -> VehiculeSpeedLevel {
        let alert = match input.vehicule_config {
            NoWarning => VehiculeSpeedLevel::Level0,
            ArabicCountries => {
                let x = self
                    .arabic_countires_over_speed_warning_alert
                    .step(ArabicCountiresOverSpeedWarningAlertInput {
                        speed_kmh: input.speed_kmh,
                        time_ms: input.time_ms,
                    });
                x
            }
            India => {
                let x_1 = self
                    .india_over_speed_warning_alert
                    .step(IndiaOverSpeedWarningAlertInput {
                        speed_kmh: input.speed_kmh,
                        time_ms: input.time_ms,
                    });
                x_1
            }
        };
        alert
    }
}
