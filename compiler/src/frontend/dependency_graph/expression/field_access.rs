use petgraph::graphmap::DiGraphMap;

prelude! {
    common::{
        color::Color,
        label::Label,
    },
    error::{Error, TerminationError},
    hir::{expression::ExpressionKind, stream_expression::StreamExpression},
    symbol_table::SymbolTable,
}

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a field access stream expression.
    pub fn compute_field_access_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of field access are dependencies of the accessed expression
            ExpressionKind::FieldAccess { expression, .. } => {
                // get accessed expression dependencies
                expression.compute_dependencies(
                    graph,
                    symbol_table,
                    processus_manager,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let expression_dependencies = expression.get_dependencies().clone();

                Ok(expression_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
