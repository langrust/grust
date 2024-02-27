use crate::india_over_speed_no_alert_conditions_speed_condition::*;
use crate::india_over_speed_low_speed_conditions_speed_condition::*;
use crate::india_over_speed_high_speed_conditions_speed_condition::*;
use crate::typedefs::VehiculeSpeedLevel;
pub struct IndiaOverSpeedWarningAlertInput {
    pub speed_kmh: i64,
    pub dt_ms: i64,
}
pub struct IndiaOverSpeedWarningAlertState {
    mem_prev_alert: VehiculeSpeedLevel,
    india_over_speed_no_alert_conditions_speed_condition: IndiaOverSpeedNoAlertConditionsSpeedConditionState,
    india_over_speed_low_speed_conditions_speed_condition: IndiaOverSpeedLowSpeedConditionsSpeedConditionState,
    india_over_speed_high_speed_conditions_speed_condition: IndiaOverSpeedHighSpeedConditionsSpeedConditionState,
}
impl IndiaOverSpeedWarningAlertState {
    pub fn init() -> IndiaOverSpeedWarningAlertState {
        IndiaOverSpeedWarningAlertState {
            mem_prev_alert: VehiculeSpeedLevel::Level0,
            india_over_speed_no_alert_conditions_speed_condition: IndiaOverSpeedNoAlertConditionsSpeedConditionState::init(),
            india_over_speed_low_speed_conditions_speed_condition: IndiaOverSpeedLowSpeedConditionsSpeedConditionState::init(),
            india_over_speed_high_speed_conditions_speed_condition: IndiaOverSpeedHighSpeedConditionsSpeedConditionState::init(),
        }
    }
    pub fn step(
        &mut self,
        input: IndiaOverSpeedWarningAlertInput,
    ) -> VehiculeSpeedLevel {
        let no_alert = self
            .india_over_speed_no_alert_conditions_speed_condition
            .step(IndiaOverSpeedNoAlertConditionsSpeedConditionInput {
                speed_kmh: input.speed_kmh,
            });
        let prev_alert = self.mem_prev_alert;
        let low_alert = self
            .india_over_speed_low_speed_conditions_speed_condition
            .step(IndiaOverSpeedLowSpeedConditionsSpeedConditionInput {
                speed_kmh: input.speed_kmh,
                prev_alert: prev_alert,
                dt_ms: input.dt_ms,
            });
        let high_alert = self
            .india_over_speed_high_speed_conditions_speed_condition
            .step(IndiaOverSpeedHighSpeedConditionsSpeedConditionInput {
                speed_kmh: input.speed_kmh,
                dt_ms: input.dt_ms,
            });
        let alert = match (high_alert, low_alert, no_alert) {
            (_, _, true) => VehiculeSpeedLevel::Level0,
            (_, true, _) => VehiculeSpeedLevel::Level2,
            (_, _, _) => VehiculeSpeedLevel::Level3,
        };
        self.mem_prev_alert = alert;
        alert
    }
}
