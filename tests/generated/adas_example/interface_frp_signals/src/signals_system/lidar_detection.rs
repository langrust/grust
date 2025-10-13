use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal, SignalExt},
};

use lidar_detection::{
    lidar_detection_list_of_detections::{
        LidarDetectionListOfDetectionsInput, LidarDetectionListOfDetectionsState,
    },
    lidar_detection_regions_of_interest::{
        LidarDetectionRegionsOfInterestInput, LidarDetectionRegionsOfInterestState,
    },
};

pub fn lidar_detection_list_of_detections<A>(
    point_cloud: A,
    mut state: LidarDetectionListOfDetectionsState,
) -> Broadcaster<impl Signal<Item = [i64; 10]>>
where
    A: Signal<Item = [i64; 10]>,
{
    let list_of_detections = map_ref! {
        point_cloud => {
            LidarDetectionListOfDetectionsInput { point_cloud: *point_cloud }
        }
    }
    .map(move |input| state.step(input));

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
        point_cloud => {
            LidarDetectionRegionsOfInterestInput { point_cloud: *point_cloud }
        }
    }
    .map(move |input| state.step(input));

    Broadcaster::new(regions_of_interest)
}
