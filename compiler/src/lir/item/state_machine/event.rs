use crate::common::r#type::Type;

/// A node event structure.
#[derive(Debug, PartialEq)]
pub struct Event {
    /// The node's name.
    pub node_name: String,
    /// The event's elements.
    pub elements: Vec<EventElement>,
    /// The event's generic types.
    pub generics: Vec<(String, Type)>,
}

/// An event element structure.
#[derive(Debug, PartialEq)]
pub enum EventElement {
    InputEvent {
        /// The name of the event.
        identifier: String,
        /// The type of the event.
        r#type: Type,
    },
    NoEvent,
}
