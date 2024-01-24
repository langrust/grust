use futures_signals::{
    map_ref,
    signal::{Broadcaster, Signal},
};

use classification::classification_classification::{
    ClassificationClassificationInput, ClassificationClassificationState,
};

use crate::event::SignalEvent;

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
        let rgb_images = rgb_images.event(10, |value| async move { value }),
        let regions_of_interest = regions_of_interest.event(10, |value| async move { value }) => {
            println!("\t\tclassification inputs changed");
            ClassificationClassificationInput {
                rgb_images: *rgb_images,
                regions_of_interest: *regions_of_interest,
            }
        }
    }
    .event(10, move |input| {
        println!("classification!");
        std::thread::sleep(std::time::Duration::from_millis(100));
        let output = state.step(input);
        async move { output }
    });

    Broadcaster::new(classification)
}
