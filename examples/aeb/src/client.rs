use crate::json::{append_json, begin_json, end_json, read_json};
use interface::Pedestrian;
use interface::{aeb_client::AebClient, input::Message, Input, Output};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{Mutex, RwLock};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Channel, Request, Response, Status, Streaming};

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

const INPATH: &str = "examples/aeb/data/inputs.json";
const OUTPATH: &str = "examples/aeb/data/outputs.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // connect to server
    let mut client = AebClient::connect("http://[::1]:50051").await.unwrap();
    println!("\r\nBidirectional stream (kill client with CTLR+C):");
    // read inputs
    let in_stream = tokio_stream::iter(read_json(INPATH)).map(Result::unwrap);
    // ask for AEB service
    let response = client.run(in_stream).await.unwrap();
    // initiate outputs file
    begin_json(OUTPATH);
    // collect all outputs
    let mut resp_stream = response.into_inner();
    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();
        println!("\treceived message: `{}`", received.brakes);
        append_json(OUTPATH, received);
    }
    end_json(OUTPATH);
    Ok(())
}
