use futures::{Stream, StreamExt};

use object_tracking::object_tracking_object_motion::{
    ObjectTrackingObjectMotionInput, ObjectTrackingObjectMotionState,
};

use crate::{
    event::StreamEvent,
    shared::{Shared, StreamShared},
};

pub fn object_tracking_object_motion<A>(
    fused_information: A,
    mut state: ObjectTrackingObjectMotionState,
) -> Shared<impl Stream<Item = [i64; 10]>>
where
    A: Stream<Item = [i64; 10]>,
{
    let object_motion = fused_information
        .map(|fused_information| {
            ObjectTrackingObjectMotionInput { fused_information }
        })
        .event(10, move |input| {
            println!("object_tracking!");
            std::thread::sleep(std::time::Duration::from_millis(100));
            let output = state.step(input);
            async move { output }
        });

    object_motion.shared()
}
