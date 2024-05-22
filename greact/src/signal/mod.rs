use std::time::Duration;

pub use filter::filter;
pub use fold::fold;
pub use input::input_mutex;
pub use input_channel::{input_channel, input};
pub use last_filter::last_filter;
pub use map::map;
pub use merge::merge;
pub use sample::sample;
pub use zip::zip;
pub use zip_event::zip_event;

use crate::stream::{
    pull_stream::PullStream, push_stream::PushStream, push_timeout_stream::PushTimeoutStream,
};

pub mod filter;
pub mod fold;
pub mod input;
pub mod input_channel;
pub mod last_filter;
pub mod map;
pub mod merge;
pub mod sample;
pub mod shared;
pub mod zip;
pub mod zip_event;

/// # Signal trait.
///
/// A signal is a continuous value varying over time.
///
/// ## Semantics
///
/// Signals can have two behaviors.
///
/// - Synchronous/Pull-based behavior: the signal's current value must be provided when requested.
/// - Asynchronous/Push-based behavior: the signal's update must be provided when it occurs.
///
/// ## Implementation
///
/// This trait implements signal behaviors in synchronous and asynchronous style.
///
/// - [Signal::pull] : `Signal<V> -> PullStream<Item = V>` transforms the signal in a pull stream.
/// The method `.pick()` on the pull stream returns the current value of the signal.
/// - [Signal::push] : `Signal<V> -> PushStream<Item = V>` transforms the signal in an asynchronous stream.
/// Applying `.next().await`on the stream waits for a new value and returns it.
pub trait Signal<V>
where
    Self::PullStream: PullStream<Item = V> + Sized,
    Self::PushStream: PushStream<Item = V> + Sized,
    Self::PushTimeoutStream: PushTimeoutStream<Item = V> + Sized,
{
    type PullStream;
    type PushStream;
    type PushTimeoutStream;

    fn pull(self) -> Self::PullStream;
    fn push(self) -> Self::PushStream;
    fn push_timeout(self, dur: Duration) -> Self::PushTimeoutStream;
}
