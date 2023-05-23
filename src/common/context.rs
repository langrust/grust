use std::collections::HashMap;

/// HashMap API handling errors.
///
/// [Context] trait is an API handling errors related to HashMap:
/// - [Error::UnknownElement](crate::error::Error::UnknownElement)
pub trait Context {
    /// The type of the elements in the context.
    type Item;
}

impl<V> Context for HashMap<String, V> {
    type Item = V;
}
