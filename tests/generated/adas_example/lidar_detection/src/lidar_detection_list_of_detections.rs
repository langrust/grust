pub struct LidarDetectionListOfDetectionsInput {
    pub rgb_images: [i64; 10],
}
pub struct LidarDetectionListOfDetectionsState {}
impl LidarDetectionListOfDetectionsState {
    pub fn init() -> LidarDetectionListOfDetectionsState {
        LidarDetectionListOfDetectionsState {
        }
    }
    pub fn step(&mut self, input: LidarDetectionListOfDetectionsInput) -> [i64; 10] {
        let list_of_detections = input.rgb_images.map(fibonacci);
        list_of_detections
    }
}
