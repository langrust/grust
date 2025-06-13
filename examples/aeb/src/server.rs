mod aeb {
    use grust::grust;

    grust! {
        #![mode = test]
        import signal car::speed_km_h                   : float;
        import event  car::detect::left::pedestrian_l   : float;
        import event  car::detect::right::pedestrian_r  : float;
        export signal car::urban::braking               : Braking;

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
        function compute_soft_braking_distance(speed: float, acc: float) -> float {
            return speed * speed / (100.0 * acc);
        }

        // determine braking strategy
        function brakes(distance: float, speed: float, acc: float) -> Braking {
            let braking_distance: float = compute_soft_braking_distance(speed, acc);
            let response: Braking = if braking_distance < distance
                                    then Braking::SoftBrake
                                    else Braking::UrgentBrake;
            return response;
        }

        component derive(v_km_h: float, t: float) -> (a_km_h: float) {
            let v: float = v_km_h / 3.6;
            init (t, v) = (0., 0.);
            let dt: float = t - (last t);
            let a: float = when {
                init => 0.,
                dt > 10. => (v - (last v))/dt,
            };
            a_km_h = 3.6 * a;
        }

        component braking_state(pedest: float?, timeout_pedest: unit?, speed: float, acc: float) -> (state: Braking)
            requires { 0. <= speed && speed < 55. } // urban limit
            ensures { when _x = pedest? => state != Braking::NoBrake } // safety
        {
            when {
                init => {
                    state = Braking::NoBrake;
                }
                let d = pedest? => {
                    state = brakes(d, speed, acc);
                }
                let _ = timeout_pedest? => {
                    state = Braking::NoBrake;
                }
            }
        }

        service aeb @ [10, 3000] {
            let event pedestrian: float = merge(pedestrian_l, pedestrian_r);
            let event timeout_pedest: unit = timeout(pedestrian, 2000);
            let signal acc_km_h: float = derive(speed_km_h, time());
            braking = braking_state(pedestrian, timeout_pedest, speed_km_h, acc_km_h);
        }
    }
}

use aeb::runtime::{Runtime, RuntimeInit, RuntimeInput, RuntimeOutput, RuntimeTimer};
use futures::StreamExt;
use interface::{
    aeb_server::{Aeb, AebServer},
    input::Message,
    Braking, Input, Output, Pedestrian, Speed,
};
use lazy_static::lazy_static;
use priority_stream::prio_stream;
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
        RuntimeOutput::Braking(aeb::Braking::UrgentBrake, instant) => Ok(Output {
            brakes: Braking::UrgentBrake.into(),
            timestamp: instant.duration_since(*INIT).as_millis() as i64,
        }),
        RuntimeOutput::Braking(aeb::Braking::SoftBrake, instant) => Ok(Output {
            brakes: Braking::SoftBrake.into(),
            timestamp: instant.duration_since(*INIT).as_millis() as i64,
        }),
        RuntimeOutput::Braking(aeb::Braking::NoBrake, instant) => Ok(Output {
            brakes: Braking::NoBrake.into(),
            timestamp: instant.duration_since(*INIT).as_millis() as i64,
        }),
    }
}

const OUTPUT_CHANNEL_SIZE: usize = 4;
const TIMER_CHANNEL_SIZE: usize = 4;
const PRIO_STREAM_SIZE: usize = 100;

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
        let (output_sink, output_stream) = futures::channel::mpsc::channel(TIMER_CHANNEL_SIZE);
        let (timers_sink, timers_stream) = futures::channel::mpsc::channel(OUTPUT_CHANNEL_SIZE);

        let request_stream = request
            .into_inner()
            .filter_map(|input| async { input.map(into_aeb_service_input).ok().flatten() });
        let timers_stream = timers_stream.map(|(timer, instant): (RuntimeTimer, Instant)| {
            let deadline = instant + timer_stream::Timing::get_duration(&timer);
            RuntimeInput::Timer(timer, deadline)
        });
        let input_stream = prio_stream::<_, _, PRIO_STREAM_SIZE>(
            futures::stream::select(request_stream, timers_stream),
            RuntimeInput::order,
        );

        let aeb_service = Runtime::new(output_sink, timers_sink);
        tokio::spawn(aeb_service.run_loop(
            *INIT,
            input_stream,
            RuntimeInit { speed_km_h: 0.0 },
        ));

        Ok(Response::new(output_stream.map(
            from_aeb_service_output as fn(RuntimeOutput) -> Result<Output, Status>,
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
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
