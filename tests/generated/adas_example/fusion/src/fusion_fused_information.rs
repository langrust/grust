pub struct FusionFusedInformationInput {
    pub radar_detections: [i64; 10usize],
    pub classification: [i64; 10usize],
    pub lidar_detections: [i64; 10usize],
}
pub struct FusionFusedInformationState {}
impl FusionFusedInformationState {
    pub fn init() -> FusionFusedInformationState {
        FusionFusedInformationState {}
    }
    pub fn step(&mut self, input: FusionFusedInformationInput) -> [i64; 10usize] {
        let fused_information = {
            let mut iter = itertools::izip!(
                input.radar_detections, input.classification, input.lidar_detections
            );
            std::array::from_fn(|_| iter.next().unwrap())
        }
            .map(move |r_c_l: (i64, i64, i64)| -> i64 { r_c_l.0 + r_c_l.1 + r_c_l.2 });
        fused_information
    }
}
