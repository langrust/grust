use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::neighbor::Label;
use crate::hir::expression::ExpressionKind;
use crate::hir::stream_expression::StreamExpressionKind;
use crate::hir::{
    dependencies::Dependencies, identifier_creator::IdentifierCreator, statement::Statement,
    stream_expression::StreamExpression,
};
use crate::symbol_table::SymbolTable;

impl StreamExpression {
    /// Change HIR expression into a normal form.
    ///
    /// The normal form of an expression is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn normal_form(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<Statement<StreamExpression>> {
        match self.kind {
            StreamExpressionKind::FollowedBy {
                ref mut expression, ..
            } => {
                let new_statements =
                    expression.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                self.dependencies = Dependencies::from(
                    expression
                        .get_dependencies()
                        .iter()
                        .map(|(id, depth)| (id.clone(), *depth + 1))
                        .collect(),
                );

                new_statements
            }
            StreamExpressionKind::NodeApplication { .. } => unreachable!(),
            StreamExpressionKind::UnitaryNodeApplication {
                node_id,
                ref mut inputs,
                output_id,
            } => {
                let mut new_statements = inputs
                    .iter_mut()
                    .flat_map(|(_, expression)| {
                        expression.into_signal_call(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        )
                    })
                    .collect::<Vec<_>>();

                let fresh_name = identifier_creator.new_identifier(
                    String::from(""),
                    String::from("x"),
                    String::from(""),
                );
                let fresh_id = symbol_table
                    .insert_identifier(fresh_name, todo!(), true, todo!(), todo!())
                    .expect("this function should not fail");

                // change dependencies to be the sum of inputs dependencies
                let reduced_graph = nodes_reduced_graphs.get(&node_id).unwrap();
                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(input_id, expression)| {
                            reduced_graph.edge_weight(output_id, *input_id).map_or(
                                vec![],
                                |label| {
                                    match label {
                                        Label::Contract => vec![], // TODO: do we loose the CREUSOT dependence with the input?
                                        Label::Weight(weight) => expression
                                            .get_dependencies()
                                            .clone()
                                            .into_iter()
                                            .map(|(id, depth)| (id, depth + weight))
                                            .collect(),
                                    }
                                },
                            )
                        })
                        .collect(),
                );

                let typing = self.get_type().clone();
                let location = self.location.clone();

                let unitary_node_application_statement = Statement {
                    id: fresh_id.clone(),
                    location: location.clone(),
                    expression: self.clone(),
                };

                self.kind = StreamExpressionKind::Expression {
                    expression: ExpressionKind::Identifier { id: fresh_id },
                };
                self.dependencies = Dependencies::from(vec![(fresh_id, 0)]);

                new_statements.push(unitary_node_application_statement);

                new_statements
            }
            StreamExpressionKind::Expression { ref mut expression } => {
                match expression {
                    ExpressionKind::Application { ref mut inputs, .. } => {
                        let new_statements = inputs
                            .iter_mut()
                            .flat_map(|expression| {
                                expression.normal_form(
                                    nodes_reduced_graphs,
                                    identifier_creator,
                                    symbol_table,
                                )
                            })
                            .collect();

                        self.dependencies = Dependencies::from(
                            inputs
                                .iter()
                                .flat_map(|expression| expression.get_dependencies().clone())
                                .collect(),
                        );

                        new_statements
                    }

                    ExpressionKind::Structure { fields, .. } => {
                        let new_statements = fields
                            .iter_mut()
                            .flat_map(|(_, expression)| {
                                expression.normal_form(
                                    nodes_reduced_graphs,
                                    identifier_creator,
                                    symbol_table,
                                )
                            })
                            .collect();

                        self.dependencies = Dependencies::from(
                            fields
                                .iter()
                                .flat_map(|(_, expression)| expression.get_dependencies().clone())
                                .collect(),
                        );

                        new_statements
                    }
                    ExpressionKind::Array { elements, .. } => {
                        let new_statements = elements
                            .iter_mut()
                            .flat_map(|expression| {
                                expression.normal_form(
                                    nodes_reduced_graphs,
                                    identifier_creator,
                                    symbol_table,
                                )
                            })
                            .collect();

                        self.dependencies = Dependencies::from(
                            elements
                                .iter()
                                .flat_map(|expression| expression.get_dependencies().clone())
                                .collect(),
                        );

                        new_statements
                    }
                    ExpressionKind::Match {
                        expression, arms, ..
                    } => {
                        let mut statements = expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );
                        let mut expression_dependencies = expression.get_dependencies().clone();

                        arms.iter_mut()
                            .for_each(|(pattern, bound, body, matched_expression)| {
                                // get local signals defined in pattern
                                let local_signals = pattern.local_identifiers();

                                // remove identifiers created by the pattern from the dependencies
                                let (mut bound_statements, mut bound_dependencies) =
                                    bound.as_mut().map_or((vec![], vec![]), |expression| {
                                        (
                                            expression.normal_form(
                                                nodes_reduced_graphs,
                                                identifier_creator,
                                                symbol_table,
                                            ),
                                            expression
                                                .get_dependencies()
                                                .clone()
                                                .into_iter()
                                                .filter(|(signal, _)| {
                                                    !local_signals.contains(signal)
                                                })
                                                .collect(),
                                        )
                                    });
                                statements.append(&mut bound_statements);
                                expression_dependencies.append(&mut bound_dependencies);

                                let mut matched_expression_statements = matched_expression
                                    .normal_form(
                                        nodes_reduced_graphs,
                                        identifier_creator,
                                        symbol_table,
                                    );
                                let mut matched_expression_dependencies = matched_expression
                                    .get_dependencies()
                                    .clone()
                                    .into_iter()
                                    .filter(|(signal, _)| !local_signals.contains(signal))
                                    .collect();
                                body.append(&mut matched_expression_statements);
                                expression_dependencies.append(&mut matched_expression_dependencies)
                            });

                        self.dependencies = Dependencies::from(expression_dependencies);

                        statements
                    }
                    ExpressionKind::When {
                        option,
                        present_body,
                        present,
                        default_body,
                        default,
                        ..
                    } => {
                        let new_statements = option.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );
                        let mut option_dependencies = option.get_dependencies().clone();

                        let mut present_statements = present.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );
                        let mut present_dependencies = present.get_dependencies().clone();
                        present_body.append(&mut present_statements);
                        option_dependencies.append(&mut present_dependencies);

                        let mut default_statements = default.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );
                        let mut default_dependencies = default.get_dependencies().clone();
                        default_body.append(&mut default_statements);
                        option_dependencies.append(&mut default_dependencies);

                        self.dependencies = Dependencies::from(option_dependencies);

                        new_statements
                    }
                    ExpressionKind::FieldAccess { expression, .. } => {
                        let new_statements = expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );

                        self.dependencies =
                            Dependencies::from(expression.get_dependencies().clone());

                        new_statements
                    }
                    ExpressionKind::TupleElementAccess { expression, .. } => {
                        let new_statements = expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );

                        self.dependencies =
                            Dependencies::from(expression.get_dependencies().clone());

                        new_statements
                    }
                    ExpressionKind::Map { expression, .. } => {
                        let new_statements = expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );

                        self.dependencies =
                            Dependencies::from(expression.get_dependencies().clone());

                        new_statements
                    }
                    ExpressionKind::Fold {
                        expression,
                        initialization_expression,
                        ..
                    } => {
                        let mut new_statements = expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );
                        let mut initialization_statements = initialization_expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );
                        new_statements.append(&mut initialization_statements);

                        // get matched expressions dependencies
                        let mut expression_dependencies = expression.get_dependencies().clone();
                        let mut initialization_expression_dependencies =
                            expression.get_dependencies().clone();
                        expression_dependencies.append(&mut initialization_expression_dependencies);

                        // push all dependencies in arms dependencies
                        self.dependencies = Dependencies::from(expression_dependencies);

                        new_statements
                    }
                    ExpressionKind::Sort { expression, .. } => {
                        let new_statements = expression.normal_form(
                            nodes_reduced_graphs,
                            identifier_creator,
                            symbol_table,
                        );

                        self.dependencies =
                            Dependencies::from(expression.get_dependencies().clone());

                        new_statements
                    }
                    ExpressionKind::Zip { arrays, .. } => {
                        let new_statements = arrays
                            .iter_mut()
                            .flat_map(|array| {
                                array.normal_form(
                                    nodes_reduced_graphs,
                                    identifier_creator,
                                    symbol_table,
                                )
                            })
                            .collect();

                        self.dependencies = Dependencies::from(
                            arrays
                                .iter()
                                .flat_map(|array| array.get_dependencies().clone())
                                .collect(),
                        );

                        new_statements
                    }
                    ExpressionKind::Constant { .. } | ExpressionKind::Identifier { .. } => {
                        vec![]
                    }
                    ExpressionKind::Abstraction { inputs, expression } => todo!(),
                    ExpressionKind::Enumeration { enum_id, elem_id } => todo!(),
                }
            }
        }
    }

    /// Change HIR expression into a signal call.
    ///
    /// If the expression is not a signal call, then normalize the expression,
    /// create an statement with the normalized expression and change current
    /// expression into a call to the statement.
    ///
    /// # Example
    ///
    /// The expression `1 + my_node(s, v*2).o` becomes `x` along with:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn into_signal_call(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<Statement<StreamExpression>> {
        match self.kind {
            StreamExpressionKind::Expression {
                expression: ExpressionKind::Identifier { .. },
            } => vec![],
            _ => {
                let mut statements =
                    self.normal_form(nodes_reduced_graphs, identifier_creator, symbol_table);

                let typing = self.get_type().clone();
                let location = self.location.clone();

                let fresh_name = identifier_creator.new_identifier(
                    String::from(""),
                    String::from("x"),
                    String::from(""),
                );
                let fresh_id = symbol_table
                    .insert_identifier(fresh_name, todo!(), true, todo!(), todo!())
                    .expect("this function should not fail");

                let new_statement = Statement {
                    id: fresh_id.clone(),
                    location: location.clone(),
                    expression: self.clone(),
                };

                self.kind = StreamExpressionKind::Expression {
                    expression: ExpressionKind::Identifier { id: fresh_id },
                };
                self.dependencies = Dependencies::from(vec![(fresh_id, 0)]);

                statements.push(new_statement);
                statements
            }
        }
    }
}
