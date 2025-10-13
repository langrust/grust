use json::{append_json, begin_json, end_json, read_json};
use rand::Rng;
use serde_json::from_value;
use tonic::Request;

use interface_grpc_frp_signals::interface_client::InterfaceClient;
use interface_grpc_frp_signals::{Inputs, Outputs};

pub mod interface_grpc_frp_signals {
    tonic::include_proto!("interface");
}

static INPUT_PATH: &str = "data/inputs_db.json";
static OUTPUT_PATH: &str = "data/outputs_db.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = InterfaceClient::connect("http://[::1]:10000").await?;

    println!("\n*** START COMPUTATION ***");
    let mut rng = rand::thread_rng();

    // Begin a new JSON file
    begin_json(OUTPUT_PATH);

    // Begin reading a JSON file
    let stream = read_json(INPUT_PATH);

    for value in stream {
        std::thread::sleep(std::time::Duration::from_millis(3000));

        // Get input
        let input: Inputs = from_value(value.unwrap()).unwrap();

        // Request computation
        let mut cloned_client = client.clone();
        tokio::spawn(async move {
            let future_response = cloned_client.compute(Request::new(input));
            let response = future_response.await;
            let output: Outputs = response.unwrap().into_inner();
            // Append output as 'JSON like' String
            append_json(OUTPUT_PATH, output);
        });
    }

    // End JSON file
    end_json(OUTPUT_PATH);

    Ok(())
}
