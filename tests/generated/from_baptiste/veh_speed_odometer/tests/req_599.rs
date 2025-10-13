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
                dt_ms: 10,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 81,
                dt_ms: 90,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 85,
                dt_ms: 100,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 79,
                dt_ms: 20,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 83,
                dt_ms: 180,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 86,
                dt_ms: 200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 90,
                dt_ms: 250,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                dt_ms: 150,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                dt_ms: 200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                dt_ms: 300,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                dt_ms: 500,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 130,
                dt_ms: 500,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                dt_ms: 900,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                dt_ms: 200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 90,
                dt_ms: 250,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                dt_ms: 150,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                dt_ms: 200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                dt_ms: 300,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                dt_ms: 500,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 130,
                dt_ms: 500,
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
