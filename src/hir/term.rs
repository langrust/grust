use crate::common::{constant::Constant, location::Location, operator::BinaryOperator};
use crate::hir::signal::Signal;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum TermKind {
    Binary {
        op: BinaryOperator,
        left: Box<Term>,
        right: Box<Term>,
    },
    Constant {
        constant: Constant,
    },
    Variable {
        signal: Signal,
    },
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Term {
    pub kind: TermKind,
    pub location: Location,
}
impl Term {
    fn rename(&mut self, old_signal_id: &String, new_signal: Signal) {
        match &mut self.kind {
            TermKind::Binary { left, right, .. } => {
                left.rename(old_signal_id, new_signal.clone());
                right.rename(old_signal_id, new_signal.clone());
            }
            TermKind::Constant { constant } => (),
            TermKind::Variable { signal } => {
                if &signal.id == old_signal_id {
                    *signal = new_signal;
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Contract {
    pub requires: Vec<Term>,
    pub ensures: Vec<Term>,
    pub invariant: Vec<Term>,
    pub assert: Vec<Term>,
}
impl Contract {
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
        self.assert
            .iter_mut()
            .for_each(|term| term.rename(old_signal_id, new_signal.clone()));
    }
}
