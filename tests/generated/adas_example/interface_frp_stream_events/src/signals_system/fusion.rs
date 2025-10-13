use futures::{Stream, StreamExt};

use fusion::fusion_fused_information::{FusionFusedInformationInput, FusionFusedInformationState};

use crate::{
    event::StreamEvent,
    shared::{Shared, StreamShared},
    zip::StreamZip,
};

pub fn fusion_fused_information<A, B, C>(
    radar_detections: A,
    classification: B,
    lidar_detections: C,
    mut state: FusionFusedInformationState,
) -> Shared<impl Stream<Item = [i64; 10]>>
where
    A: Stream<Item = [i64; 10]>,
    B: Stream<Item = [i64; 10]>,
    C: Stream<Item = [i64; 10]>,
{
    let fused_information = StreamZip::zip(
        radar_detections,
        StreamZip::zip(classification, lidar_detections),
    )
    .map(|(radar_detections, (classification, lidar_detections))| {
        FusionFusedInformationInput {
            radar_detections,
            classification,
            lidar_detections,
        }
    })
    .event(10, move |input| {
        println!("fusion!");
        std::thread::sleep(std::time::Duration::from_millis(100));
        let output = state.step(input);
        async move { output }
    });

    fused_information.shared()
}
