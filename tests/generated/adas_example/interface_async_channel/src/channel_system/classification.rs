use classification::classification_classification::{
    ClassificationClassificationInput, ClassificationClassificationState,
};
use futures::join;
use tokio::sync::mpsc::Receiver;

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
    async fn get_input(&mut self) -> ClassificationClassificationInput {
        let (rgb_images, regions_of_interest) =
            join!(async { self.rgb_images.recv().await.unwrap() }, async {
                self.regions_of_interest.recv().await.unwrap()
            });
        ClassificationClassificationInput {
            rgb_images,
            regions_of_interest,
        }
    }
    pub async fn job(&'static mut self) {
        let input = self.get_input().await;
        let state = &mut self.state;

        let (send, recv) = tokio::sync::oneshot::channel();

        // Spawn a task on rayon.
        rayon::spawn(move || {
            println!("class_information!");
            // Perform an expensive computation.
            let class_information = state.step(input);

            // Send the result back to Tokio.
            let _ = send.send(class_information);
        });

        // Wait for the rayon task.
        let class_information = recv.await.expect("Panic in rayon::spawn");
        self.class_information
            .send(class_information)
            .await
            .unwrap();
    }
}
