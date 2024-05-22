use std::time::Duration;

pub use filter::filter;
pub use fold::fold;
pub use input::{input_channel, input};
pub use last_filter::last_filter;
pub use map::map;

use crate::stream::{
    pull_stream::PullStream, push_stream::PushStream, push_timeout_stream::PushTimeoutStream,
};

pub mod filter;
pub mod fold;
pub mod input;
pub mod last_filter;
pub mod map;
pub mod merge;

/// # Event trait.
///
/// An event is a flow of one-off data occurring at any time.
///
/// ## Semantics
///
/// Events can have two behaviors.
///
/// - Synchronous/Pull-based behavior: the event's potential value must be provided when requested.
/// - Asynchronous/Push-based behavior: the event's value must be provided when it occurs,
/// a timeout must inform the absence of the event for a period of time.
///
/// ## Implementation
///
/// This trait implements event behaviors in synchronous and asynchronous style.
///
/// - [Event::pull] : `Event<V> -> PullStream<Item = Option<V>>` transforms the event
/// in a pull stream of optional values. The method `.pick()` on the pull stream returns `Some(value)`
/// if the event occured with the value, it returns `None` if no event occured.
/// - [Event::push] : `(Event<V>, Duration) -> PushStream<Item = Option<V>>` transforms the event
/// in an asynchronous stream of optional values. The `Duration` is a timeout: applying `.next().await`
/// on the stream returns `Some(value)` if the event occured before the timeout, it returns `None`
/// if the timeout occured before the event.
pub trait Event<V>
where
    Self::PullStream: PullStream<Item = Option<V>> + Sized,
    Self::PushStream: PushStream<Item = V> + Sized,
    Self::PushTimeoutStream: PushTimeoutStream<Item = Option<V>> + Sized,
{
    type PullStream;
    type PushStream;
    type PushTimeoutStream;

    fn pull(self) -> Self::PullStream;
    fn push(self) -> Self::PushStream;
    fn push_timeout(self, dur: Duration) -> Self::PushTimeoutStream;
}
