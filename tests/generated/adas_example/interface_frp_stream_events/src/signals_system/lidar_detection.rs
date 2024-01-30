use futures::{Stream, StreamExt};

use lidar_detection::{
    lidar_detection_list_of_detections::{
        LidarDetectionListOfDetectionsInput, LidarDetectionListOfDetectionsState,
    },
    lidar_detection_regions_of_interest::{
        LidarDetectionRegionsOfInterestInput, LidarDetectionRegionsOfInterestState,
    },
};

use crate::{
    event::StreamEvent,
    shared::{Shared, StreamShared},
};

pub fn lidar_detection_list_of_detections<A>(
    point_cloud: A,
    mut state: LidarDetectionListOfDetectionsState,
) -> Shared<impl Stream<Item = [i64; 10]>>
where
    A: Stream<Item = [i64; 10]>,
{
    let list_of_detections = point_cloud
        .map(|point_cloud| {
            LidarDetectionListOfDetectionsInput { point_cloud }
        })
        .event(10, move |input| {
            println!("lidar_detection list_of_detections!");
            std::thread::sleep(std::time::Duration::from_millis(300));
            let output = state.step(input);
            async move { output }
        });

    list_of_detections.shared()
}

pub fn lidar_detection_regions_of_interest<A>(
    point_cloud: A,
    mut state: LidarDetectionRegionsOfInterestState,
) -> Shared<impl Stream<Item = i64>>
where
    A: Stream<Item = [i64; 10]>,
{
    let regions_of_interest = point_cloud
        .map(|point_cloud| {
            LidarDetectionRegionsOfInterestInput { point_cloud }
        })
        .event(10, move |input| {
            println!("lidar_detection regions_of_interest!");
            std::thread::sleep(std::time::Duration::from_millis(300));
            let output = state.step(input);
            async move { output }
        });

    regions_of_interest.shared()
}
