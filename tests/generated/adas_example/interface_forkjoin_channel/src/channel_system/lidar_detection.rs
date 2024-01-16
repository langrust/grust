use crossbeam_channel::Receiver;
use lidar_detection::{
    lidar_detection_list_of_detections::{
        LidarDetectionListOfDetectionsInput, LidarDetectionListOfDetectionsState,
    },
    lidar_detection_regions_of_interest::{
        LidarDetectionRegionsOfInterestInput, LidarDetectionRegionsOfInterestState,
    },
};

use crate::channel_system::Broadcast;

pub struct LidarDetectionListChannel {
    /// Input `point_cloud`.
    point_cloud: Receiver<[i64; 10]>,
    /// Current state.
    state: LidarDetectionListOfDetectionsState,
    /// Output `lidar_detections`.
    lidar_detections: Broadcast<[i64; 10]>,
}
impl LidarDetectionListChannel {
    pub fn init(point_cloud: Receiver<[i64; 10]>, lidar_detections: Broadcast<[i64; 10]>) -> Self {
        LidarDetectionListChannel {
            point_cloud,
            state: LidarDetectionListOfDetectionsState::init(),
            lidar_detections,
        }
    }
    fn get_input(&mut self) -> LidarDetectionListOfDetectionsInput {
        LidarDetectionListOfDetectionsInput {
            point_cloud: self.point_cloud.recv().unwrap(),
        }
    }
    pub fn job(&mut self) {
        let input: LidarDetectionListOfDetectionsInput = self.get_input();
        let lidar_detections = self.state.step(input);
        self.lidar_detections.send(lidar_detections).unwrap();
    }
}

pub struct LidarDetectionROIChannel {
    /// Input `point_cloud`.
    point_cloud: Receiver<[i64; 10]>,
    /// Current state.
    state: LidarDetectionRegionsOfInterestState,
    /// Output `regions_of_interest`.
    regions_of_interest: Broadcast<i64>,
}
impl LidarDetectionROIChannel {
    pub fn init(point_cloud: Receiver<[i64; 10]>, regions_of_interest: Broadcast<i64>) -> Self {
        LidarDetectionROIChannel {
            point_cloud,
            state: LidarDetectionRegionsOfInterestState::init(),
            regions_of_interest,
        }
    }
    fn get_input(&mut self) -> LidarDetectionRegionsOfInterestInput {
        LidarDetectionRegionsOfInterestInput {
            point_cloud: self.point_cloud.recv().unwrap(),
        }
    }
    pub fn job(&mut self) {
        let input = self.get_input();
        let regions_of_interest = self.state.step(input);
        self.regions_of_interest.send(regions_of_interest).unwrap();
    }
}
