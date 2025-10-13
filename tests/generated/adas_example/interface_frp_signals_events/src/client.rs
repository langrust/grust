use json::{append_json, begin_json, end_json, read_json};
use rand::Rng;
use serde_json::from_value;
use tonic::transport::Server;
use tonic::{Request, Response};

use interface_grpc_frp_signals::interface_client::InterfaceClient;
use interface_grpc_frp_signals::next_interface_server::{NextInterface, NextInterfaceServer};
use interface_grpc_frp_signals::{Empty, Inputs, Outputs};

pub mod interface_grpc_frp_signals {
    tonic::include_proto!("interface");
}

#[derive(Debug)]
struct NextInterfaceService {}

#[tonic::async_trait]
impl NextInterface for NextInterfaceService {
    /// Compute according to inputs.
    async fn add_outputs(
        &self,
        request: tonic::Request<Outputs>,
    ) -> std::result::Result<tonic::Response<Empty>, tonic::Status> {
        let output: &Outputs = request.get_ref();

        println!("\n*** RESPONSE RECEIVED ***");
        // Append output as 'JSON like' String
        append_json(OUTPUT_PATH, output);

        // End JSON file
        end_json(OUTPUT_PATH);

        Ok(Response::new(Empty {}))
    }
}

static INPUT_PATH: &str = "data/inputs_db.json";
static OUTPUT_PATH: &str = "data/outputs_db.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = tokio::spawn(async {
        let svc = NextInterfaceServer::new(NextInterfaceService {});
        // let svc = InterfaceServer::with_interceptor(interface, todo!("interceptor"));
        Server::builder()
            .add_service(svc)
            .serve("[::1]:20000".parse().unwrap())
            .await
            .expect("Error: server crashed")
    });

    let client = loop {
        match InterfaceClient::connect("http://[::1]:10000").await {
            Ok(client) => break client,
            Err(_) => (),
        }
    };

    println!("\n*** START COMPUTATION ***");
    let mut rng = rand::thread_rng();

    // Begin a new JSON file
    begin_json(OUTPUT_PATH);

    // Begin reading a JSON file
    let stream = read_json(INPUT_PATH);

    for value in stream {
        std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(5..1000)));

        // Get input
        let input: Inputs = from_value(value.unwrap()).unwrap();

        // Request computation
        let mut cloned_client = client.clone();
        tokio::spawn(async move {
            println!("\n*** REQUEST SENDED ***");
            cloned_client
                .compute(Request::new(input))
                .await
                .expect("Error: communication with server ended");
        });
    }

    handler.await.expect("Error: client server paniqued");

    Ok(())
}
