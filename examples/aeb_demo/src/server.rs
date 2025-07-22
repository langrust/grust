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
}

use aeb::runtime::{RuntimeInit, RuntimeInput, RuntimeOutput};
use futures::StreamExt;
use interface::{
    aeb_server::{Aeb, AebServer},
    input::Message,
    Braking, Input, Output, Pedestrian, Speed,
};
use lazy_static::lazy_static;
use std::time::{Duration, Instant};
use tonic::{transport::Server, Request, Response, Status, Streaming};

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

lazy_static! {
    /// Initial instant.
    static ref INIT : Instant = Instant::now();
}

fn into_aeb_service_input(input: Input) -> Option<RuntimeInput> {
    match input.message {
        Some(Message::PedestrianL(Pedestrian { distance })) => Some(RuntimeInput::PedestrianL(
            distance,
            *INIT + Duration::from_millis(input.timestamp as u64),
        )),
        Some(Message::PedestrianR(Pedestrian { distance })) => Some(RuntimeInput::PedestrianR(
            distance,
            *INIT + Duration::from_millis(input.timestamp as u64),
        )),
        Some(Message::Speed(Speed { value })) => Some(RuntimeInput::SpeedKmH(
            value,
            *INIT + Duration::from_millis(input.timestamp as u64),
        )),
        None => None,
    }
}

fn from_aeb_service_output(output: RuntimeOutput) -> Result<Output, Status> {
    match output {
        RuntimeOutput::Brakes(aeb::Braking::UrgentBrake, instant) => Ok(Output {
            brakes: Braking::UrgentBrake.into(),
            timestamp: instant.duration_since(*INIT).as_millis() as i64,
        }),
        RuntimeOutput::Brakes(aeb::Braking::SoftBrake, instant) => Ok(Output {
            brakes: Braking::SoftBrake.into(),
            timestamp: instant.duration_since(*INIT).as_millis() as i64,
        }),
        RuntimeOutput::Brakes(aeb::Braking::NoBrake, instant) => Ok(Output {
            brakes: Braking::NoBrake.into(),
            timestamp: instant.duration_since(*INIT).as_millis() as i64,
        }),
    }
}

pub struct AebRuntime;

#[tonic::async_trait]
impl Aeb for AebRuntime {
    type RunAEBStream = futures::stream::Map<
        futures::channel::mpsc::Receiver<RuntimeOutput>,
        fn(RuntimeOutput) -> Result<Output, Status>,
    >;

    async fn run_aeb(
        &self,
        request: Request<Streaming<Input>>,
    ) -> Result<Response<Self::RunAEBStream>, Status> {
        let input_stream = request
            .into_inner()
            .filter_map(|input| async { input.map(into_aeb_service_input).ok().flatten() });

        let output_stream = aeb::run(*INIT, input_stream, RuntimeInit { speed_km_h: 50.0 });

        Ok(Response::new(output_stream.map(
            from_aeb_service_output as fn(RuntimeOutput) -> Result<Output, Status>,
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse().unwrap();
    println!("AebServer listening on {}", addr);

    Server::builder()
        .add_service(AebServer::new(AebRuntime))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl_c")
        })
        .await?;

    Ok(())
}
