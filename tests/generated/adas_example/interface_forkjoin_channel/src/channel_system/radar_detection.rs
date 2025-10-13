use crossbeam_channel::Receiver;
use radar_detection::radar_detection_list_of_detections::{
    RadarDetectionListOfDetectionsInput, RadarDetectionListOfDetectionsState,
};

use crate::channel_system::Broadcast;

pub struct RadarDetectionChannel {
    /// Input `distances`.
    distances: Receiver<[i64; 10]>,
    /// Current state.
    state: RadarDetectionListOfDetectionsState,
    /// Output `list_of_detections`.
    radar_detections: Broadcast<[i64; 10]>,
}
impl RadarDetectionChannel {
    pub fn init(distances: Receiver<[i64; 10]>, radar_detections: Broadcast<[i64; 10]>) -> Self {
        RadarDetectionChannel {
            distances,
            state: RadarDetectionListOfDetectionsState::init(),
            radar_detections,
        }
    }
    fn get_input(&mut self) -> RadarDetectionListOfDetectionsInput {
        RadarDetectionListOfDetectionsInput {
            distances: self.distances.recv().unwrap(),
        }
    }
    pub fn job(&mut self) {
        let input = self.get_input();
        let radar_detections = self.state.step(input);
        self.radar_detections.send(radar_detections).unwrap();
    }
}
