use veh_speed_odometer::typedefs::{Config, VehiculeSpeedLevel};
use veh_speed_odometer::vehicule_speed_odometer_alert::*;

#[test]
fn should_always_raise_no_alert_in_condition() {
    let config = Config::ArabicCountries;
    let inputs_raise = [
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 70,
                time_ms: 10,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 81,
                time_ms: 100,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 85,
                time_ms: 200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 79,
                time_ms: 220,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 83,
                time_ms: 400,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 86,
                time_ms: 600,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 90,
                time_ms: 850,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                time_ms: 1000,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 1200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                time_ms: 1500,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 2000,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 130,
                time_ms: 2500,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 3400,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                time_ms: 3600,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 90,
                time_ms: 3850,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                time_ms: 4000,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 4200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                time_ms: 4500,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 5000,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 130,
                time_ms: 5500,
            },
            false,
        ),
    ];

    let mut state = VehiculeSpeedOdometerAlertState::init();
    for (input, raise) in inputs_raise {
        let output = state.step(input);
        if raise {
            assert_eq!(output, VehiculeSpeedLevel::Level0)
        }
    }
}
