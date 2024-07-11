prelude! {
    frontend::typing_analysis::TypeAnalysis,
    hir::interface::{ FlowDeclaration, FlowInstantiation, FlowStatement },
}

impl TypeAnalysis for FlowStatement {
    // precondition: identifiers associated with statement is already typed
    // postcondition: expression associated with statement is typed and checked
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                pattern,
                flow_expression,
                ..
            }) => {
                let expected_type = pattern.typing.as_ref().unwrap();
                flow_expression.typing(symbol_table, errors)?;
                let expression_type = flow_expression.get_type().unwrap();
                expression_type.eq_check(expected_type, Location::default(), errors)
            }
            FlowStatement::Instantiation(FlowInstantiation {
                pattern,
                flow_expression,
                ..
            }) => {
                pattern.construct_statement_type(symbol_table, errors)?;
                let expected_type = pattern.typing.as_ref().unwrap();
                flow_expression.typing(symbol_table, errors)?;
                let expression_type = flow_expression.get_type().unwrap();
                expression_type.eq_check(expected_type, Location::default(), errors)
            }
        }
    }
}
