use crossbeam_channel::Receiver;
use object_tracking::object_tracking_object_motion::{
    ObjectTrackingObjectMotionInput, ObjectTrackingObjectMotionState,
};

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
    fn get_input(&mut self) -> ObjectTrackingObjectMotionInput {
        ObjectTrackingObjectMotionInput {
            fused_information: self.fused_information.recv().unwrap(),
        }
    }
    pub fn job(&mut self) {
        let input = self.get_input();
        let moving_objects = self.state.step(input);
        self.moving_objects.send(moving_objects).unwrap();
    }
}
