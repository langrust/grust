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
                time_ms: 10,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 123,
                time_ms: 100,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 124,
                time_ms: 500,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 126,
                time_ms: 1220,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 127,
                time_ms: 1800,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 130,
                time_ms: 2300,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 128,
                time_ms: 2950,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 129,
                time_ms: 3150,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 125,
                time_ms: 3700,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 121,
                time_ms: 3900,
            },
            true,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
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
                speed_kmh: 130,
                time_ms: 4500,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 5400,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                time_ms: 5600,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 90,
                time_ms: 5850,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 100,
                time_ms: 6000,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 6200,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 119,
                time_ms: 6500,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 110,
                time_ms: 6000,
            },
            false,
        ),
        (
            VehiculeSpeedOdometerAlertInput {
                vehicule_config: config,
                speed_kmh: 130,
                time_ms: 6500,
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
