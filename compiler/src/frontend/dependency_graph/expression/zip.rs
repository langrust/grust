use petgraph::graphmap::DiGraphMap;

prelude! {
    common::{
        HashMap,
        color::Color,
        label::Label,
    },
    error::{Error, TerminationError},
    hir::{expression::ExpressionKind, stream_expression::StreamExpression},
    symbol_table::SymbolTable,
}

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a zip stream expression.
    pub fn compute_zip_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of zip are dependencies of its arrays
            ExpressionKind::Zip { arrays, .. } => {
                // propagate dependencies computation
                arrays
                    .iter()
                    .map(|array_expression| {
                        array_expression.compute_dependencies(
                            graph,
                            symbol_table,
                            processus_manager,
                            nodes_reduced_graphs,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                Ok(arrays
                    .iter()
                    .flat_map(|array_expression| array_expression.get_dependencies().clone())
                    .collect())
            }
            _ => unreachable!(),
        }
    }
}
