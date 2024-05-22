use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use stream_proc_macros::stream;

trait PushStream: futures::Stream {
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item>;
}

#[pin_project(project = MapPushProj)]
#[stream(push, item = V)]
struct MapPush<U, V, Push, F>
where
    Push: PushStream<Item = U>,
    F: FnMut(U) -> V,
{
    #[pin]
    stream: Push,
    function: F,
}
impl<U, V, Push, F> PushStream for MapPush<U, V, Push, F>
where
    Push: PushStream<Item = U>,
    F: FnMut(U) -> V,
{
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let project = self.project();
        match project.stream.poll_update(cx) {
            Poll::Ready(value) => Poll::Ready((project.function)(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}
