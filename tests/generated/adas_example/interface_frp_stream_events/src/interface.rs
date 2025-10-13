use classification::classification_classification::ClassificationClassificationState;
use fusion::fusion_fused_information::FusionFusedInformationState;
use futures::Stream;
use lidar_detection::{
    lidar_detection_list_of_detections::LidarDetectionListOfDetectionsState,
    lidar_detection_regions_of_interest::LidarDetectionRegionsOfInterestState,
};
use object_tracking::object_tracking_object_motion::ObjectTrackingObjectMotionState;
use radar_detection::radar_detection_list_of_detections::RadarDetectionListOfDetectionsState;

use crate::{
    shared::Shared,
    signals_system::{
        classification::classification_classification,
        fusion::fusion_fused_information,
        lidar_detection::{
            lidar_detection_list_of_detections, lidar_detection_regions_of_interest,
        },
        object_tracking::object_tracking_object_motion,
        radar_detection::radar_detection_list_of_detections,
    },
};

pub fn interface<A, B, C>(
    distances: Shared<A>,
    rgb_images: Shared<B>,
    point_cloud: Shared<C>,
) -> Shared<impl Stream<Item = [i64; 10]>>
where
    A: Stream<Item = [i64; 10]>,
    B: Stream<Item = [i64; 10]>,
    C: Stream<Item = [i64; 10]>,
{
    let radar_detections =
        radar_detection_list_of_detections(distances, RadarDetectionListOfDetectionsState::init());
    let lidar_detections = lidar_detection_list_of_detections(
        point_cloud.clone(),
        LidarDetectionListOfDetectionsState::init(),
    );
    let regions_of_interest = lidar_detection_regions_of_interest(
        point_cloud,
        LidarDetectionRegionsOfInterestState::init(),
    );
    let classification = classification_classification(
        rgb_images,
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

#[cfg(test)]
mod interface {
    use futures::StreamExt;
    use futures_signals::signal::{Mutable, SignalExt};

    use crate::{interface::interface, shared::StreamShared};

    #[tokio::test]
    async fn should_transform_mutable_into_stream() {
        let mutable = Mutable::new(0);
        let stream = mutable.signal().to_stream();

        let handler =
            tokio::spawn(stream.for_each(|value| async move { println!("received: {value}") }));

        mutable.set(1);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        mutable.set(2);

        drop(mutable);
        handler.await.expect("Error: tokio thread paniqued");
    }

    #[tokio::test]
    async fn should_compute_interface() {
        let distances = Mutable::new([0; 10]);
        let rgb_images = Mutable::new([0; 10]);
        let point_cloud = Mutable::new([0; 10]);
        let interface = interface(
            distances.signal().to_stream().shared(),
            rgb_images.signal().to_stream().shared(),
            point_cloud.signal().to_stream().shared(),
        );

        let handler =
            tokio::spawn(interface.for_each(|value| async move { println!("computed: {value:?}") }));

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        println!("#######################");
        point_cloud.set([1; 10]);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        println!("#######################");
        distances.set([2; 10]);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        println!("#######################");
        distances.set([3; 10]);
        rgb_images.set([2; 10]);
        point_cloud.set([2; 10]);

        drop(distances);
        drop(rgb_images);
        drop(point_cloud);
        handler.await.expect("Error: tokio thread paniqued");
    }
}
