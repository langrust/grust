use crate::functions::factorial;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub struct RadarDetectionListOfDetectionsInput {
    pub distances: [i64; 10],
}
pub struct RadarDetectionListOfDetectionsState {}
impl RadarDetectionListOfDetectionsState {
    pub fn init() -> RadarDetectionListOfDetectionsState {
        RadarDetectionListOfDetectionsState {}
    }
    pub fn step(&mut self, input: RadarDetectionListOfDetectionsInput) -> [i64; 10] {
        let list_of_detections = input
            .distances
            .into_par_iter()
            .map(factorial)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        list_of_detections
    }
}
