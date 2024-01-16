use crate::par_sort::par_sort;

pub struct ObjectTrackingObjectMotionInput {
    pub fused_information: [i64; 10],
}
pub struct ObjectTrackingObjectMotionState {}
impl ObjectTrackingObjectMotionState {
    pub fn init() -> ObjectTrackingObjectMotionState {
        ObjectTrackingObjectMotionState {}
    }
    pub fn step(&mut self, input: ObjectTrackingObjectMotionInput) -> [i64; 10] {
        let object_motion = par_sort(input.fused_information);
        object_motion
    }
}
