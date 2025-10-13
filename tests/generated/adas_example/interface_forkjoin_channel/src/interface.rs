//! Implementation of the asynchronous scheduling using Rayon.

use crossbeam_channel::Receiver;

use crate::{
    channel_system::{
        classification::ClassificationChannel,
        features_fusion::FeaturesFusionChannel,
        lidar_detection::{LidarDetectionListChannel, LidarDetectionROIChannel},
        object_tracking::ObjectTrackingChannel,
        radar_detection::RadarDetectionChannel,
        Broadcast,
    },
    SystemInput, SystemOutput,
};

pub struct MainState {
    rgb_images: Broadcast<[i64; 10]>,
    point_cloud: Broadcast<[i64; 10]>,
    distances: Broadcast<[i64; 10]>,
    object_tracking: ObjectTrackingChannel,
    features_fusion: FeaturesFusionChannel,
    classification: ClassificationChannel,
    lidar_detection_list: LidarDetectionListChannel,
    lidar_detection_roi: LidarDetectionROIChannel,
    radar_detection: RadarDetectionChannel,
    output: Receiver<[i64; 10]>,
}
impl MainState {
    pub fn init() -> Self {
        println!("init!");
        // Initiate broadcast channels
        let mut moving_objects = Broadcast::new();
        let mut radar_detections = Broadcast::new();
        let mut lidar_detections = Broadcast::new();
        let mut class_information = Broadcast::new();
        let mut fused = Broadcast::new();
        let mut rgb_images = Broadcast::new();
        let mut regions_of_interest = Broadcast::new();
        let mut point_cloud = Broadcast::new();
        let mut distances = Broadcast::new();

        // Create outptu subscriber
        let output = moving_objects.subscribe();

        // Initiate channeled states
        let object_tracking = ObjectTrackingChannel::init(fused.subscribe(), moving_objects);
        let features_fusion = FeaturesFusionChannel::init(
            radar_detections.subscribe(),
            class_information.subscribe(),
            lidar_detections.subscribe(),
            fused,
        );
        let classification = ClassificationChannel::init(
            rgb_images.subscribe(),
            regions_of_interest.subscribe(),
            class_information,
        );
        let lidar_detection_list =
            LidarDetectionListChannel::init(point_cloud.subscribe(), lidar_detections);
        let lidar_detection_roi =
            LidarDetectionROIChannel::init(point_cloud.subscribe(), regions_of_interest);
        let radar_detection = RadarDetectionChannel::init(distances.subscribe(), radar_detections);

        MainState {
            rgb_images,
            point_cloud,
            distances,
            object_tracking,
            features_fusion,
            classification,
            lidar_detection_list,
            lidar_detection_roi,
            radar_detection,
            output,
        }
    }
    pub fn step(&mut self, input: SystemInput) -> SystemOutput {
        println!("step!");
        self.rgb_images.send(input.rgb_images).unwrap();
        self.point_cloud.send(input.point_cloud).unwrap();
        self.distances.send(input.distances).unwrap();

        // transform DAG into fork-join model with rayon
        rayon::join(
            || {
                rayon::join(
                    || {
                        self.radar_detection.job();
                    },
                    || {
                        self.lidar_detection_list.job();
                    },
                );
            },
            || {
                self.lidar_detection_roi.job();
                self.classification.job();
            },
        );

        self.features_fusion.job();
        self.object_tracking.job();

        SystemOutput {
            moving_objects: self.output.recv().unwrap(),
        }
    }
}
