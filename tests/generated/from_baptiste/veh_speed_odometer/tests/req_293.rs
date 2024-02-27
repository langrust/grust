use veh_speed_odometer::typedefs::{Config, VehiculeSpeedLevel};
use veh_speed_odometer::vehicule_speed_odometer_alert::*;

#[test]
fn should_always_raise_alert_in_condition() {
    let config = Config::ArabicCountries;
    let inputs_raise = [
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                dt_ms: 10,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 123,
                dt_ms: 90,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 124,
                dt_ms: 400,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 126,
                dt_ms: 720,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 127,
                dt_ms: 580,
            },
            false,
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
                speed_kmh: 128,
                dt_ms: 650,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 129,
                dt_ms: 200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 125,
                dt_ms: 550,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 121,
                dt_ms: 200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                dt_ms: 100,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                dt_ms: 200,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 130,
                dt_ms: 300,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                dt_ms: 900,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                dt_ms: 200,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 90,
                dt_ms: 250,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                dt_ms: 150,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                dt_ms: 200,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                dt_ms: 300,
            },
            false,
        ),
    ];

    let mut state = VehiculeSpeedOdometerAlertState::init();
    for (input, raise) in inputs_raise {
        let output = state.step(input);
        if raise {
            assert_eq!(output, VehiculeSpeedLevel::Level1)
        }
    }
}
