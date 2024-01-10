use crate::functions::factorial;
pub struct LidarDetectionRegionsOfInterestInput {
    pub rgb_images: [i64; 10],
}
pub struct LidarDetectionRegionsOfInterestState {}
impl LidarDetectionRegionsOfInterestState {
    pub fn init() -> LidarDetectionRegionsOfInterestState {
        LidarDetectionRegionsOfInterestState {
        }
    }
    pub fn step(&mut self, input: LidarDetectionRegionsOfInterestInput) -> i64 {
        let regions_of_interest = input
            .rgb_images
            .into_iter()
            .fold(0i64, |acc: i64, n: i64| -> i64 { acc + factorial(n) });
        regions_of_interest
    }
}
