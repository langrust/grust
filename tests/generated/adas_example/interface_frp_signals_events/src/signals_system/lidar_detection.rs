use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal},
};

use lidar_detection::{
    lidar_detection_list_of_detections::{
        LidarDetectionListOfDetectionsInput, LidarDetectionListOfDetectionsState,
    },
    lidar_detection_regions_of_interest::{
        LidarDetectionRegionsOfInterestInput, LidarDetectionRegionsOfInterestState,
    },
};

use crate::event::SignalEvent;

pub fn lidar_detection_list_of_detections<A>(
    point_cloud: A,
    mut state: LidarDetectionListOfDetectionsState,
) -> Broadcaster<impl Signal<Item = [i64; 10]>>
where
    A: Signal<Item = [i64; 10]>,
{
    let list_of_detections = map_ref! {
        let point_cloud = point_cloud.event(10, |value| async move { value }) => {
            println!("\t\tlidar_detection list inputs changed");
            LidarDetectionListOfDetectionsInput { point_cloud: *point_cloud }
        }
    }
    .event(10, move |input| {
        println!("lidar_detection list_of_detections!");
        std::thread::sleep(std::time::Duration::from_millis(300));
        let output = state.step(input);
        async move { output }
    });

    Broadcaster::new(list_of_detections)
}

pub fn lidar_detection_regions_of_interest<A>(
    point_cloud: A,
    mut state: LidarDetectionRegionsOfInterestState,
) -> Broadcaster<impl Signal<Item = i64>>
where
    A: Signal<Item = [i64; 10]>,
{
    let regions_of_interest = map_ref! {
        let point_cloud = point_cloud.event(10, |value| async move { value }) => {
            println!("\t\tlidar_detection ROI inputs changed");
            LidarDetectionRegionsOfInterestInput { point_cloud: *point_cloud }
        }
    }
    .event(10, move |input| {
        println!("lidar_detection regions_of_interest!");
        std::thread::sleep(std::time::Duration::from_millis(300));
        let output = state.step(input);
        async move { output }
    });

    Broadcaster::new(regions_of_interest)
}
