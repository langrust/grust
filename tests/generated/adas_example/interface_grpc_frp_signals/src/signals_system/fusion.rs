use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal, SignalExt},
};

use fusion::fusion_fused_information::{FusionFusedInformationInput, FusionFusedInformationState};

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
        radar_detections, classification, lidar_detections => {
            FusionFusedInformationInput { radar_detections: *radar_detections, classification: *classification, lidar_detections: *lidar_detections }
        }
    }.map(move |input| {
        println!("fusion!");
        std::thread::sleep(std::time::Duration::from_millis(100));
        state.step(input)
    });

    fused_information.broadcast()
}
