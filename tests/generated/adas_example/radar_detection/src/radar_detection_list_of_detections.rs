use crate::functions::factorial;
pub struct RadarDetectionListOfDetectionsInput {
    pub distances: [i64; 10usize],
}
pub struct RadarDetectionListOfDetectionsState {}
impl RadarDetectionListOfDetectionsState {
    pub fn init() -> RadarDetectionListOfDetectionsState {
        RadarDetectionListOfDetectionsState {
        }
    }
    pub fn step(
        &mut self,
        input: RadarDetectionListOfDetectionsInput,
    ) -> [i64; 10usize] {
        let list_of_detections = input.distances.map(factorial);
        list_of_detections
    }
}
