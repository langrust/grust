//! Shareable stream.

use futures::Stream;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

impl<T: ?Sized> StreamZip for T where T: Stream {}

/// `StreamShared` trait.
pub trait StreamZip: Stream {
    fn zip<S>(self, other: S) -> Zip<Self, S>
    where
        Self: Sized,
        S: Stream + Sized,
    {
        Zip::new(self, other)
    }
}

/// Stream for the `zip` method in the trait `StreamZip`.
#[pin_project(project = ZipProj)]
pub struct Zip<S1: Stream, S2: Stream> {
    #[pin]
    value1: Option<S1::Item>,
    #[pin]
    value2: Option<S2::Item>,
    #[pin]
    stream1: Option<S1>,
    #[pin]
    stream2: Option<S2>,
}

impl<S1: Stream, S2: Stream> Zip<S1, S2> {
    pub fn new(stream1: S1, stream2: S2) -> Self {
        Self {
            value1: None,
            value2: None,
            stream1: Some(stream1),
            stream2: Some(stream2),
        }
    }
}

impl<S1: Stream, S2: Stream> Stream for Zip<S1, S2>
where
    <S1 as Stream>::Item: Clone,
    <S2 as Stream>::Item: Clone,
{
    type Item = (S1::Item, S2::Item);

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> std::task::Poll<std::option::Option<(<S1 as Stream>::Item, <S2 as Stream>::Item)>> {
        let ZipProj {
            mut value1,
            mut value2,
            mut stream1,
            mut stream2,
        } = self.project();

        let new_value1 = match stream1
            .as_mut()
            .as_pin_mut()
            .map(|stream1| stream1.poll_next(cx))
        {
            None => None,
            Some(Poll::Ready(None)) => {
                stream1.set(None);
                None
            }
            Some(Poll::Ready(Some(value))) => {
                value1.set(Some(value.clone()));
                Some(value)
            }
            Some(Poll::Pending) => None,
        };
        let new_value2 = match stream2
            .as_mut()
            .as_pin_mut()
            .map(|stream2| stream2.poll_next(cx))
        {
            None => None,
            Some(Poll::Ready(None)) => {
                stream2.set(None);
                None
            }
            Some(Poll::Ready(Some(value))) => {
                value2.set(Some(value.clone()));
                Some(value)
            }
            Some(Poll::Pending) => None,
        };

        if stream1.is_none() && stream2.is_none() {
            return Poll::Ready(None);
        }

        match (new_value1, new_value2) {
            (None, None) => Poll::Pending,
            (None, Some(new_value2)) => match value1.as_pin_mut() {
                Some(value1) => Poll::Ready(Some((value1.clone(), new_value2))),
                None => Poll::Pending,
            },
            (Some(new_value1), None) => match value2.as_pin_mut() {
                Some(value2) => Poll::Ready(Some((new_value1, value2.clone()))),
                None => Poll::Pending,
            },
            (Some(new_value1), Some(new_value2)) => Poll::Ready(Some((new_value1, new_value2))),
        }
    }
}

#[cfg(test)]
mod zip {
    use futures::{stream, StreamExt};

    use crate::zip::StreamZip;

    #[tokio::test]
    async fn should_be_triggered_when_at_least_one_stream_change() {
        let stream = StreamZip::zip(stream::iter(1..5), stream::once(async { 1 }));

        stream
            .for_each(|(a, b)| async move { println!("zipped: ({a}, {b})") })
            .await;
    }
}
