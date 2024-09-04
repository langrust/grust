use futures::StreamExt;
use interface::para_client::ParaClient;
use json::*;

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

const INPATH: &str = "examples/para/data/inputs.json";
const OUTPATH: &str = "examples/para/data/outputs.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // connect to server
    let mut client = ParaClient::connect("http://[::1]:50051").await.unwrap();
    println!("\r\nBidirectional stream (kill client with CTLR+C):");
    // read inputs
    let in_stream = futures::stream::iter(read_json(INPATH)).map(Result::unwrap);
    // ask for PARA service
    let response = client.run_para(in_stream).await.unwrap();
    // initiate outputs file
    begin_json(OUTPATH);
    // collect all outputs
    let mut resp_stream = response.into_inner();
    let mut counter = 0;
    while let Some(received) = resp_stream.next().await {
        counter += 1;
        let received = received.unwrap();
        println!("\treceived message: `{}`", received.o1);
        append_json(OUTPATH, received);
        if counter > 10 {
            break;
        }
    }
    end_json(OUTPATH);
    Ok(())
}
