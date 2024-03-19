use crate::hir::contract::{Term, TermKind};

impl Term {
    /// Tells if the term contains the identifier.
    pub fn contains_id(&self, id: usize) -> bool {
        match &self.kind {
            TermKind::Unary { term, .. } => term.contains_id(id),
            TermKind::Binary { left, right, .. } => left.contains_id(id) || right.contains_id(id),
            TermKind::Constant { .. } => false,
            TermKind::Identifier { id: other } => id == *other,
        }
    }
}
