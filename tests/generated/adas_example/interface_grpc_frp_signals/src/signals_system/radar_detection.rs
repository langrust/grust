use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal, SignalExt},
};

use radar_detection::radar_detection_list_of_detections::{
    RadarDetectionListOfDetectionsInput, RadarDetectionListOfDetectionsState,
};

pub fn radar_detection_list_of_detections<A>(
    distances: A,
    mut state: RadarDetectionListOfDetectionsState,
) -> Broadcaster<impl Signal<Item = [i64; 10]>>
where
    A: Signal<Item = [i64; 10]>,
{
    let list_of_detections = map_ref! {
        distances => {
            RadarDetectionListOfDetectionsInput { distances: *distances }
        }
    }
    .map(move |input| {
        println!("radar_detection!");
        state.step(input)
    });

    list_of_detections.broadcast()
}
