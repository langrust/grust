use futures::{Stream, StreamExt};

use classification::classification_classification::{
    ClassificationClassificationInput, ClassificationClassificationState,
};

use crate::{
    event::StreamEvent,
    shared::{Shared, StreamShared},
    zip::StreamZip,
};

pub fn classification_classification<A, B>(
    rgb_images: A,
    regions_of_interest: B,
    mut state: ClassificationClassificationState,
) -> Shared<impl Stream<Item = [i64; 10]>>
where
    A: Stream<Item = [i64; 10]>,
    B: Stream<Item = i64>,
{
    let classification = StreamZip::zip(rgb_images, regions_of_interest)
        .map(|(rgb_images, regions_of_interest)| {
            ClassificationClassificationInput {
                rgb_images,
                regions_of_interest,
            }
        })
        .event(10, move |input| {
            println!("classification!");
            std::thread::sleep(std::time::Duration::from_millis(100));
            let output = state.step(input);
            async move { output }
        });

    classification.shared()
}
