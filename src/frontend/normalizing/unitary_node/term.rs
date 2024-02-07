use crate::hir::{
    signal::Signal,
    term::{Term, TermKind},
};

impl Term {
    pub fn contains_id(&self, id: &String) -> bool {
        match &self.kind {
            TermKind::Binary { left, right, .. } => left.contains_id(id) || right.contains_id(id),
            TermKind::Constant { .. } => false,
            TermKind::Variable {
                signal: Signal { id: other, .. },
            } => id == other,
        }
    }
}
