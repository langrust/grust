//! HIR [Statement](crate::hir::Stmt) module.

prelude! {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// HIR pattern kind.
pub enum Kind {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier.
        id: usize,
    },
    /// Typed pattern.
    Typed {
        /// Identifier.
        id: usize,
        /// The type.
        typing: Typ,
    },
    /// Tuple pattern that matches tuples.
    Tuple {
        /// The elements of the tuple.
        elements: Vec<Pattern>,
    },
}

mk_new! { impl Kind =>
    Identifier: ident { id: usize }
    Typed: typed {
        id: usize,
        typing: Typ,
    }
    Tuple: tuple { elements: Vec<Pattern> }
}

/// HIR pattern.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Pattern {
    /// Pattern kind.
    pub kind: Kind,
    /// Pattern type.
    pub typing: Option<Typ>,
    /// Pattern location.
    pub location: Location,
}

/// Constructs pattern.
///
/// Typing and location are empty.
pub fn init(kind: Kind) -> Pattern {
    Pattern {
        kind,
        typing: None,
        location: Location::default(),
    }
}

impl Pattern {
    /// Get pattern's type.
    pub fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
    /// Get pattern's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
    /// Get pattern's identifiers.
    pub fn identifiers(&self) -> Vec<usize> {
        match &self.kind {
            Kind::Identifier { id } | Kind::Typed { id, .. } => vec![*id],
            Kind::Tuple { elements } => elements
                .iter()
                .flat_map(|pattern| pattern.identifiers())
                .collect(),
        }
    }
    /// Get mutable references to pattern's identifiers.
    pub fn identifiers_mut(&mut self) -> Vec<&mut usize> {
        match &mut self.kind {
            Kind::Identifier { id } | Kind::Typed { id, .. } => vec![id],
            Kind::Tuple { elements } => elements
                .iter_mut()
                .flat_map(|pattern| pattern.identifiers_mut())
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust statement HIR.
pub struct Stmt<E> {
    /// Pattern of elements.
    pub pattern: Pattern,
    /// The expression defining the element.
    pub expression: E,
    /// Stmt location.
    pub location: Location,
}

impl hir::stream::Stmt {
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.expression.is_normal_form()
    }
    /// Tell if there is no node application.
    pub fn no_component_application(&self) -> bool {
        self.expression.no_component_application()
    }
}
