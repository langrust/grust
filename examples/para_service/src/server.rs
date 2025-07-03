/// The system implements the following graph:
///
/// ```text
///        |    |--s2-->| C3 |--e2-->| C4 |--s4-->|    |
/// --e0-->| C1 |                                 |    |
///        |    |--e1-->|    |--------s3--------->| C5 |--o-->
///                     | C2 |                    |    |
///                     |    |--------e3--------->|    |
/// ```
mod para {
    use grust::grust;

    grust! {
        #![service_para, mode = test]
        import event e0: int;
        export signal o1: int;

        component C1(e0: int?) -> (s2: int, e1: int?) {
            when {
                init => {
                    s2 = 0;
                }
                e0? if e0 > prev_s2 => {
                    s2 = e0;
                    e1 = emit e0 / (e0 - prev_s2);
                }
                e0? => {
                    s2 = e0;
                }
            }
            let prev_s2: int = last s2;
        }

        component C2(e1: int?) -> (s3: int, e3: int?) {
            when {
                init => {
                    s3 = 0;
                }
                e1? if e1 > 1 => {
                    s3 = e1;
                    e3 = emit (last s3);
                }
                e1? => {
                    s3 = e1;
                }
            }
        }

        component C3(s2: int) -> (e2: int?) {
            e2 = when { s2 > 1 => emit s2 };
        }

        component C4(e2: int?) -> (s4: int) {
            s4 = when { init => 0, e2? => e2 };
        }

        component C5(s4: int, s3: int, e3: int?) -> (o: int) {
            o = when {
                init => 0,
                e3? => e3,
                s4 > 0 => s4*2,
                s3 >= 0 => s3,
            };
        }

        service para_mess @ [10, 3000] {
            let (signal s2: int, event e1: int) = C1(e0);
            let (signal s3: int, event e3: int) = C2(e1);
            let event e2: int = C3(s2);
            let signal s4: int = C4(e2);
            o1 = C5(s4, s3, e3);
        }
    }
}

use futures::StreamExt;
use interface::{
    output::Message,
    para_server::{Para, ParaServer},
    Input, Output,
};
use lazy_static::lazy_static;
use para::runtime::{RuntimeInit, RuntimeInput, RuntimeOutput};
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

fn into_para_service_input(input: Input) -> Option<RuntimeInput> {
    let Input { timestamp, e0 } = input;
    Some(RuntimeInput::E0(
        e0,
        *INIT + Duration::from_millis(timestamp as u64),
    ))
}

fn from_para_service_output(output: RuntimeOutput) -> Result<Output, Status> {
    match output {
        RuntimeOutput::O1(o1, instant) => Ok(Output {
            timestamp: instant.duration_since(*INIT).as_millis() as i64,
            message: Some(Message::O1(o1)),
        }),
    }
}

pub struct ParaRuntime;

#[tonic::async_trait]
impl Para for ParaRuntime {
    type RunPARAStream = futures::stream::Map<
        futures::channel::mpsc::Receiver<RuntimeOutput>,
        fn(RuntimeOutput) -> Result<Output, Status>,
    >;

    async fn run_para(
        &self,
        request: Request<Streaming<Input>>,
    ) -> Result<Response<Self::RunPARAStream>, Status> {
        // make the server wait for the client to load the inputs (this is because the test mode
        // load all values in priority stream ad it is possible that the timers are created before
        // the inputs, which cause issues)
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let input_stream = request
            .into_inner()
            .filter_map(|input| async { input.map(into_para_service_input).ok().flatten() });

        let output_stream = para::run(*INIT, input_stream, RuntimeInit {});

        Ok(Response::new(output_stream.map(
            from_para_service_output as fn(RuntimeOutput) -> Result<Output, Status>,
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    println!("ParaServer listening on {}", addr);

    Server::builder()
        .add_service(ParaServer::new(ParaRuntime))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl_c")
        })
        .await?;

    Ok(())
}
