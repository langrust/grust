use lidar_detection::{
    lidar_detection_list_of_detections::{
        LidarDetectionListOfDetectionsInput, LidarDetectionListOfDetectionsState,
    },
    lidar_detection_regions_of_interest::{
        LidarDetectionRegionsOfInterestInput, LidarDetectionRegionsOfInterestState,
    },
};
use tokio::sync::mpsc::Receiver;

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
    async fn get_input(&mut self) -> LidarDetectionListOfDetectionsInput {
        LidarDetectionListOfDetectionsInput {
            point_cloud: self.point_cloud.recv().await.unwrap(),
        }
    }
    pub async fn job(&'static mut self) {
        let input = self.get_input().await;
        let state = &mut self.state;

        let (send, recv) = tokio::sync::oneshot::channel();

        // Spawn a task on rayon.
        rayon::spawn(move || {
            println!("lidar_detections!");
            // Perform an expensive computation.
            let lidar_detections = state.step(input);

            // Send the result back to Tokio.
            let _ = send.send(lidar_detections);
        });

        // Wait for the rayon task.
        let lidar_detections = recv.await.expect("Panic in rayon::spawn");
        self.lidar_detections.send(lidar_detections).await.unwrap();
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
    async fn get_input(&mut self) -> LidarDetectionRegionsOfInterestInput {
        LidarDetectionRegionsOfInterestInput {
            point_cloud: self.point_cloud.recv().await.unwrap(),
        }
    }
    pub async fn job(&'static mut self) {
        let input = self.get_input().await;
        let state = &mut self.state;

        let (send, recv) = tokio::sync::oneshot::channel();

        // Spawn a task on rayon.
        rayon::spawn(move || {
            println!("regions_of_interest!");
            // Perform an expensive computation.
            let regions_of_interest = state.step(input);

            // Send the result back to Tokio.
            let _ = send.send(regions_of_interest);
        });

        // Wait for the rayon task.
        let regions_of_interest = recv.await.expect("Panic in rayon::spawn");
        self.regions_of_interest
            .send(regions_of_interest)
            .await
            .unwrap();
    }
}
