use petgraph::graphmap::DiGraphMap;

prelude! {
    common::{HashMap, color::Color, label::Label},
    error::{Error, TerminationError},
    hir::{
        expression::ExpressionKind,
        statement::Statement,
        stream_expression::{StreamExpression, StreamExpressionKind},
    },
    symbol_table::SymbolTable,
}

use super::add_edge;

impl Statement<StreamExpression> {
    /// Add direct dependencies of a statement.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn add_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let signals = self.pattern.identifiers();
        for signal in signals {
            self.add_signal_dependencies(
                signal,
                graph,
                symbol_table,
                processus_manager,
                nodes_reduced_graphs,
                errors,
            )?
        }
        Ok(())
    }

    /// Add direct dependencies of a signal.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn add_signal_dependencies(
        &self,
        signal: usize,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Statement {
            expression,
            location,
            ..
        } = self;

        // get signal's color
        let color = processus_manager
            .get_mut(&signal)
            .expect("signal should be in processing manager");

        match color {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                *color = Color::Grey;

                // compute and get dependencies
                if expression.dependencies.get().is_none() {
                    expression.compute_dependencies(
                        graph,
                        symbol_table,
                        processus_manager,
                        nodes_reduced_graphs,
                        errors,
                    )?;
                }

                // add dependencies as graph's edges:
                // s = e depends on s' <=> s -> s'
                expression
                    .get_dependencies()
                    .iter()
                    .for_each(|(id, label)| {
                        // if there was another edge, keep the most important label
                        add_edge(graph, signal, *id, label.clone())
                    });

                // get signal's color
                let color = processus_manager
                    .get_mut(&signal)
                    .expect("signal should be in processing manager");
                // update status: processed
                *color = Color::Black;

                Ok(())
            }
            // if processing: error
            Color::Grey => {
                let error = Error::NotCausalSignal {
                    signal: symbol_table.get_name(signal).clone(),
                    location: location.clone(),
                };
                errors.push(error);
                Err(TerminationError)
            }
            // if processed: nothing to do
            Color::Black => Ok(()),
        }
    }

    pub fn get_identifiers(&self) -> Vec<usize> {
        let mut identifiers = match &self.expression.kind {
            StreamExpressionKind::Expression { expression } => match expression {
                ExpressionKind::Match { arms, .. } => arms
                    .iter()
                    .flat_map(|(pattern, _, statements, _)| {
                        statements
                            .iter()
                            .flat_map(|statement| statement.get_identifiers())
                            .chain(pattern.identifiers())
                    })
                    .collect(),
                _ => vec![],
            },
            _ => vec![],
        };

        identifiers.append(&mut self.pattern.identifiers());
        identifiers
    }
}
