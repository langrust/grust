use futures_signals::signal::{Mutable, SignalExt};
use json::{append_json, begin_json, end_json, read_json};
use serde::{Deserialize, Serialize};
use serde_json::from_value;
use std::path::Path;

use crate::interface::interface;

pub mod event;
mod interface;
mod signals_system;
pub mod util;

/// Input of the system.
#[derive(Deserialize)]
pub struct SystemInput {
    /// Input `distances`.
    pub distances: [i64; 10],
    /// Input `point_cloud`.
    pub point_cloud: [i64; 10],
    /// Input `rgb_images`.
    pub rgb_images: [i64; 10],
}

/// Output of the system
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SystemOutput {
    /// Output `moving_objects`.
    pub moving_objects: [i64; 10],
}

/// Launch the example test.
pub fn launch<P>(input_path: P, output_path: P)
where
    P: AsRef<Path> + Clone + Sync + Send + 'static,
{
    println!("launch!");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Begin a new JSON file
            begin_json(output_path.clone());

            // Begin reading a JSON file
            let mut stream = read_json(input_path);

            // Get first input
            let value = stream.next().unwrap().unwrap();
            let input: SystemInput = from_value(value).unwrap();

            // Initiate system
            let distances = Mutable::new(input.distances);
            let rgb_images = Mutable::new(input.rgb_images);
            let point_cloud = Mutable::new(input.point_cloud);
            let interface = interface(
                distances.read_only(),
                rgb_images.read_only(),
                point_cloud.read_only(),
            );

            // Make it into reactive future
            let cloned_output_path = output_path.clone();
            let future = interface.signal().for_each(move |object_motion| {
                // Perform computation
                let output = SystemOutput {
                    moving_objects: object_motion,
                };

                // Append output as 'JSON like' String
                append_json(cloned_output_path.clone(), output);

                async {}
            });

            tokio::spawn(future);

            for value in stream {
                std::thread::sleep(std::time::Duration::from_millis(100));

                // Get input
                let input: SystemInput = from_value(value.unwrap()).unwrap();
                distances.set(input.distances);
                rgb_images.set(input.rgb_images);
                point_cloud.set(input.point_cloud);
            }

            // End JSON file
            end_json(output_path);
        });
}
