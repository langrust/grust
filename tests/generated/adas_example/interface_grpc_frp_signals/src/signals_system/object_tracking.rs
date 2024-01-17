use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal, SignalExt},
};

use object_tracking::object_tracking_object_motion::{
    ObjectTrackingObjectMotionInput, ObjectTrackingObjectMotionState,
};

pub fn object_tracking_object_motion<A>(
    fused_information: A,
    mut state: ObjectTrackingObjectMotionState,
) -> Broadcaster<impl Signal<Item = [i64; 10]>>
where
    A: Signal<Item = [i64; 10]>,
{
    let object_motion = map_ref! {
        fused_information => {
            ObjectTrackingObjectMotionInput { fused_information: *fused_information }
        }
    }
    .map(move |input| {
        println!("object_tracking!");
        state.step(input)
    });

    object_motion.broadcast()
}
