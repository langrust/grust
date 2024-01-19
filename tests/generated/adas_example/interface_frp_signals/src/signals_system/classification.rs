use std::task::Poll;

use futures_signals::{
    internal::{MapRef1, MapRefSignal},
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
    let classification = {
        let mut rgb_images = MapRef1::new(rgb_images);
        let mut regions_of_interest = MapRef1::new(regions_of_interest);

        MapRefSignal::new(move |cx| {
            let mut rgb_images = rgb_images.unsafe_pin();
            let mut regions_of_interest = regions_of_interest.unsafe_pin();

            let result = rgb_images
                .as_mut()
                .poll(cx)
                .merge(regions_of_interest.as_mut().poll(cx));

            if result.changed {
                let rgb_images = rgb_images.value_ref();
                let regions_of_interest = regions_of_interest.value_ref();
                Poll::Ready(Some({
                    ClassificationClassificationInput {
                        rgb_images: *rgb_images,
                        regions_of_interest: *regions_of_interest,
                    }
                }))
            } else if result.done {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        })
    }
    .map(move |input| state.step(input));

    Broadcaster::new(classification)
}
