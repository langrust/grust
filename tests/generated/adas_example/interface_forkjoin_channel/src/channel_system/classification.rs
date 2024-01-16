use classification::classification_classification::{
    ClassificationClassificationInput, ClassificationClassificationState,
};
use crossbeam_channel::Receiver;

use crate::channel_system::Broadcast;

pub struct ClassificationChannel {
    /// Input `rgb_images`.
    rgb_images: Receiver<[i64; 10]>,
    /// Input `regions_of_interest`.
    regions_of_interest: Receiver<i64>,
    /// Current state.
    state: ClassificationClassificationState,
    /// Output `class_information`.
    class_information: Broadcast<[i64; 10]>,
}
impl ClassificationChannel {
    pub fn init(
        rgb_images: Receiver<[i64; 10]>,
        regions_of_interest: Receiver<i64>,
        class_information: Broadcast<[i64; 10]>,
    ) -> Self {
        ClassificationChannel {
            rgb_images,
            regions_of_interest,
            state: ClassificationClassificationState::init(),
            class_information,
        }
    }
    fn get_input(&mut self) -> ClassificationClassificationInput {
        ClassificationClassificationInput {
            rgb_images: self.rgb_images.recv().unwrap(),
            regions_of_interest: self.regions_of_interest.recv().unwrap(),
        }
    }
    pub fn job(&mut self) {
        let input = self.get_input();
        let class_information = self.state.step(input);
        self.class_information.send(class_information).unwrap();
    }
}
