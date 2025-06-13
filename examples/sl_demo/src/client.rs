use futures::StreamExt;
use interface::{sl_client::SlClient, Input};
use json::*;
use lazy_static::lazy_static;
use std::time::Instant;

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

const INPATH: &str = "examples/sl_demo/data/inputs.json";
const OUTPATH: &str = "examples/sl_demo/data/outputs.json";

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
                    let deadline = *INIT + duration;
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
    Ok(())
}
