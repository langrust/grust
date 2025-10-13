use crate::functions::fibonacci;
pub struct LidarDetectionListOfDetectionsInput {
    pub point_cloud: [i64; 10usize],
}
pub struct LidarDetectionListOfDetectionsState {}
impl LidarDetectionListOfDetectionsState {
    pub fn init() -> LidarDetectionListOfDetectionsState {
        LidarDetectionListOfDetectionsState {
        }
    }
    pub fn step(
        &mut self,
        input: LidarDetectionListOfDetectionsInput,
    ) -> [i64; 10usize] {
        let list_of_detections = input.point_cloud.map(fibonacci);
        list_of_detections
    }
}
