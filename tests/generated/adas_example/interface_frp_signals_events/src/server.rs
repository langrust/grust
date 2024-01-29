use futures_signals::signal::{Mutable, SignalExt};
use interface_grpc_frp_signals::next_interface_client::NextInterfaceClient;
use std::convert::TryInto;
use tonic::transport::Server;
use tonic::{Request, Response};

use crate::event::SignalEvent;
use crate::interface::interface;
use crate::interface_grpc_frp_signals::outputs::MovingObjects;
use crate::interface_grpc_frp_signals::Outputs;

use interface_grpc_frp_signals::interface_server::{Interface, InterfaceServer};
use interface_grpc_frp_signals::{
    inputs::{Distances, PointCloud, RgbImages},
    Empty, Inputs,
};

pub mod interface_grpc_frp_signals {
    tonic::include_proto!("interface");
}
pub mod event;
mod interface;
pub mod shared;
mod signals_system;
pub mod util;

fn convert<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

#[derive(Debug)]
struct InterfaceService {
    distances: Mutable<[i64; 10]>,
    rgb_images: Mutable<[i64; 10]>,
    point_cloud: Mutable<[i64; 10]>,
}

#[tonic::async_trait]
impl Interface for InterfaceService {
    /// Compute according to inputs.
    async fn compute(
        &self,
        request: tonic::Request<Inputs>,
    ) -> std::result::Result<tonic::Response<Empty>, tonic::Status> {
        println!("\n*** REQUEST RECEIVED ***");

        // Set inputs
        let input: &Inputs = request.get_ref();
        if let Some(Distances { distances }) = &input.distances {
            self.distances.set(convert(distances.clone()));
        }
        if let Some(RgbImages { rgb_images }) = &input.rgb_images {
            self.rgb_images.set(convert(rgb_images.clone()));
        }
        if let Some(PointCloud { point_cloud }) = &input.point_cloud {
            self.point_cloud.set(convert(point_cloud.clone()));
        }

        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initiate system
    let distances = Mutable::new([0; 10]);
    let rgb_images = Mutable::new([0; 10]);
    let point_cloud = Mutable::new([0; 10]);
    let interface = interface(
        distances.read_only(),
        rgb_images.read_only(),
        point_cloud.read_only(),
    );

    let handler = tokio::spawn(async {
        let interface = InterfaceService {
            distances,
            rgb_images,
            point_cloud,
        };
        let svc = InterfaceServer::new(interface);
        // let svc = InterfaceServer::with_interceptor(interface, todo!("interceptor"));
        Server::builder()
            .add_service(svc)
            .serve("[::1]:10000".parse().unwrap())
            .await
            .expect("Error: server crashed")
    });

    let client = loop {
        match NextInterfaceClient::connect("http://[::1]:20000").await {
            Ok(client) => break client,
            Err(_) => (),
        }
    };
    println!("*** CONNECTED TO CLIENT ***");

    // Make it into reactive future
    let future = interface
        .signal()
        .event(10, |value| async move {
            println!("\n*** COMPUTED ***");
            value
        })
        .for_each(move |object_motion| {
            let mut client = client.clone();
            async move {
                println!("\n*** RESPONSE SENDED ***");
                client
                    .add_outputs(Request::new(Outputs {
                        moving_objects: Some(MovingObjects {
                            moving_objects: object_motion.to_vec(),
                        }),
                    }))
                    .await
                    .expect("Error: communication with server ended");
            }
        });

    tokio::spawn(future)
        .await
        .expect("Error: reactive interface paniqued");

    handler.await.expect("Error: server paniqued");

    Ok(())
}
