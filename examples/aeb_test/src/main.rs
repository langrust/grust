#![allow(warnings)]
mod aeb {
    use grust::grust;

    grust! {
        #![mode = test, dump = "examples/aeb_test/out/mod.rs"]
        import signal car::speed_km_h                   : float;
        import event  car::detect::left::pedestrian_l   : float;
        import event  car::detect::right::pedestrian_r  : float;
        export signal car::urban::braking::brakes       : Braking;

        // Braking type
        enum Braking {
            NoBrake,
            SoftBrake,
            UrgentBrake,
        }

        // Formula: d = 2 * s^2 / (250 * f)
        // d = braking distance in metres (to be calculated).
        // s = speed in km/h.
        // 250 = fixed figure which is always used.
        // f = coefficient of friction, approx. 0.8 on dry asphalt.
        function compute_soft_braking_distance(speed: float) -> float {
            return speed * speed / 100.0;
        }

        // determine braking strategy
        function brakes(distance: float, speed: float) -> Braking {
            let braking_distance: float = compute_soft_braking_distance(speed);
            let response: Braking = if braking_distance < distance
                                    then Braking::SoftBrake
                                    else Braking::UrgentBrake;
            return response;
        }

        component braking_state(pedest: float?, timeout_pedestrian: unit?, speed: float) -> (state: Braking) {
            when {
                init                                                              => { state = Braking::NoBrake;   }
                let d = pedest?                                                   => { state = brakes(d, speed);   }
                let _ = timeout_pedestrian? if last state == Braking::UrgentBrake => { state = Braking::SoftBrake; }
                let _ = timeout_pedestrian?                                       => { state = Braking::NoBrake;   }
            }
        }

        service aeb @ [10, 3000] {
            let event pedestrian: float = merge(pedestrian_l, pedestrian_r);
            let event timeout_pedestrian: unit = timeout(pedestrian, 2000);
            brakes = braking_state(pedestrian, timeout_pedestrian, speed_km_h);
        }
    }
}

use aeb::{
    run,
    runtime::{RuntimeInit, RuntimeInput, RuntimeOutput},
    Braking,
};
use grust::{
    futures::{self, Stream, StreamExt},
    tokio,
};
use json::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// JSON input type, without timestamp.
#[derive(Deserialize, Debug)]
#[serde(tag = "variant", content = "value")]
pub enum Input {
    PedestrianL(f64),
    PedestrianR(f64),
    SpeedKmH(f64),
}
impl From<(Instant, Input)> for RuntimeInput {
    fn from(value: (Instant, Input)) -> Self {
        match value {
            (instant, Input::PedestrianL(val)) => RuntimeInput::PedestrianL(val, instant),
            (instant, Input::PedestrianR(val)) => RuntimeInput::PedestrianR(val, instant),
            (instant, Input::SpeedKmH(val)) => RuntimeInput::SpeedKmH(val, instant),
        }
    }
}

/// JSON output type, without timestamp.
#[derive(Serialize, Debug)]
pub enum Output {
    Brakes(usize),
}
#[derive(Serialize, Debug)]
pub struct TimedOutput(u64, Output);
impl From<(Instant, RuntimeOutput)> for TimedOutput {
    fn from(value: (Instant, RuntimeOutput)) -> Self {
        match value {
            (INIT, RuntimeOutput::Brakes(Braking::NoBrake, instant)) => TimedOutput(
                instant.duration_since(INIT).as_millis() as u64,
                Output::Brakes(0),
            ),
            (INIT, RuntimeOutput::Brakes(Braking::SoftBrake, instant)) => TimedOutput(
                instant.duration_since(INIT).as_millis() as u64,
                Output::Brakes(1),
            ),
            (INIT, RuntimeOutput::Brakes(Braking::UrgentBrake, instant)) => TimedOutput(
                instant.duration_since(INIT).as_millis() as u64,
                Output::Brakes(2),
            ),
        }
    }
}

#[tokio::main]
async fn main() {
    const INPUT_PATH: &str = "examples/aeb_test/data/inputs.json";
    const OUTPUT_PATH: &str = "examples/aeb_test/data/outputs.json";
    let INIT: Instant = Instant::now();

    // read inputs
    let read_stream = futures::stream::iter(read_json(INPUT_PATH));

    // transform in RuntimeInput + sleep
    let input_stream = read_stream.filter_map(move |input: Result<(u64, Input), _>| async move {
        match input {
            Ok((timestamp, input)) => {
                let duration = tokio::time::Duration::from_millis(timestamp as u64);
                let instant = INIT + duration;
                Some((instant, input).into())
            }
            Err(err) => panic!("{err}"),
        }
    });

    // initiate JSON file
    begin_json(OUTPUT_PATH);

    // collect N outputs
    const N: usize = 10;
    let mut output_stream = run(INIT, input_stream, RuntimeInit { speed_km_h: 0. });
    let mut counter = 0;
    while let Some(received) = output_stream.next().await {
        counter += 1;
        let output: TimedOutput = (INIT, received).into();
        append_json(OUTPUT_PATH, output);
        // stop at N
        if counter >= N {
            break;
        }
    }

    // finalize JSON file
    end_json(OUTPUT_PATH);
}
