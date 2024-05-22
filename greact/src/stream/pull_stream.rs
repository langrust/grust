/// # Pull-based stream.
///
/// Represents the behavior of a pull-based data flow.
/// The method [pick](PullStream::pick) makes it possible to see the value
/// in the data flow.
///
/// Observing the value in the data flow may have an effect on the stream:
///
///  - In the case of an event, the picked value is removed from the data flow.
///  - If it is a signal, the value is preserved.
pub trait PullStream {
    type Item;

    /// Observes the stream and returns the current value.
    ///
    /// May have an effect on the stream:
    ///
    ///  - In the case of an event, the picked value is removed from the data flow.
    ///  - If it is a signal, the value is preserved.
    fn pick(&mut self) -> Self::Item;
}
