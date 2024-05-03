use crate::ast::contract::{ClauseKind, Contract};
use crate::error::{Error, TerminationError};
use crate::hir::contract::Contract as HIRContract;
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Contract {
    type HIR = HIRContract;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let (requires, ensures, invariant) = self.clauses.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut requires, mut ensures, mut invariant), clause| {
                match clause.kind {
                    ClauseKind::Requires(_) => {
                        requires.push(clause.term.hir_from_ast(symbol_table, errors))
                    }
                    ClauseKind::Ensures(_) => {
                        ensures.push(clause.term.hir_from_ast(symbol_table, errors))
                    }
                    ClauseKind::Invariant(_) => {
                        invariant.push(clause.term.hir_from_ast(symbol_table, errors))
                    }
                    ClauseKind::Assert(_) => todo!(),
                };
                (requires, ensures, invariant)
            },
        );

        Ok(HIRContract {
            requires: requires.into_iter().collect::<Result<Vec<_>, _>>()?,
            ensures: ensures.into_iter().collect::<Result<Vec<_>, _>>()?,
            invariant: invariant.into_iter().collect::<Result<Vec<_>, _>>()?,
        })
    }
}

mod term {
    use crate::ast::contract::{Binary, Implication, Term, Unary};
    use crate::common::location::Location;
    use crate::common::operator::{BinaryOperator, UnaryOperator};
    use crate::error::{Error, TerminationError};
    use crate::hir::contract::{Term as HIRTerm, TermKind};
    use crate::symbol_table::SymbolTable;

    use super::HIRFromAST;

    impl HIRFromAST for Term {
        type HIR = HIRTerm;

        fn hir_from_ast(
            self,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> Result<Self::HIR, TerminationError> {
            let location = Location::default();
            match self {
                Term::Implication(Implication { left, right, .. }) => {
                    let left = Box::new(left.hir_from_ast(symbol_table, errors)?);
                    let right = Box::new(right.hir_from_ast(symbol_table, errors)?);

                    Ok(HIRTerm {
                        kind: TermKind::Binary {
                            op: BinaryOperator::Or,
                            left: Box::new(HIRTerm {
                                kind: TermKind::Binary {
                                    op: BinaryOperator::And,
                                    left: left.clone(),
                                    right,
                                },
                                location: location.clone(),
                            }),
                            right: Box::new(HIRTerm {
                                kind: TermKind::Unary {
                                    op: UnaryOperator::Not,
                                    term: left,
                                },
                                location: location.clone(),
                            }),
                        },
                        location,
                    })
                }
                Term::Unary(Unary { op, term }) => Ok(HIRTerm {
                    kind: TermKind::Unary {
                        op,
                        term: Box::new(term.hir_from_ast(symbol_table, errors)?),
                    },
                    location,
                }),
                Term::Binary(Binary { op, left, right }) => Ok(HIRTerm {
                    kind: TermKind::Binary {
                        op,
                        left: Box::new(left.hir_from_ast(symbol_table, errors)?),
                        right: Box::new(right.hir_from_ast(symbol_table, errors)?),
                    },
                    location,
                }),
                Term::Constant(constant) => Ok(HIRTerm {
                    kind: TermKind::Constant { constant },
                    location,
                }),
                Term::Identifier(ident) => {
                    let name = ident.to_string();
                    let id =
                        symbol_table.get_signal_id(&name, true, Location::default(), errors)?;
                    Ok(HIRTerm {
                        kind: TermKind::Identifier { id },
                        location,
                    })
                }
            }
        }
    }
}
