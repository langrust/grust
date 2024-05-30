use crate::common::{constant::Constant, location::Location, r#type::Type};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// HIR pattern kind.
pub enum PatternKind {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier.
        id: usize,
    },
    /// Constant pattern, matches le given constant.
    Constant {
        /// The matching constant.
        constant: Constant,
    },
    /// Typed pattern.
    Typed {
        /// The pattern.
        pattern: Box<Pattern>,
        /// The type.
        typing: Type,
    },
    /// Structure pattern that matches the structure and its fields.
    Structure {
        /// The structure id.
        id: usize,
        /// The structure fields with the corresponding patterns to match.
        fields: Vec<(usize, Option<Pattern>)>,
    },
    /// Enumeration pattern.
    Enumeration {
        /// The enumeration type id.
        enum_id: usize,
        /// The element id.
        elem_id: usize,
    },
    /// Event enumeration pattern.
    Event {
        /// The event id.
        event_enum_id: usize,
        /// The event element id.
        event_element_id: usize,
        /// The pattern matching the event.
        pattern: Box<Pattern>,
    },
    /// TimeoutEvent enumeration pattern.
    TimeoutEvent {
        /// The event id.
        event_enum_id: usize,
        /// The event element id.
        event_element_id: usize,
        /// The pattern matching the event.
        pattern: Box<Pattern>,
    },
    /// NoEvent enumeration pattern.
    NoEvent {
        /// The event id.
        event_enum_id: usize,
    },
    /// Tuple pattern that matches tuples.
    Tuple {
        /// The elements of the tuple.
        elements: Vec<Pattern>,
    },
    /// Some pattern that matches when an optional has a value which match the pattern.
    Some {
        /// The pattern matching the value.
        pattern: Box<Pattern>,
    },
    /// None pattern, matches when the optional does not have a value.
    None,
    /// The default pattern that matches anything.
    Default,
}

/// HIR pattern.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Pattern {
    /// Pattern kind.
    pub kind: PatternKind,
    /// Pattern type.
    pub typing: Option<Type>,
    /// Pattern location.
    pub location: Location,
}
impl Pattern {
    /// Get pattern's type.
    pub fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }
    /// Get pattern's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Type> {
        self.typing.as_mut()
    }
    /// Get pattern's identifiers.
    pub fn identifiers(&self) -> Vec<usize> {
        match &self.kind {
            PatternKind::Identifier { id } => vec![*id],
            PatternKind::Constant { .. }
            | PatternKind::Enumeration { .. }
            | PatternKind::NoEvent { .. }
            | PatternKind::None
            | PatternKind::Default => vec![],
            PatternKind::Structure { fields, .. } => fields
                .iter()
                .flat_map(|(id, optional_pattern)| {
                    if let Some(pattern) = optional_pattern {
                        pattern.identifiers()
                    } else {
                        vec![*id]
                    }
                })
                .collect(),
            PatternKind::Tuple { elements } => elements
                .iter()
                .flat_map(|pattern| pattern.identifiers())
                .collect(),
            PatternKind::Some { pattern }
            | PatternKind::Typed { pattern, .. }
            | PatternKind::Event { pattern, .. }
            | PatternKind::TimeoutEvent { pattern, .. } => pattern.identifiers(),
        }
    }
    /// Get mutable references to pattern's identifiers.
    pub fn identifiers_mut(&mut self) -> Vec<&mut usize> {
        match &mut self.kind {
            PatternKind::Identifier { id } => vec![id],
            PatternKind::Constant { .. }
            | PatternKind::Enumeration { .. }
            | PatternKind::NoEvent { .. }
            | PatternKind::None
            | PatternKind::Default => vec![],
            PatternKind::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(id, optional_pattern)| {
                    if let Some(pattern) = optional_pattern {
                        pattern.identifiers_mut()
                    } else {
                        vec![id]
                    }
                })
                .collect(),
            PatternKind::Tuple { elements } => elements
                .iter_mut()
                .flat_map(|pattern| pattern.identifiers_mut())
                .collect(),
            PatternKind::Some { pattern }
            | PatternKind::Typed { pattern, .. }
            | PatternKind::Event { pattern, .. }
            | PatternKind::TimeoutEvent { pattern, .. } => pattern.identifiers_mut(),
        }
    }
}
