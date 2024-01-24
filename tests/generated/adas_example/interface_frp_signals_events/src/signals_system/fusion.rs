use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal},
};

use fusion::fusion_fused_information::{FusionFusedInformationInput, FusionFusedInformationState};

use crate::event::SignalEvent;

pub fn fusion_fused_information<A, B, C>(
    radar_detections: A,
    classification: B,
    lidar_detections: C,
    mut state: FusionFusedInformationState,
) -> Broadcaster<impl Signal<Item = [i64; 10]>>
where
    A: Signal<Item = [i64; 10]>,
    B: Signal<Item = [i64; 10]>,
    C: Signal<Item = [i64; 10]>,
{
    let fused_information = map_ref! {
        let radar_detections = radar_detections.event(10, |value| async move { value }),
        let classification = classification.event(10, |value| async move { value }),
        let lidar_detections = lidar_detections.event(10, |value| async move { value }) => {
            println!("\t\tfused_information inputs changed");
            FusionFusedInformationInput { radar_detections: *radar_detections, classification: *classification, lidar_detections: *lidar_detections }
        }
    }
    .event(10, move |input| {
        println!("fusion!");
        std::thread::sleep(std::time::Duration::from_millis(100));
        let output = state.step(input);
        println!("\n{output:?}");
        async move { output }
    });

    Broadcaster::new(fused_information)
}
