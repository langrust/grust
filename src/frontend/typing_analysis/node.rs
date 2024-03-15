use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::node::Node;
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for Node {
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node {
            unscheduled_equations,
            ..
        } = self;

        // type all equations
        unscheduled_equations
            .iter_mut()
            .map(|(_, equation)| equation.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()
    }
}
