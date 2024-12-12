//! [Pattern] module.

prelude! {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// Pattern kind.
pub enum Kind {
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
    /// Event pattern.
    PresentEvent {
        /// The event id.
        event_id: usize,
        /// The pattern matching the event.
        pattern: Box<Pattern>,
    },
    /// NoEvent pattern.
    NoEvent {
        /// The event id.
        event_id: usize,
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

mk_new! { impl Kind =>
    Constant: constant { constant: Constant }
    Identifier: ident { id: usize }
    Structure: structure {
        id: usize,
        fields: Vec<(usize, Option<Pattern>)>,
    }
    Enumeration: enumeration {
        enum_id: usize,
        elem_id: usize,
    }
    PresentEvent: present {
        event_id: usize,
        pattern: Pattern = pattern.into(),
    }
    NoEvent: absent { event_id: usize }
    Tuple: tuple { elements: Vec<Pattern> }
    Some: some { pattern: Pattern = pattern.into() }
    None: none {}
    Default: default {}
}

/// Pattern.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Pattern {
    /// Pattern kind.
    pub kind: Kind,
    /// Pattern type.
    pub typing: Option<Typ>,
    /// Pattern location.
    pub loc: Loc,
}
impl HasLoc for Pattern {
    fn loc(&self) -> Loc {
        self.loc
    }
}

impl Pattern {
    /// Constructs pattern.
    ///
    /// Typing and location are empty.
    pub fn new(loc: impl Into<Loc>, kind: Kind) -> Pattern {
        Pattern {
            kind,
            typing: None,
            loc: loc.into(),
        }
    }

    /// Get pattern's type.
    pub fn get_typ(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
    /// Get pattern's mutable type.
    pub fn get_typ_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
    /// Get pattern's identifiers.
    pub fn identifiers(&self) -> Vec<usize> {
        match &self.kind {
            Kind::Identifier { id } => vec![*id],
            Kind::Constant { .. }
            | Kind::Enumeration { .. }
            | Kind::NoEvent { .. }
            | Kind::None
            | Kind::Default => vec![],
            Kind::Structure { fields, .. } => fields
                .iter()
                .flat_map(|(id, optional_pattern)| {
                    if let Some(pattern) = optional_pattern {
                        pattern.identifiers()
                    } else {
                        vec![*id]
                    }
                })
                .collect(),
            Kind::Tuple { elements } => elements
                .iter()
                .flat_map(|pattern| pattern.identifiers())
                .collect(),
            Kind::Some { pattern } | Kind::PresentEvent { pattern, .. } => pattern.identifiers(),
        }
    }
    /// Get mutable references to pattern's identifiers.
    pub fn identifiers_mut(&mut self) -> Vec<&mut usize> {
        match &mut self.kind {
            Kind::Identifier { id } => vec![id],
            Kind::Constant { .. }
            | Kind::Enumeration { .. }
            | Kind::NoEvent { .. }
            | Kind::None
            | Kind::Default => vec![],
            Kind::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(id, optional_pattern)| {
                    if let Some(pattern) = optional_pattern {
                        pattern.identifiers_mut()
                    } else {
                        vec![id]
                    }
                })
                .collect(),
            Kind::Tuple { elements } => elements
                .iter_mut()
                .flat_map(|pattern| pattern.identifiers_mut())
                .collect(),
            Kind::Some { pattern } | Kind::PresentEvent { pattern, .. } => {
                pattern.identifiers_mut()
            }
        }
    }
}
