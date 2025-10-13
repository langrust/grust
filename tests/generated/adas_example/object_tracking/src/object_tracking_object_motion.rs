pub struct ObjectTrackingObjectMotionInput {
    pub fused_information: [i64; 10usize],
}
pub struct ObjectTrackingObjectMotionState {}
impl ObjectTrackingObjectMotionState {
    pub fn init() -> ObjectTrackingObjectMotionState {
        ObjectTrackingObjectMotionState {}
    }
    pub fn step(&mut self, input: ObjectTrackingObjectMotionInput) -> [i64; 10usize] {
        let object_motion = {
            let mut x = input.fused_information.clone();
            let slice = x.as_mut();
            slice
                .sort_by(|a, b| {
                    let compare = move |a: i64, b: i64| -> i64 { a - b }(*a, *b);
                    if compare < 0 {
                        std::cmp::Ordering::Less
                    } else if compare > 0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            x
        };
        object_motion
    }
}
