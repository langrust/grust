use json::{append_json, begin_json, end_json, read_json};
use serde::{Deserialize, Serialize};
use serde_json::from_value;
use std::path::Path;

use interface::MainState;

mod channel_system;
mod interface;

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
    P: AsRef<Path> + Clone,
{
    println!("launch!");
    // Initiate system
    let mut state = MainState::init();

    // Begin a new JSON file
    begin_json(output_path.clone());

    // Begin reading a JSON file
    let stream = read_json(input_path);

    for value in stream {
        // Get input
        let input: SystemInput = from_value(value.unwrap()).unwrap();

        // Perform computation
        let state =
            unsafe { std::mem::transmute::<&mut MainState, &'static mut MainState>(&mut state) };
        let output: SystemOutput = state.step(input);

        // Append output as 'JSON like' String
        append_json(output_path.clone(), output);
    }

    // End JSON file
    end_json(output_path);
}
