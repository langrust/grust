use veh_speed_odometer::typedefs::{Config, VehiculeSpeedLevel};
use veh_speed_odometer::vehicule_speed_odometer_alert::*;

#[test]
fn should_always_raise_high_alert_in_condition() {
    let config = Config::India;
    let inputs_raise = [
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 70,
                time_ms: 10,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 81,
                time_ms: 100,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 85,
                time_ms: 200,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 79,
                time_ms: 220,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 83,
                time_ms: 400,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 86,
                time_ms: 600,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 90,
                time_ms: 850,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                time_ms: 1000,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 1200,
            },
            false,
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
                speed_kmh: 123,
                time_ms: 2000,
            },
            false,
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
                speed_kmh: 125,
                time_ms: 3001,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 123,
                time_ms: 3200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 121,
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
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 90,
                time_ms: 3850,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                time_ms: 4000,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 4200,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                time_ms: 4300,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 4500,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 130,
                time_ms: 4900,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 125,
                time_ms: 6000,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 123,
                time_ms: 6200,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 121,
                time_ms: 6400,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                time_ms: 6600,
            },
            false,
        ),
    ];

    let mut state = VehiculeSpeedOdometerAlertState::init();
    for (input, raise) in inputs_raise {
        let output = state.step(input);
        if raise {
            assert_eq!(output, VehiculeSpeedLevel::Level3)
        }
    }
}
