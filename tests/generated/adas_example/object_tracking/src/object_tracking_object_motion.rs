pub struct ObjectTrackingObjectMotionInput {
    pub fused_information: [i64; 10],
}
pub struct ObjectTrackingObjectMotionState {}
impl ObjectTrackingObjectMotionState {
    pub fn init() -> ObjectTrackingObjectMotionState {
        ObjectTrackingObjectMotionState {}
    }
    pub fn step(&mut self, input: ObjectTrackingObjectMotionInput) -> [i64; 10] {
        let object_motion = input.fused_information.map(factorial);
        object_motion
    }
}
