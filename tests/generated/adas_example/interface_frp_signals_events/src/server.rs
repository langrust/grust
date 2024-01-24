use std::convert::TryInto;
use crossbeam_channel::{bounded, Receiver};
use futures_signals::signal::{Mutable, SignalExt};
use tonic::transport::Server;
use tonic::Response;

use crate::interface::interface;

use interface_grpc_frp_signals::interface_server::{Interface, InterfaceServer};
use interface_grpc_frp_signals::{
    inputs::{Distances, PointCloud, RgbImages},
    outputs::MovingObjects,
    Inputs, Outputs,
};

pub mod interface_grpc_frp_signals {
    tonic::include_proto!("interface");
}
pub mod event;
mod interface;
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
    rx: Receiver<[i64; 10]>,
}

#[tonic::async_trait]
impl Interface for InterfaceService {
    /// Compute according to inputs.
    async fn compute(
        &self,
        request: tonic::Request<Inputs>,
    ) -> std::result::Result<tonic::Response<Outputs>, tonic::Status> {
        println!("*** REQUEST RECEIVED ***");

        // Set inputs
        let input: &Inputs = request.get_ref();
        let mut should_compute = false;
        if let Some(Distances { distances }) = &input.distances {
            self.distances.set(convert(distances.clone()));
            should_compute = true;
        }
        if let Some(RgbImages { rgb_images }) = &input.rgb_images {
            self.rgb_images.set(convert(rgb_images.clone()));
            should_compute = true;
        }
        if let Some(PointCloud { point_cloud }) = &input.point_cloud {
            self.point_cloud.set(convert(point_cloud.clone()));
            should_compute = true;
        }

        if should_compute {
            // Get outputs
            let moving_objects = self
                .rx
                .recv()
                .expect("Error: did not manage to receive object_motion");

            Ok(Response::new(Outputs {
                moving_objects: Some(MovingObjects {
                    moving_objects: moving_objects.to_vec(),
                }),
            }))
        } else {
            Ok(Response::new(Outputs {
                moving_objects: None,
            }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initiate system
    let distances = Mutable::new([0; 10]);
    let rgb_images = Mutable::new([0; 10]);
    let point_cloud = Mutable::new([0; 10]);
    let (tx, rx) = bounded(1);
    let interface = interface(
        distances.read_only(),
        rgb_images.read_only(),
        point_cloud.read_only(),
    );

    // Make it into reactive future
    let future = interface.signal().for_each(move |object_motion| {
        println!("*** INTERFACE COMPUTED ***\n");
        tx.send(object_motion)
            .expect("Error: did not manage to send object_motion");
        async {}
    });

    tokio::spawn(future);

    let interface = InterfaceService {
        distances,
        rgb_images,
        point_cloud,
        rx,
    };

    let svc = InterfaceServer::new(interface);
    // let svc = InterfaceServer::with_interceptor(interface, todo!("interceptor"));

    let addr = "[::1]:10000".parse().unwrap();
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
