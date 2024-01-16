use crossbeam_channel::Receiver;
use fusion::fusion_fused_information::{FusionFusedInformationInput, FusionFusedInformationState};

use crate::channel_system::Broadcast;

pub struct FeaturesFusionChannel {
    /// Input `radar_detections`.
    radar_detections: Receiver<[i64; 10]>,
    /// Input `classification`.
    classification: Receiver<[i64; 10]>,
    /// Input `lidar_detections`.
    lidar_detections: Receiver<[i64; 10]>,
    /// Current state.
    state: FusionFusedInformationState,
    /// Output `fused`.
    fused: Broadcast<[i64; 10]>,
}
impl FeaturesFusionChannel {
    pub fn init(
        radar_detections: Receiver<[i64; 10]>,
        classification: Receiver<[i64; 10]>,
        lidar_detections: Receiver<[i64; 10]>,
        fused: Broadcast<[i64; 10]>,
    ) -> Self {
        FeaturesFusionChannel {
            radar_detections,
            classification,
            lidar_detections,
            state: FusionFusedInformationState::init(),
            fused,
        }
    }
    fn get_input(&mut self) -> FusionFusedInformationInput {
        FusionFusedInformationInput {
            radar_detections: self.radar_detections.recv().unwrap(),
            classification: self.classification.recv().unwrap(),
            lidar_detections: self.lidar_detections.recv().unwrap(),
        }
    }
    pub fn job(&mut self) {
        let input = self.get_input();
        let fused = self.state.step(input);
        self.fused.send(fused).unwrap();
    }
}
