//! [Stmt] module.

prelude! {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// pattern kind.
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
        typ: Typ,
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
        typ: Typ,
    }
    Tuple: tuple { elements: Vec<Pattern> }
}

/// pattern.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Pattern kind.
    pub kind: Kind,
    /// Pattern type.
    pub typ: Option<Typ>,
    /// Pattern location.
    pub loc: Loc,
}
impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.typ == other.typ
    }
}
impl Eq for Pattern {}
impl Hash for Pattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.typ.hash(state);
    }
}
impl Pattern {
    pub fn of_many(loc: Loc, elements: Vec<Self>) -> Self {
        Self {
            kind: Kind::Tuple { elements },
            typ: None,
            loc,
        }
    }

    /// Constructs pattern.
    ///
    /// Typing and location are empty.
    pub fn new(loc: impl Into<Loc>, kind: Kind) -> Pattern {
        Pattern {
            kind,
            typ: None,
            loc: loc.into(),
        }
    }

    /// Get pattern's type.
    pub fn get_type(&self) -> Option<&Typ> {
        self.typ.as_ref()
    }
    /// Get pattern's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typ.as_mut()
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

#[derive(Debug, Clone)]
/// LanGRust statement.
pub struct Stmt<E> {
    /// Pattern of elements.
    pub pattern: Pattern,
    /// The expression defining the element.
    pub expr: E,
    /// Stmt location.
    pub loc: Loc,
}
impl<E: PartialEq> PartialEq for Stmt<E> {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern && self.expr == other.expr
    }
}

impl<E: synced::HasWeight> synced::HasWeight for Stmt<E> {
    fn weight(&self, wb: &synced::WeightBounds) -> synced::Weight {
        self.expr.weight(wb)
    }
}

impl ir1::stream::Stmt {
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.expr.is_normal_form()
    }
    /// Tell if there is no node application.
    pub fn no_component_application(&self) -> bool {
        self.expr.no_component_application()
    }
}
