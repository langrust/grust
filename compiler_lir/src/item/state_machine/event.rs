prelude! {}

/// A node event structure.
#[derive(Debug, PartialEq)]
pub struct Event {
    /// The node's name.
    pub node_name: String,
    /// The event's elements.
    pub elements: Vec<EventElement>,
    /// The event's conversions.
    pub intos: Vec<IntoOtherEvent>,
    /// The event's generic types.
    pub generics: Vec<(String, Typ)>,
}

/// An event element structure.
#[derive(Debug, PartialEq)]
pub enum EventElement {
    InputEvent {
        /// The name of the event.
        identifier: String,
        /// The type of the event.
        r#type: Typ,
    },
    NoEvent,
}

/// An event element structure.
#[derive(Debug, PartialEq)]
pub struct IntoOtherEvent {
    /// The other node's name.
    pub other_node_name: String,
    /// Maps an event from the current node to the other node.
    pub conversions: Vec<(String, String)>,
}
