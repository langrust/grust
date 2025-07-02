#![allow(warnings)]
mod aeb {
    use grust::grust;

    grust! {
        #![mode = demo]
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

        component braking_state(pedest: float?, timeout_pedestrian: unit?, speed: float)
                            -> (state: Braking)
            requires { 0. <= speed && speed < 55. } // urban limit
            ensures { when _x = pedest? => state != Braking::NoBrake } // safety
        {
            log (pedest, state);
            state = when {
                init                        => Braking::NoBrake,
                let d = pedest?             => brakes(d, speed),
                let _ = timeout_pedestrian? => Braking::NoBrake,
            };
        }

        service aeb @ [10, 3000] {
            let event pedestrian: float = merge(pedestrian_l, pedestrian_r);
            let event timeout_pedestrian: unit = timeout(pedestrian, 2000);
            brakes = braking_state(pedestrian, timeout_pedestrian, speed_km_h);
        }
    }

    use super::INIT;
    use futures::{Stream, StreamExt};
    use priority_stream::prio_stream;
    use runtime::{Runtime, RuntimeInit, RuntimeInput, RuntimeOutput, RuntimeTimer};
    use timer_stream::timer_stream;

    pub fn run_aeb(
        input_stream: impl Stream<Item = RuntimeInput> + Send + 'static,
        init_signals: RuntimeInit,
    ) -> impl Stream<Item = RuntimeOutput> {
        const OUTPUT_CHANNEL_SIZE: usize = 4;
        const TIMER_CHANNEL_SIZE: usize = 4;
        const PRIO_STREAM_SIZE: usize = 6;
        const TIMER_STREAM_SIZE: usize = 3;

        let (output_sink, output_stream) = futures::channel::mpsc::channel(OUTPUT_CHANNEL_SIZE);
        let (timers_sink, timers_stream) = futures::channel::mpsc::channel(TIMER_CHANNEL_SIZE);

        let timers_stream = timer_stream::<_, _, TIMER_STREAM_SIZE>(timers_stream)
            .map(|(timer, deadline)| RuntimeInput::Timer(timer, deadline));
        let prio_stream = prio_stream::<_, _, PRIO_STREAM_SIZE>(
            futures::stream::select(input_stream, timers_stream),
            RuntimeInput::order,
        );

        let aeb_service = Runtime::new(output_sink, timers_sink);
        tokio::spawn(aeb_service.run_loop(*INIT, prio_stream, init_signals));

        output_stream
    }
}

use aeb::{
    run_aeb,
    runtime::{RuntimeInit, RuntimeInput, RuntimeOutput},
    Braking,
};
use futures::{Stream, StreamExt};
use json::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

lazy_static! {
    /// Initial instant.
    static ref INIT : Instant = Instant::now();
}

/// JSON input type, without timestamp.
#[derive(Deserialize, std::fmt::Debug)]
#[serde(tag = "variant", content = "value")]
pub enum Input {
    PedestrianL(f64),
    PedestrianR(f64),
    SpeedKmH(f64),
}
impl Input {
    fn into(self, instant: Instant) -> RuntimeInput {
        match self {
            Input::PedestrianL(val) => RuntimeInput::PedestrianL(val, instant),
            Input::PedestrianR(val) => RuntimeInput::PedestrianR(val, instant),
            Input::SpeedKmH(val) => RuntimeInput::SpeedKmH(val, instant),
        }
    }
}

/// JSON output type, without timestamp.
#[derive(Serialize, std::fmt::Debug)]
pub enum Output {
    Brakes(usize),
}
impl From<RuntimeOutput> for Output {
    fn from(value: RuntimeOutput) -> Self {
        match value {
            RuntimeOutput::Brakes(Braking::NoBrake, _) => Output::Brakes(0),
            RuntimeOutput::Brakes(Braking::SoftBrake, _) => Output::Brakes(1),
            RuntimeOutput::Brakes(Braking::UrgentBrake, _) => Output::Brakes(2),
        }
    }
}

#[tokio::main]
async fn main() {
    const INPUT_PATH: &str = "examples/simple_aeb_demo/data/inputs.json";
    const OUTPUT_PATH: &str = "examples/simple_aeb_demo/data/outputs.json";

    // read inputs
    let read_stream = futures::stream::iter(read_json(INPUT_PATH));

    // transform in RuntimeInput + sleep
    let input_stream = read_stream.filter_map(move |input: Result<(u64, Input), _>| async move {
        match input {
            Ok((timestamp, input)) => {
                let duration = tokio::time::Duration::from_millis(timestamp as u64);
                let instant = *INIT + duration;
                tokio::time::sleep_until(instant.into()).await;
                Some(input.into(instant))
            }
            Err(_) => None,
        }
    });

    // initiate JSON file
    begin_json(OUTPUT_PATH);

    // collect N outputs
    const N: usize = 10;
    let mut output_stream = run_aeb(input_stream, RuntimeInit { speed_km_h: 0.0 });
    let mut counter = 0;
    while let Some(received) = output_stream.next().await {
        counter += 1;
        let output: Output = received.into();
        append_json(OUTPUT_PATH, output);
        // stop at N
        if counter >= N {
            break;
        }
    }

    // finalize JSON file
    end_json(OUTPUT_PATH);
}
