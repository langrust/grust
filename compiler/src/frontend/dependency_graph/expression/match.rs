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
    /// Compute dependencies of a match stream expression.
    pub fn compute_match_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of match are dependencies of matched expression and
            // dependencies of arms (without new signals defined in patterns)
            ExpressionKind::Match {
                expression, arms, ..
            } => {
                // compute arms dependencies
                let mut arms_dependencies = arms
                    .iter()
                    .map(|(pattern, bound, body, arm_expression)| {
                        let mut arm_dependencies = vec![];

                        for statement in body {
                            statement.add_dependencies(
                                graph,
                                symbol_table,
                                processus_manager,
                                nodes_reduced_graphs,
                                errors,
                            )?;

                            let mut more_dependencies =
                                statement.expression.get_dependencies().clone();
                            arm_dependencies.append(&mut more_dependencies);
                        }

                        // get arm expression dependencies
                        arm_expression.compute_dependencies(
                            graph,
                            symbol_table,
                            processus_manager,
                            nodes_reduced_graphs,
                            errors,
                        )?;
                        let mut more_dependencies = arm_expression.get_dependencies().clone();
                        arm_dependencies.append(&mut more_dependencies);

                        // get bound dependencies
                        let mut more_dependencies =
                            bound.as_ref().map_or(Ok(vec![]), |bound_expression| {
                                bound_expression.compute_dependencies(
                                    graph,
                                    symbol_table,
                                    processus_manager,
                                    nodes_reduced_graphs,
                                    errors,
                                )?;

                                Ok(bound_expression.get_dependencies().clone())
                            })?;
                        arm_dependencies.append(&mut more_dependencies);

                        // get local signals defined in pattern
                        let local_signals = pattern.identifiers();

                        // remove pattern-local signals from the dependencies
                        arm_dependencies = arm_dependencies
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal))
                            .collect::<Vec<(usize, Label)>>();

                        // return arm dependencies
                        Ok(arm_dependencies)
                    })
                    .collect::<Result<Vec<Vec<(usize, Label)>>, TerminationError>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(usize, Label)>>();

                // get matched expression dependencies
                expression.compute_dependencies(
                    graph,
                    symbol_table,
                    processus_manager,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut expression_dependencies = expression.get_dependencies().clone();

                arms_dependencies.append(&mut expression_dependencies);
                Ok(arms_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
