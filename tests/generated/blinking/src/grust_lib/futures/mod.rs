/// Asynchronous step as a [Future].
pub mod async_step;
/// A `Map` [Future].
pub mod map;
/// [Future] that might be done.
pub mod maybe_done;
/// Already ready [Future].
pub mod ready;

/// JoinError that can be cloned for shareable future.
#[derive(Clone, Debug)]
pub struct JoinError;
impl JoinError {
    /// Converts any result into a [Result] of error type [JoinError].
    pub fn convert_error<T, E>(result: Result<T, E>) -> Result<T, JoinError> {
        match result {
            Ok(output) => Ok(output),
            Err(_) => Err(JoinError),
        }
    }
}
