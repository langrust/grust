use std::collections::BTreeMap;

use petgraph::graphmap::DiGraphMap;

use crate::{
    common::graph::neighbor::Label,
    hir::{
        dependencies::Dependencies, expression::ExpressionKind,
        identifier_creator::IdentifierCreator, memory::Memory, statement::Statement,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    },
    symbol_table::SymbolTable,
};

use super::Union;

impl ExpressionKind<StreamExpression> {
    /// Replace identifier occurence by element in context.
    ///
    /// It will modify the expression according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` will become
    /// `a + b/2`.
    pub fn replace_by_context(
        &mut self,
        dependencies: &mut Dependencies,
        context_map: &BTreeMap<usize, Union<usize, StreamExpression>>,
    ) -> Option<StreamExpression> {
        match self {
            ExpressionKind::Constant { .. }
            | ExpressionKind::Abstraction { .. }
            | ExpressionKind::Enumeration { .. } => None,
            ExpressionKind::Identifier { ref mut id } => {
                if let Some(element) = context_map.get(id) {
                    match element {
                        Union::I1(new_id) => {
                            *id = *new_id;
                            *dependencies = Dependencies::from(vec![(*new_id, 0)]);
                            None
                        }
                        Union::I2(new_expression) => Some(new_expression.clone()),
                    }
                } else {
                    None
                }
            }
            ExpressionKind::Application { ref mut inputs, .. } => {
                inputs
                    .iter_mut()
                    .for_each(|expression| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            ExpressionKind::Structure { ref mut fields, .. } => {
                fields
                    .iter_mut()
                    .for_each(|(_, expression)| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    fields
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            ExpressionKind::Array {
                ref mut elements, ..
            } => {
                elements
                    .iter_mut()
                    .for_each(|expression| expression.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    elements
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );
                None
            }
            ExpressionKind::Match {
                ref mut expression,
                ref mut arms,
                ..
            } => {
                expression.replace_by_context(context_map);
                let mut expression_dependencies = expression.get_dependencies().clone();

                arms.iter_mut()
                    .for_each(|(pattern, bound, body, matched_expression)| {
                        let local_signals = pattern.local_identifiers();

                        // remove identifiers created by the pattern from the context
                        let context_map = context_map
                            .clone()
                            .into_iter()
                            .filter(|(key, _)| !local_signals.contains(key))
                            .collect();

                        if let Some(expression) = bound.as_mut() {
                            expression.replace_by_context(&context_map);
                            let mut bound_dependencies = expression
                                .get_dependencies()
                                .clone()
                                .into_iter()
                                .filter(|(signal, _)| !local_signals.contains(signal))
                                .collect();
                            expression_dependencies.append(&mut bound_dependencies);
                        };

                        debug_assert!(body.is_empty());
                        // body.iter_mut().for_each(|statements| {
                        //     statements.expression.replace_by_context(&context_map)
                        // });

                        matched_expression.replace_by_context(&context_map);
                        let mut matched_expression_dependencies = matched_expression
                            .get_dependencies()
                            .clone()
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal))
                            .collect::<Vec<(usize, usize)>>();
                        expression_dependencies.append(&mut matched_expression_dependencies);
                    });

                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::When {
                ref mut option,
                ref mut present_body,
                ref mut present,
                ref mut default_body,
                ref mut default,
                ..
            } => {
                option.replace_by_context(context_map);
                let mut option_dependencies = option.get_dependencies().clone();

                debug_assert!(present_body.is_empty());
                // present_body
                //     .iter_mut()
                //     .for_each(|statements| statements.expression.replace_by_context(context_map));

                present.replace_by_context(context_map);
                let mut present_dependencies = present.get_dependencies().clone();

                debug_assert!(default_body.is_empty());
                // default_body
                //     .iter_mut()
                //     .for_each(|statements| statements.expression.replace_by_context(context_map));

                default.replace_by_context(context_map);
                let mut default_dependencies = default.get_dependencies().clone();

                option_dependencies.append(&mut present_dependencies);
                option_dependencies.append(&mut default_dependencies);
                *dependencies = Dependencies::from(option_dependencies);
                None
            }
            ExpressionKind::FieldAccess {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::TupleElementAccess {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Map {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Fold {
                ref mut expression,
                ref mut initialization_expression,
                ..
            } => {
                expression.replace_by_context(context_map);
                initialization_expression.replace_by_context(context_map);
                // get matched expressions dependencies
                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut initialization_expression_dependencies =
                    expression.get_dependencies().clone();
                expression_dependencies.append(&mut initialization_expression_dependencies);

                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Sort {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);
                // get matched expression dependencies
                let expression_dependencies = expression.get_dependencies().clone();
                // push all dependencies in arms dependencies
                *dependencies = Dependencies::from(expression_dependencies);
                None
            }
            ExpressionKind::Zip { ref mut arrays, .. } => {
                arrays
                    .iter_mut()
                    .for_each(|array| array.replace_by_context(context_map));

                *dependencies = Dependencies::from(
                    arrays
                        .iter()
                        .flat_map(|array| array.get_dependencies().clone())
                        .collect(),
                );
                None
            }
        }
    }

    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// ```
    /// In this example, if an expression `semi_fib(fib).o` is assigned to the
    /// signal `fib` there is no causality loop.
    /// But we need to inline the code, a function can not compute an output
    /// before knowing the input.
    pub fn inline_when_needed(
        &mut self,
        signal_id: usize,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        graph: &DiGraphMap<usize, Label>,
        symbol_table: &mut SymbolTable,
        unitary_nodes: &BTreeMap<usize, UnitaryNode>,
    ) -> Vec<Statement<StreamExpression>> {
        match self {
            ExpressionKind::Constant { .. }
            | ExpressionKind::Abstraction { .. }
            | ExpressionKind::Enumeration { .. }
            | ExpressionKind::Identifier { .. } => vec![],
            ExpressionKind::Application {
                function_expression,
                inputs,
            } => {
                let mut statements = function_expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                let mut inputs_staments = inputs
                    .iter_mut()
                    .flat_map(|expression| {
                        expression.inline_when_needed(
                            signal_id,
                            memory,
                            identifier_creator,
                            graph,
                            symbol_table,
                            unitary_nodes,
                        )
                    })
                    .collect();

                statements.append(&mut inputs_staments);

                statements
            }
            ExpressionKind::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(_, expression)| {
                    expression.inline_when_needed(
                        signal_id,
                        memory,
                        identifier_creator,
                        graph,
                        symbol_table,
                        unitary_nodes,
                    )
                })
                .collect(),
            ExpressionKind::Array { elements } => elements
                .iter_mut()
                .flat_map(|expression| {
                    expression.inline_when_needed(
                        signal_id,
                        memory,
                        identifier_creator,
                        graph,
                        symbol_table,
                        unitary_nodes,
                    )
                })
                .collect(),
            ExpressionKind::Match { expression, arms } => {
                let mut statements = expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                let mut arms_staments = arms
                    .iter_mut()
                    .flat_map(|(_, option, body, expression)| {
                        debug_assert!(body.is_empty());

                        let mut statements = expression.inline_when_needed(
                            signal_id,
                            memory,
                            identifier_creator,
                            graph,
                            symbol_table,
                            unitary_nodes,
                        );
                        let mut option_statements = option.as_mut().map_or(vec![], |expression| {
                            expression.inline_when_needed(
                                signal_id,
                                memory,
                                identifier_creator,
                                graph,
                                symbol_table,
                                unitary_nodes,
                            )
                        });

                        statements.append(&mut option_statements);

                        statements
                    })
                    .collect();

                statements.append(&mut arms_staments);

                statements
            }
            ExpressionKind::When {
                option,
                present,
                present_body,
                default,
                default_body,
                ..
            } => {
                debug_assert!(present_body.is_empty());
                debug_assert!(default_body.is_empty());

                let mut option_statements = option.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                let mut present_statements = present.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                let mut default_statements = default.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );

                option_statements.append(&mut present_statements);
                option_statements.append(&mut default_statements);

                option_statements
            }
            ExpressionKind::FieldAccess { expression, .. } => expression.inline_when_needed(
                signal_id,
                memory,
                identifier_creator,
                graph,
                symbol_table,
                unitary_nodes,
            ),
            ExpressionKind::TupleElementAccess { expression, .. } => expression.inline_when_needed(
                signal_id,
                memory,
                identifier_creator,
                graph,
                symbol_table,
                unitary_nodes,
            ),
            ExpressionKind::Map {
                expression,
                function_expression,
            } => {
                let mut statements = expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                let mut function_statements = function_expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );

                statements.append(&mut function_statements);

                statements
            }
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                let mut statements = expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                let mut initialization_statements = initialization_expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                let mut function_statements = function_expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );

                statements.append(&mut initialization_statements);
                statements.append(&mut function_statements);

                statements
            }
            ExpressionKind::Sort {
                expression,
                function_expression,
            } => {
                let mut statements = expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                let mut function_statements = function_expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );

                statements.append(&mut function_statements);

                statements
            }
            ExpressionKind::Zip { arrays } => arrays
                .iter_mut()
                .flat_map(|expression| {
                    expression.inline_when_needed(
                        signal_id,
                        memory,
                        identifier_creator,
                        graph,
                        symbol_table,
                        unitary_nodes,
                    )
                })
                .collect(),
        }
    }
}
