use futures::StreamExt;
use interface::{output::Message, sl_client::SlClient, Input, Output};
use json::*;
use lazy_static::lazy_static;
use std::time::Instant;

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

const INPATH: &str = "examples/two_speed_limiters/data/inputs.json";
const OUTPATH: &str = "examples/two_speed_limiters/data/outputs.json";

lazy_static! {
    /// Initial instant.
    static ref INIT : Instant = Instant::now();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // connect to server
    let mut client = SlClient::connect("http://[::1]:50051").await.unwrap();
    println!("\r\nBidirectional stream (kill client with CTLR+C):");
    // read inputs
    let in_stream = futures::stream::iter(read_json(INPATH)).filter_map(
        move |input: Result<Input, _>| async move {
            match input {
                Ok(input) => {
                    let duration = tokio::time::Duration::from_millis(input.timestamp as u64);
                    let deadline = INIT.clone() + duration;
                    tokio::time::sleep_until(deadline.into()).await;
                    Some(input)
                }
                Err(_) => None,
            }
        },
    );
    // ask for SL service
    let response = client.run_sl(in_stream).await.unwrap();
    // initiate outputs file
    begin_json(OUTPATH);
    // collect all outputs
    let mut resp_stream = response.into_inner();
    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();
        let end = received.timestamp > 2000;
        println!("\treceived message: `{:?}`", received.message);
        append_json(OUTPATH, received);
        if end {
            break;
        }
    }
    end_json(OUTPATH);

    // test that every output has its twin
    test_equality_of_outputs();

    Ok(())
}

fn test_equality_of_outputs() {
    let mut prev_timestamp = 0;
    let mut recv_at_timestamp = vec![];
    // test that every output has its twin
    for output in read_json(OUTPATH) {
        let output: Output = output.unwrap();
        if output.timestamp != prev_timestamp {
            prev_timestamp = output.timestamp;
            // test that recv_at_timestamp contains twins of messages
            assert!(contains_twins(&recv_at_timestamp));
            recv_at_timestamp.clear();
        }
        recv_at_timestamp.push(output.message.unwrap());
    }
}

fn contains_twins(v: &Vec<Message>) -> bool {
    let n = v.len();
    if (n % 2) != 0 {
        return false;
    }
    let mut waiting = Vec::with_capacity(n / 2);
    let mut len = 0;
    for output in v {
        // add or remove from waiting output
        if let Some(index) = waiting.iter().position(|elem: &&Message| *elem == output) {
            len -= 1;
            waiting.remove(index);
            println!("match!");
        } else {
            len += 1;
            waiting.push(output);
            println!("waiting!");
        }
        // early exit
        if len > n / 2 {
            return false;
        }
    }
    // no output waits for its twin
    waiting.is_empty()
}
