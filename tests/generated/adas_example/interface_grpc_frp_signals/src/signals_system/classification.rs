use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal, SignalExt},
};

use classification::classification_classification::{
    ClassificationClassificationInput, ClassificationClassificationState,
};

pub fn classification_classification<A, B>(
    rgb_images: A,
    regions_of_interest: B,
    mut state: ClassificationClassificationState,
) -> Broadcaster<impl Signal<Item = [i64; 10]>>
where
    A: Signal<Item = [i64; 10]>,
    B: Signal<Item = i64>,
{
    let classification = map_ref! {
        rgb_images, regions_of_interest => {
            ClassificationClassificationInput { rgb_images: *rgb_images, regions_of_interest: *regions_of_interest }
        }
    }.map(move |input| {
        println!("classification!");
        state.step(input)
    });

    classification.broadcast()
}
