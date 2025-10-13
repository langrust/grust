use radar_detection::radar_detection_list_of_detections::{
    RadarDetectionListOfDetectionsInput, RadarDetectionListOfDetectionsState,
};
use tokio::sync::mpsc::Receiver;

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
    async fn get_input(&mut self) -> RadarDetectionListOfDetectionsInput {
        RadarDetectionListOfDetectionsInput {
            distances: self.distances.recv().await.unwrap(),
        }
    }
    pub async fn job(&'static mut self) {
        let input = self.get_input().await;
        let state = &mut self.state;

        let (send, recv) = tokio::sync::oneshot::channel();

        // Spawn a task on rayon.
        rayon::spawn(move || {
            println!("radar_detections!");
            // Perform an expensive computation.
            let radar_detections = state.step(input);

            // Send the result back to Tokio.
            let _ = send.send(radar_detections);
        });

        // Wait for the rayon task.
        let radar_detections = recv.await.expect("Panic in rayon::spawn");
        self.radar_detections.send(radar_detections).await.unwrap();
    }
}
