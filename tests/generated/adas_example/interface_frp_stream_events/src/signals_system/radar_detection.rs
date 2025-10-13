use futures::{Stream, StreamExt};

use radar_detection::radar_detection_list_of_detections::{
    RadarDetectionListOfDetectionsInput, RadarDetectionListOfDetectionsState,
};

use crate::{
    event::StreamEvent,
    shared::{Shared, StreamShared},
};

pub fn radar_detection_list_of_detections<A>(
    distances: A,
    mut state: RadarDetectionListOfDetectionsState,
) -> Shared<impl Stream<Item = [i64; 10]>>
where
    A: Stream<Item = [i64; 10]>,
{
    let list_of_detections = distances
        .map(|distances| {
            RadarDetectionListOfDetectionsInput { distances }
        })
        .event(10, move |input| {
            println!("radar_detection!");
            std::thread::sleep(std::time::Duration::from_millis(400));
            let output = state.step(input);
            async move { output }
        });

    list_of_detections.shared()
}
