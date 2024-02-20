use std::collections::HashMap;

use petgraph::{algo::has_path_connecting, graphmap::DiGraphMap};

use crate::{
    common::{graph::neighbor::Label, scope::Scope},
    hir::{
        dependencies::Dependencies,
        expression::ExpressionKind,
        identifier_creator::IdentifierCreator,
        memory::Memory,
        statement::Statement,
        stream_expression::{StreamExpression, StreamExpressionKind},
        unitary_node::UnitaryNode,
    },
    symbol_table::SymbolTable,
};

use super::Union;

impl StreamExpression {
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
        context_map: &HashMap<usize, Union<usize, StreamExpression>>,
    ) {
        match self.kind {
            StreamExpressionKind::Expression { ref mut expression } => {
                match expression {
                    ExpressionKind::Constant { .. }
                    | ExpressionKind::Abstraction { .. }
                    | ExpressionKind::Enumeration { .. } => (),
                    ExpressionKind::Identifier { ref mut id } => {
                        if let Some(element) = context_map.get(id) {
                            match element {
                                Union::I1(new_id) => {
                                    *id = *new_id;
                                    self.dependencies = Dependencies::from(vec![(*new_id, 0)]);
                                }
                                Union::I2(new_expression) => *self = new_expression.clone(),
                            }
                        }
                    }
                    ExpressionKind::Application { ref mut inputs, .. } => {
                        inputs
                            .iter_mut()
                            .for_each(|expression| expression.replace_by_context(context_map));

                        self.dependencies = Dependencies::from(
                            inputs
                                .iter()
                                .flat_map(|expression| expression.get_dependencies().clone())
                                .collect(),
                        );
                    }
                    ExpressionKind::Structure { ref mut fields, .. } => {
                        fields
                            .iter_mut()
                            .for_each(|(_, expression)| expression.replace_by_context(context_map));

                        self.dependencies = Dependencies::from(
                            fields
                                .iter()
                                .flat_map(|(_, expression)| expression.get_dependencies().clone())
                                .collect(),
                        );
                    }
                    ExpressionKind::Array {
                        ref mut elements, ..
                    } => {
                        elements
                            .iter_mut()
                            .for_each(|expression| expression.replace_by_context(context_map));

                        self.dependencies = Dependencies::from(
                            elements
                                .iter()
                                .flat_map(|expression| expression.get_dependencies().clone())
                                .collect(),
                        );
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
                                // body.iter_mut().for_each(|statement| {
                                //     statement.expression.replace_by_context(&context_map)
                                // });

                                matched_expression.replace_by_context(&context_map);
                                let mut matched_expression_dependencies = matched_expression
                                    .get_dependencies()
                                    .clone()
                                    .into_iter()
                                    .filter(|(signal, _)| !local_signals.contains(signal))
                                    .collect::<Vec<(usize, usize)>>();
                                expression_dependencies
                                    .append(&mut matched_expression_dependencies);
                            });

                        self.dependencies = Dependencies::from(expression_dependencies);
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
                        //     .for_each(|statement| statement.expression.replace_by_context(context_map));

                        present.replace_by_context(context_map);
                        let mut present_dependencies = present.get_dependencies().clone();

                        debug_assert!(default_body.is_empty());
                        // default_body
                        //     .iter_mut()
                        //     .for_each(|statement| statement.expression.replace_by_context(context_map));

                        default.replace_by_context(context_map);
                        let mut default_dependencies = default.get_dependencies().clone();

                        option_dependencies.append(&mut present_dependencies);
                        option_dependencies.append(&mut default_dependencies);
                        self.dependencies = Dependencies::from(option_dependencies);
                    }
                    ExpressionKind::FieldAccess {
                        ref mut expression, ..
                    } => {
                        expression.replace_by_context(context_map);
                        // get matched expression dependencies
                        let expression_dependencies = expression.get_dependencies().clone();
                        // push all dependencies in arms dependencies
                        self.dependencies = Dependencies::from(expression_dependencies);
                    }
                    ExpressionKind::TupleElementAccess {
                        ref mut expression, ..
                    } => {
                        expression.replace_by_context(context_map);
                        // get matched expression dependencies
                        let expression_dependencies = expression.get_dependencies().clone();
                        // push all dependencies in arms dependencies
                        self.dependencies = Dependencies::from(expression_dependencies);
                    }
                    ExpressionKind::Map {
                        ref mut expression, ..
                    } => {
                        expression.replace_by_context(context_map);
                        // get matched expression dependencies
                        let expression_dependencies = expression.get_dependencies().clone();
                        // push all dependencies in arms dependencies
                        self.dependencies = Dependencies::from(expression_dependencies);
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
                        self.dependencies = Dependencies::from(expression_dependencies);
                    }
                    ExpressionKind::Sort {
                        ref mut expression, ..
                    } => {
                        expression.replace_by_context(context_map);
                        // get matched expression dependencies
                        let expression_dependencies = expression.get_dependencies().clone();
                        // push all dependencies in arms dependencies
                        self.dependencies = Dependencies::from(expression_dependencies);
                    }
                    ExpressionKind::Zip { ref mut arrays, .. } => {
                        arrays
                            .iter_mut()
                            .for_each(|array| array.replace_by_context(context_map));

                        self.dependencies = Dependencies::from(
                            arrays
                                .iter()
                                .flat_map(|array| array.get_dependencies().clone())
                                .collect(),
                        );
                    }
                }
            }
            StreamExpressionKind::FollowedBy {
                ref mut expression, ..
            } => {
                expression.replace_by_context(context_map);

                self.dependencies = Dependencies::from(
                    expression
                        .get_dependencies()
                        .iter()
                        .map(|(id, depth)| (id.clone(), *depth + 1))
                        .collect(),
                );
            }
            StreamExpressionKind::UnitaryNodeApplication {
                ref mut node_id,
                ref mut inputs,
                ..
            } => {
                // replace the id of the called node
                if let Some(element) = context_map.get(node_id) {
                    match element {
                        Union::I1(new_id)
                        | Union::I2(StreamExpression {
                            kind:
                                StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            *node_id = new_id.clone();
                        }
                        Union::I2(_) => unreachable!(),
                    }
                }

                inputs
                    .iter_mut()
                    .for_each(|(_, expression)| expression.replace_by_context(context_map));

                // change dependencies to be the sum of inputs dependencies
                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
            }
            StreamExpressionKind::NodeApplication { .. } => unreachable!(),
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
        unitary_nodes: &HashMap<usize, UnitaryNode>,
    ) -> Vec<Statement<StreamExpression>> {
        match &mut self.kind {
            StreamExpressionKind::Expression { .. } => vec![],
            StreamExpressionKind::FollowedBy {
                ref mut expression, ..
            } => {
                let new_statements = expression.inline_when_needed(
                    signal_id,
                    memory,
                    identifier_creator,
                    graph,
                    symbol_table,
                    unitary_nodes,
                );
                self.dependencies = Dependencies::from(
                    expression
                        .get_dependencies()
                        .iter()
                        .map(|(id, depth)| (id.clone(), depth + 1))
                        .collect(),
                );
                new_statements
            }
            StreamExpressionKind::UnitaryNodeApplication {
                node_id,
                ref mut inputs,
                ..
            } => {
                // inline potential node calls in the inputs
                let mut new_statements = inputs
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
                    .collect::<Vec<_>>();

                // a loop in the graph induces that inputs depends on output
                let should_inline = has_path_connecting(graph, signal_id, signal_id, None); // TODO: check it is correct

                // then node call must be inlined
                if should_inline {
                    let called_unitary_node = unitary_nodes.get(&node_id).unwrap();

                    // get statements and memory from called node, with corresponding inputs
                    let (mut retrieved_statements, retrieved_memory) = called_unitary_node
                        .instantiate_statements_and_memory(
                            identifier_creator,
                            inputs,
                            None,
                            symbol_table,
                        );

                    // remove called node from memory
                    memory.remove_called_node(&node_id);

                    // change the expression to a signal call to the output signal
                    retrieved_statements.iter_mut().for_each(|statement| {
                        let scope = symbol_table.get_scope(&statement.id);
                        if scope == &Scope::Output {
                            self.kind = StreamExpressionKind::Expression {
                                expression: ExpressionKind::Identifier { id: statement.id },
                            };
                            self.dependencies = Dependencies::from(vec![(statement.id.clone(), 0)]);
                            symbol_table.set_scope(&statement.id, Scope::Local);
                        }
                    });

                    new_statements.append(&mut retrieved_statements);
                    memory.combine(retrieved_memory);
                } else {
                    // change dependencies to be the sum of inputs dependencies
                    self.dependencies = Dependencies::from(
                        inputs
                            .iter()
                            .flat_map(|(_, expression)| expression.get_dependencies().clone())
                            .collect(),
                    );
                }

                new_statements
            }
            StreamExpressionKind::NodeApplication { .. } => unreachable!(),
        }
    }
}
