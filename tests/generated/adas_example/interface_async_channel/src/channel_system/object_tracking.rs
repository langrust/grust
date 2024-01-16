use object_tracking::object_tracking_object_motion::{
    ObjectTrackingObjectMotionInput, ObjectTrackingObjectMotionState,
};
use tokio::sync::mpsc::Receiver;

use crate::channel_system::Broadcast;

pub struct ObjectTrackingChannel {
    /// Input `fused_information`.
    fused_information: Receiver<[i64; 10]>,
    /// Current state.
    state: ObjectTrackingObjectMotionState,
    /// Output `moving_objects`.
    moving_objects: Broadcast<[i64; 10]>,
}
impl ObjectTrackingChannel {
    pub fn init(
        fused_information: Receiver<[i64; 10]>,
        moving_objects: Broadcast<[i64; 10]>,
    ) -> Self {
        ObjectTrackingChannel {
            fused_information,
            state: ObjectTrackingObjectMotionState::init(),
            moving_objects,
        }
    }
    async fn get_input(&mut self) -> ObjectTrackingObjectMotionInput {
        ObjectTrackingObjectMotionInput {
            fused_information: self.fused_information.recv().await.unwrap(),
        }
    }
    pub async fn job(&'static mut self) {
        let input = self.get_input().await;
        let state = &mut self.state;

        let (send, recv) = tokio::sync::oneshot::channel();

        // Spawn a task on rayon.
        rayon::spawn(move || {
            println!("moving_objects!");
            // Perform an expensive computation.
            let moving_objects = state.step(input);

            // Send the result back to Tokio.
            let _ = send.send(moving_objects);
        });

        // Wait for the rayon task.
        let moving_objects = recv.await.expect("Panic in rayon::spawn");
        self.moving_objects.send(moving_objects).await.unwrap();
    }
}
