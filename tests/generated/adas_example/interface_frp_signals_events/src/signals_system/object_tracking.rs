use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal},
};

use object_tracking::object_tracking_object_motion::{
    ObjectTrackingObjectMotionInput, ObjectTrackingObjectMotionState,
};

use crate::event::SignalEvent;

pub fn object_tracking_object_motion<A>(
    fused_information: A,
    mut state: ObjectTrackingObjectMotionState,
) -> Broadcaster<impl Signal<Item = [i64; 10]>>
where
    A: Signal<Item = [i64; 10]>,
{
    let object_motion = map_ref! {
        let fused_information = fused_information.event(10, |value| async move { value }) => {
            println!("\t\tobject_tracking inputs changed");
            ObjectTrackingObjectMotionInput { fused_information: *fused_information }
        }
    }
    .event(10, move |input| {
        println!("object_tracking!");
        std::thread::sleep(std::time::Duration::from_millis(100));
        let output = state.step(input);
        async move { output }
    });

    Broadcaster::new(object_motion)
}
