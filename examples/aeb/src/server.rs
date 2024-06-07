mod aeb {
    use grust::grust;

    grust! {
        #![dump = "examples/aeb/src/macro_output.rs"]
        import signal car::speed_km_h                   : float;
        import event  car::detect::left::pedestrian_l   : float;
        import event  car::detect::right::pedestrian_r  : float;
        export signal car::urban::braking::brakes       : Braking;

        // Branking type
        enum Braking {
            UrgentBrake,
            SoftBrake,
            NoBrake,
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

        component braking_state(pedest: float!, speed: float) -> (state: Braking)
            // requires { 0. <= speed && speed < 55. } // urban limit
            // ensures { pedest? => state != NoBrake } // safety
        {
            when {
                d = pedest => {
                    state = brakes(d, speed);
                },
                timeout pedest => {
                    state = Braking::NoBrake;
                },
                otherwise => {
                    state = Braking::NoBrake fby state;
                }
            }
        }

        // AEB service
        let event pedestrian: timeout(float) = timeout(pedestrian_l, 2000);
        brakes = braking_state(pedestrian, speed_km_h);
    }
}

use aeb::toto_service::{TotoService, TotoServiceInput, TotoServiceOutput};
use futures::StreamExt;
use interface::{
    aeb_server::{Aeb, AebServer},
    input::Message,
    Braking, Input, Output, Pedestrian, Speed,
};
use tonic::{transport::Server, Request, Response, Status, Streaming};

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

pub struct AebRuntime {}
impl AebRuntime {
    pub fn init() -> Self {
        AebRuntime {}
    }
}

fn into_toto_service_input(input: Input) -> Option<TotoServiceInput> {
    match input.message {
        Some(Message::PedestrianL(Pedestrian { distance })) => {
            Some(TotoServiceInput::pedestrian_l(distance))
        }
        Some(Message::PedestrianR(Pedestrian { distance })) => {
            Some(TotoServiceInput::pedestrian_r(distance))
        }
        Some(Message::Speed(Speed { value })) => Some(TotoServiceInput::speed_km_h(value)),
        None => None,
    }
}

fn from_toto_service_output(output: TotoServiceOutput) -> Result<Output, Status> {
    match output {
        TotoServiceOutput::brakes(aeb::Braking::UrgentBrake) => Ok(Output {
            brakes: Braking::UrgentBrake.into(),
        }),
        TotoServiceOutput::brakes(aeb::Braking::SoftBrake) => Ok(Output {
            brakes: Braking::SoftBrake.into(),
        }),
        TotoServiceOutput::brakes(aeb::Braking::NoBrake) => Ok(Output {
            brakes: Braking::NoBrake.into(),
        }),
    }
}

#[tonic::async_trait]
impl Aeb for AebRuntime {
    type runStream = futures::stream::Map<
        futures::channel::mpsc::Receiver<TotoServiceOutput>,
        fn(TotoServiceOutput) -> Result<Output, Status>,
    >;

    async fn run(
        &self,
        request: Request<Streaming<Input>>,
    ) -> Result<Response<Self::runStream>, Status> {
        let input_stream = request
            .into_inner()
            .filter_map(|input| async { input.map(into_toto_service_input).ok().flatten() });

        let (output_sender, output_receiver) = futures::channel::mpsc::channel(4);
        let toto_service = TotoService::new(output_sender);

        tokio::spawn(toto_service.run_loop(input_stream));
        let out_stream = output_receiver
            .map(from_toto_service_output as fn(TotoServiceOutput) -> Result<Output, Status>);
        Ok(Response::new(out_stream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let aeb_runtime = AebRuntime::init();

    println!("AebServer listening on {}", addr);

    Server::builder()
        .add_service(AebServer::new(aeb_runtime))
        .serve(addr)
        .await?;

    Ok(())
}
