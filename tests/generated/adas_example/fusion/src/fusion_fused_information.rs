use crate::par_zip;
use rayon::iter::ParallelIterator;

pub struct FusionFusedInformationInput {
    pub radar_detections: [i64; 10],
    pub classification: [i64; 10],
    pub lidar_detections: [i64; 10],
}
pub struct FusionFusedInformationState {}
impl FusionFusedInformationState {
    pub fn init() -> FusionFusedInformationState {
        FusionFusedInformationState {}
    }
    pub fn step(&mut self, input: FusionFusedInformationInput) -> [i64; 10] {
        let fused_information = par_zip!(
            input.radar_detections,
            input.classification,
            input.lidar_detections
        )
        .map(|r_c_l: (i64, i64, i64)| -> i64 { r_c_l.0 + r_c_l.1 + r_c_l.2 })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
        fused_information
    }
}
