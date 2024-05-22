use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

/// # Push-based stream.
///
/// Represents the behavior of a push-based data flow.
/// The method [poll_update](PushStream::poll_update) makes it possible to wait the next update.
pub trait PushStream: Stream {
    /// Attempt to pull out the next update of the stream,
    /// registering the current task for wakeup if the update is not yet available.
    fn poll_update(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item>;
}
