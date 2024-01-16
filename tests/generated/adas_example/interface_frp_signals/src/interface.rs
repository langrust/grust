use classification::classification_classification::ClassificationClassificationState;
use fusion::fusion_fused_information::FusionFusedInformationState;
use futures_signals::signal::{ReadOnlyMutable, Signal};
use lidar_detection::{
    lidar_detection_list_of_detections::LidarDetectionListOfDetectionsState,
    lidar_detection_regions_of_interest::LidarDetectionRegionsOfInterestState,
};
use object_tracking::object_tracking_object_motion::ObjectTrackingObjectMotionState;
use radar_detection::radar_detection_list_of_detections::RadarDetectionListOfDetectionsState;

use crate::signals_system::{
    classification::classification_classification,
    fusion::fusion_fused_information,
    lidar_detection::{lidar_detection_list_of_detections, lidar_detection_regions_of_interest},
    object_tracking::object_tracking_object_motion,
    radar_detection::radar_detection_list_of_detections,
};

pub fn interface(
    distances: ReadOnlyMutable<[i64; 10]>,
    rgb_images: ReadOnlyMutable<[i64; 10]>,
    point_cloud: ReadOnlyMutable<[i64; 10]>,
) -> impl Signal<Item = [i64; 10]> {
    let radar_detections = radar_detection_list_of_detections(
        distances.signal_cloned(),
        RadarDetectionListOfDetectionsState::init(),
    );
    let lidar_detections = lidar_detection_list_of_detections(
        point_cloud.signal_cloned(),
        LidarDetectionListOfDetectionsState::init(),
    );
    let regions_of_interest = lidar_detection_regions_of_interest(
        point_cloud.signal_cloned(),
        LidarDetectionRegionsOfInterestState::init(),
    );
    let classification = classification_classification(
        rgb_images.signal_cloned(),
        regions_of_interest,
        ClassificationClassificationState::init(),
    );
    let fused_information = fusion_fused_information(
        radar_detections,
        classification,
        lidar_detections,
        FusionFusedInformationState::init(),
    );
    let object_motion =
        object_tracking_object_motion(fused_information, ObjectTrackingObjectMotionState::init());

    object_motion
}
