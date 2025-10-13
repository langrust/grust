use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal},
};

use radar_detection::radar_detection_list_of_detections::{
    RadarDetectionListOfDetectionsInput, RadarDetectionListOfDetectionsState,
};

use crate::event::SignalEvent;

pub fn radar_detection_list_of_detections<A>(
    distances: A,
    mut state: RadarDetectionListOfDetectionsState,
) -> Broadcaster<impl Signal<Item = [i64; 10]>>
where
    A: Signal<Item = [i64; 10]>,
{
    let list_of_detections = map_ref! {
        let distances = distances.event(10, |value| async move { value }) => {
            println!("\t\tradar_detection inputs changed");
            RadarDetectionListOfDetectionsInput { distances: *distances }
        }
    }
    .event(10, move |input| {
        println!("radar_detection!");
        std::thread::sleep(std::time::Duration::from_millis(400));
        let output = state.step(input);
        async move { output }
    });

    Broadcaster::new(list_of_detections)
}
