use fusion::fusion_fused_information::{FusionFusedInformationInput, FusionFusedInformationState};
use futures::join;
use tokio::sync::mpsc::Receiver;

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
    async fn get_input(&mut self) -> FusionFusedInformationInput {
        let (radar_detections, classification, lidar_detections) = join!(
            async { self.radar_detections.recv().await.unwrap() },
            async { self.classification.recv().await.unwrap() },
            async { self.lidar_detections.recv().await.unwrap() }
        );
        FusionFusedInformationInput {
            radar_detections,
            classification,
            lidar_detections,
        }
    }
    pub async fn job(&'static mut self) {
        let input = self.get_input().await;
        let state = &mut self.state;

        let (send, recv) = tokio::sync::oneshot::channel();

        // Spawn a task on rayon. https://ryhl.io/blog/async-what-is-blocking/
        rayon::spawn(move || {
            println!("fused!");
            // Perform an expensive computation.
            let fused = state.step(input);

            // Send the result back to Tokio.
            let _ = send.send(fused);
        });

        // Wait for the rayon task.
        let fused = recv.await.expect("Panic in rayon::spawn");
        self.fused.send(fused).await.unwrap();
    }
}
