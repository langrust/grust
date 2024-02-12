use crate::common::{constant::Constant, location::Location, operator::BinaryOperator};
use crate::hir::signal::Signal;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Term's kind.
pub enum TermKind {
    /// Constant term: 3
    Constant {
        /// The constant
        constant: Constant,
    },
    /// Identifier term: x
    Identifier {
        /// The signal corresponding to the identifier
        signal: Signal,
    },
    /// Binary term: x == y
    Binary {
        /// The operator
        op: BinaryOperator,
        /// Left term
        left: Box<Term>,
        /// Right term
        right: Box<Term>,
    },
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Contract's term.
pub struct Term {
    /// The kind of the term
    pub kind: TermKind,
    /// The location in source code
    pub location: Location,
}
impl Term {
    fn rename(&mut self, old_signal_id: &String, new_signal: Signal) {
        match &mut self.kind {
            TermKind::Binary { left, right, .. } => {
                left.rename(old_signal_id, new_signal.clone());
                right.rename(old_signal_id, new_signal.clone());
            }
            TermKind::Constant { .. } => (),
            TermKind::Identifier { signal } => {
                if &signal.id == old_signal_id {
                    *signal = new_signal;
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize)]
/// Contract to prove using Creusot.
pub struct Contract {
    /// Requirements clauses to suppose
    pub requires: Vec<Term>,
    /// Ensures clauses to prove
    pub ensures: Vec<Term>,
    /// Invariant clauses to prove
    pub invariant: Vec<Term>,
}
impl Contract {
    /// Replace identifier occurance by a new signal.
    pub fn rename(&mut self, old_signal_id: &String, new_signal: Signal) {
        self.requires
            .iter_mut()
            .for_each(|term| term.rename(old_signal_id, new_signal.clone()));
        self.ensures
            .iter_mut()
            .for_each(|term| term.rename(old_signal_id, new_signal.clone()));
        self.invariant
            .iter_mut()
            .for_each(|term| term.rename(old_signal_id, new_signal.clone()));
    }
}
