use crate::common::location::Location;
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::flow_statement::{
    FlowDeclaration, FlowExport, FlowImport, FlowInstanciation, FlowStatement,
};
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for FlowStatement {
    // precondition: identifiers associated with statement is already typed
    // postcondition: expression associated with statement is typed and checked
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                flow_type,
                flow_expression,
                ..
            }) => {
                flow_expression.typing(symbol_table, errors)?;
                let expression_type = flow_expression.get_type().unwrap();
                expression_type.eq_check(flow_type, Location::default(), errors)
            }
            FlowStatement::Instanciation(FlowInstanciation {
                id,
                flow_expression,
                ..
            }) => {
                flow_expression.typing(symbol_table, errors)?;
                let flow_type = symbol_table.get_type(*id);
                let expression_type = flow_expression.get_type().unwrap();
                expression_type.eq_check(flow_type, Location::default(), errors)
            }
            FlowStatement::Import(FlowImport { .. }) => Ok(()),
            FlowStatement::Export(FlowExport { .. }) => Ok(()),
        }
    }
}
