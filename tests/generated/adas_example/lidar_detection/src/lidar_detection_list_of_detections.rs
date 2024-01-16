use crate::functions::fibonacci;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub struct LidarDetectionListOfDetectionsInput {
    pub point_cloud: [i64; 10],
}
pub struct LidarDetectionListOfDetectionsState {}
impl LidarDetectionListOfDetectionsState {
    pub fn init() -> LidarDetectionListOfDetectionsState {
        LidarDetectionListOfDetectionsState {}
    }
    pub fn step(&mut self, input: LidarDetectionListOfDetectionsInput) -> [i64; 10] {
        let list_of_detections = input
            .point_cloud
            .into_par_iter()
            .map(fibonacci)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        list_of_detections
    }
}
