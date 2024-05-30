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
    /// Compute dependencies of an tuple stream expression.
    pub fn compute_tuple_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of tuple are dependencies of its elements
            ExpressionKind::Tuple { elements } => {
                // propagate dependencies computation
                elements
                    .iter()
                    .map(|element_expression| {
                        element_expression.compute_dependencies(
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

                Ok(elements
                    .iter()
                    .flat_map(|element_expression| element_expression.get_dependencies().clone())
                    .collect())
            }
            _ => unreachable!(),
        }
    }
}
