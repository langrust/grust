/// The system implements the following graph:
///        |    |--s2-->| C3 |--e2-->| C4 |--s4-->|    |
/// --e0-->| C1 |                                 |    |
///        |    |--e1-->|    |--------s3--------->| C5 |--o-->
///                     | C2 |                    |    |
///                     |    |--------e3--------->|    |
mod para {
    use grust::grust;

    grust! {
        #![dump = "examples/para/src/macro_output.rs", propag = "onchange", para, test]
        import event e0: int;
        export event e1: int;
        export event e2: int;
        export event e3: int;
        export signal s2: int;
        export signal s3: int;
        export signal s4: int;
        export signal o1: int;

        component C1(e0: int?) -> (s2: int, e1: int?) {
            when {
                e0? => {
                    s2 = e0;
                    let x: bool = e0 > prev_s2;
                    e1 = when x then emit e0 / (e0 - prev_s2);
                }
            }
            let prev_s2: int = last s2;
        }

        component C2(e1: int?) -> (s3: int, e3: int?) {
            when {
                e1? => {
                    s3 = e1;
                }
                prev_s3 > 0 => {
                    s3 = prev_s3;
                    e3 = emit prev_s3;
                }
            }
            let prev_s3: int = last s3;
        }

        component C3(s2: int) -> (e2: int?) {
            e2 = when s2 > 1 then emit s2;
        }

        component C4(e2: int?) -> (s4: int) {
            s4 = when e2? then e2;
        }

        component C5(s4: int, s3: int, e3: int?) -> (o: int) {
            when {
                e3? => {
                    o = e3;
                }
                s4 > 0 => {
                    o = s4*2;
                }
                s3 >= 0 => {
                    o = s3;
                }
            }
            let prev_o: int = last o;
        }

        service para_mess @ [10, 3000] {
            (s2, e1) = C1(e0);
            (s3, e3) = C2(e1);
            e2 = C3(s2);
            s4 = C4(e2);
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
use para::runtime::{Runtime, RuntimeInput, RuntimeOutput, RuntimeTimer};
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

fn into_para_service_input(input: Input) -> Option<RuntimeInput> {
    let Input { timestamp, e0 } = input;
    Some(RuntimeInput::E0(
        e0,
        INIT.clone() + Duration::from_millis(timestamp as u64),
    ))
}

fn from_para_service_output(output: RuntimeOutput) -> Result<Output, Status> {
    match output {
        RuntimeOutput::S3(s3, instant) => Ok(Output {
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
            message: Some(Message::S3(s3)),
        }),
        RuntimeOutput::E2(e2, instant) => Ok(Output {
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
            message: Some(Message::E2(e2)),
        }),
        RuntimeOutput::S4(s4, instant) => Ok(Output {
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
            message: Some(Message::S4(s4)),
        }),
        RuntimeOutput::E3(e3, instant) => Ok(Output {
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
            message: Some(Message::E3(e3)),
        }),
        RuntimeOutput::O1(o1, instant) => Ok(Output {
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
            message: Some(Message::O1(o1)),
        }),
        RuntimeOutput::S2(s2, instant) => Ok(Output {
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
            message: Some(Message::S2(s2)),
        }),
        RuntimeOutput::E1(e1, instant) => Ok(Output {
            timestamp: instant.duration_since(INIT.clone()).as_millis() as i64,
            message: Some(Message::E1(e1)),
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
        let (timers_sink, timers_stream) = futures::channel::mpsc::channel(4);
        let (output_sink, output_stream) = futures::channel::mpsc::channel(4);

        let request_stream = request
            .into_inner()
            .filter_map(|input| async { input.map(into_para_service_input).ok().flatten() });
        let timers_stream = timers_stream.map(|(timer, instant): (RuntimeTimer, Instant)| {
            let deadline = instant + timer_stream::Timing::get_duration(&timer);
            RuntimeInput::Timer(timer, deadline)
        });
        let input_stream = prio_stream::<_, _, 100>(
            futures::stream::select(request_stream, timers_stream),
            RuntimeInput::order,
        );

        let para_service = Runtime::new(output_sink, timers_sink);
        tokio::spawn(para_service.run_loop(INIT.clone(), input_stream));

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
