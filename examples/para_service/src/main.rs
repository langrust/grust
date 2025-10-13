#![allow(warnings)]
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

use grust::{
    futures::{self, Stream, StreamExt},
    tokio,
};
use json::*;
use para::runtime::{RuntimeInit, RuntimeInput, RuntimeOutput};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// JSON input type, without timestamp.
#[derive(Deserialize, std::fmt::Debug)]
#[serde(tag = "variant", content = "value")]
pub enum Input {
    E0(i64),
}
impl Input {
    fn into(self, instant: Instant) -> RuntimeInput {
        match self {
            Input::E0(e0) => RuntimeInput::E0(e0, instant),
        }
    }
}

/// JSON output type, without timestamp.
#[derive(Serialize, std::fmt::Debug)]
pub enum Output {
    O1(i64),
}
impl From<RuntimeOutput> for Output {
    fn from(value: RuntimeOutput) -> Self {
        match value {
            RuntimeOutput::O1(o1, _) => Output::O1(o1),
        }
    }
}

#[tokio::main]
async fn main() {
    const INPUT_PATH: &str = "examples/para_service/data/inputs.json";
    const OUTPUT_PATH: &str = "examples/para_service/data/outputs.json";
    let INIT: Instant = Instant::now();

    // read inputs
    let read_stream = futures::stream::iter(read_json(INPUT_PATH));

    // transform in RuntimeInput + sleep
    let input_stream = read_stream.filter_map(move |input: Result<(u64, Input), _>| async move {
        match input {
            Ok((timestamp, input)) => {
                let duration = tokio::time::Duration::from_millis(timestamp as u64);
                let instant = INIT + duration;
                Some(input.into(instant))
            }
            Err(_) => None,
        }
    });

    // initiate JSON file
    begin_json(OUTPUT_PATH);

    // collect N outputs
    const N: usize = 10;
    let mut output_stream = para::run(INIT, input_stream, RuntimeInit {});
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
