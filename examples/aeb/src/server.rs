mod aeb {
    use grust::grust;

    grust! {
        #![dump = "examples/aeb/src/macro_output.rs", test]
        import signal car::speed_km_h                   : float;
        import event  car::detect::left::pedestrian_l   : float;
        import event  car::detect::right::pedestrian_r  : float;
        export signal car::urban::braking::brakes       : Braking;

        // Braking type
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
            state = when d = pedest? then brakes(d, speed)
                    timeout Braking::NoBrake otherwise previous_state;
            let previous_state: Braking = Braking::NoBrake fby state;
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

pub struct AebRuntime;

#[tonic::async_trait]
impl Aeb for AebRuntime {
    type RunAEBStream = futures::stream::Map<
        futures::channel::mpsc::Receiver<TotoServiceOutput>,
        fn(TotoServiceOutput) -> Result<Output, Status>,
    >;

    async fn run_aeb(
        &self,
        request: Request<Streaming<Input>>,
    ) -> Result<Response<Self::RunAEBStream>, Status> {
        let input_stream = request
            .into_inner()
            .filter_map(|input| async { input.map(into_toto_service_input).ok().flatten() });
        let timers_stream = timers_stream.map(|(timer, instant): (TotoServiceTimer, Instant)| {
            let deadline = instant + timer_stream::Timing::get_duration(&timer);
            TotoServiceInput::timer(timer, deadline)
        });
        let input_stream = prio_stream::<_, _, 100>(
            futures::stream::select(request_stream, timers_stream),
            TotoServiceInput::order,
        );

        let (sink, output_stream) = futures::channel::mpsc::channel(4);
        let toto_service = TotoService::new(sink);
        tokio::spawn(toto_service.run_loop(input_stream));

        Ok(Response::new(output_stream.map(
            from_toto_service_output as fn(TotoServiceOutput) -> Result<Output, Status>,
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    println!("AebServer listening on {}", addr);

    Server::builder()
        .add_service(AebServer::new(AebRuntime))
        .serve(addr)
        .await?;

    Ok(())
}
