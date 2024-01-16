use crate::functions::factorial;
pub struct LidarDetectionRegionsOfInterestInput {
    pub point_cloud: [i64; 10],
}
pub struct LidarDetectionRegionsOfInterestState {}
impl LidarDetectionRegionsOfInterestState {
    pub fn init() -> LidarDetectionRegionsOfInterestState {
        LidarDetectionRegionsOfInterestState {}
    }
    pub fn step(&mut self, input: LidarDetectionRegionsOfInterestInput) -> i64 {
        let regions_of_interest = input
            .point_cloud
            .into_iter()
            .fold(0i64, |acc: i64, n: i64| -> i64 { acc + factorial(n) });
        regions_of_interest
    }
}
