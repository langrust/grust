pub struct ClassificationClassificationInput {
    pub rgb_images: [i64; 10],
    pub regions_of_interest: i64,
}
pub struct ClassificationClassificationState {}
impl ClassificationClassificationState {
    pub fn init() -> ClassificationClassificationState {
        ClassificationClassificationState {}
    }
    pub fn step(&mut self, input: ClassificationClassificationInput) -> [i64; 10] {
        let x = input
            .rgb_images
            .into_iter()
            .fold(input.regions_of_interest, |acc: i64, n: i64| -> i64 {
                acc + n
            });
        let classification = [x, x, x, x, x, x, x, x, x, x];
        classification
    }
}
