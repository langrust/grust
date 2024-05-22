use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

/// # Push-based stream with a timeout to send frequent outputs.
///
/// Represents the behavior of a push/pull-based data flow with a periodic pull.
///
/// The method [poll_timeout](PushTimeoutStream::poll_timeout) makes it possible to wait the next update.
/// In case nothing was updated for a long time, a periodic timeout will send the output.
///
/// - If it is a signal, repeat the last output.
/// - If it is an event, send an absence of value.
pub trait PushTimeoutStream: Stream {
    /// Attempt to pull out the next update of the stream,
    /// registering the current task for wakeup if the update is not yet available.
    ///
    /// In case nothing was updated for a long time, a periodic timeout will send the output.
    ///
    /// - If it is a signal, repeat the last output.
    /// - If it is an event, send an absence of value.
    fn poll_timeout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item>;
}
