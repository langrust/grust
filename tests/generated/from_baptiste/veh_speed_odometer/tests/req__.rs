use veh_speed_odometer::typedefs::{Config, VehiculeSpeedLevel};
use veh_speed_odometer::vehicule_speed_odometer_alert::*;

#[test]
fn should_always_raise_no_alert_in_condition() {
    let config = Config::NoWarning;
    let speed_range = 70..150;
    let time_range = 0..100;
    let inputs =
        speed_range
            .zip(time_range)
            .map(|(speed_kmh, time_ms)| VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh,
                time_ms,
            });

    let mut state = VehiculeSpeedOdometerAlertState::init();
    for input in inputs {
        assert_eq!(state.step(input), VehiculeSpeedLevel::Level0)
    }
}
