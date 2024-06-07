use crate::aeb::{
    self,
    toto_service::{TotoService, TotoServiceInput, TotoServiceOutput},
};
use crate::interface::{
    aeb_server::{Aeb, AebServer},
    input::Message,
    Braking, Input, Output,
};
use crate::interface::{Pedestrian, Speed};
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{Mutex, RwLock};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};

pub struct AebRuntime {
    input_sender: Sender<TotoServiceInput>,
    output_receiver: Arc<Mutex<Receiver<TotoServiceOutput>>>,
}
impl AebRuntime {
    pub fn init() -> Self {
        let (output_sender, output_receiver) = channel(4);
        let (input_sender, input_receiver) = channel(4);

        tokio::spawn(async move {
            let toto_service = TotoService::new(output_sender);
            toto_service.run_loop(ReceiverStream::new(input_receiver))
        });

        AebRuntime {
            input_sender,
            output_receiver: Arc::new(Mutex::new(output_receiver)),
        }
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

fn from_toto_service_output(output: TotoServiceOutput) -> Output {
    match output {
        TotoServiceOutput::brakes(aeb::Braking::UrgentBrake) => Output {
            brakes: Braking::UrgentBrake.into(),
        },
        TotoServiceOutput::brakes(aeb::Braking::SoftBrake) => Output {
            brakes: Braking::SoftBrake.into(),
        },
        TotoServiceOutput::brakes(aeb::Braking::NoBrake) => Output {
            brakes: Braking::NoBrake.into(),
        },
    }
}

#[tonic::async_trait]
impl Aeb for AebRuntime {
    type runStream = ReceiverStream<Result<Output, Status>>;

    async fn run(
        &self,
        request: Request<Streaming<Input>>,
    ) -> Result<Response<Self::runStream>, Status> {
        let mut input_stream = request.into_inner();
        let (tx, rx) = channel(128);
        tokio::spawn({
            let input_sender = self.input_sender.clone();
            let tx = tx.clone();
            async move {
                for result in input_stream.next().await {
                    match result {
                        Ok(input) => {
                            if let Some(input) = into_toto_service_input(input) {
                                match input_sender.send(input).await {
                                    Ok(_) => (),
                                    Err(_err) => break, // response was droped
                                }
                            }
                        }
                        Err(err) => {
                            match tx.send(Err(err)).await {
                                Ok(_) => (),
                                Err(_err) => break, // response was droped
                            }
                        }
                    }
                }
            }
        });
        tokio::spawn({
            let output_receiver = self.output_receiver.clone();
            async move {
                loop {
                    match output_receiver.lock().await.recv().await {
                        Some(output) => match tx.send(Ok(from_toto_service_output(output))).await {
                            Ok(_) => (),
                            Err(_err) => break, // response was droped
                        },
                        None => match tx.send(Err(Status::aborted("service ended"))).await {
                            Ok(_) => (),
                            Err(_err) => break, // response was droped
                        },
                    }
                }
            }
        });
        let out_stream = ReceiverStream::new(rx);
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
