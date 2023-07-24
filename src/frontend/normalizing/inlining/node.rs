use std::collections::HashMap;

use crate::{
    common::scope::Scope,
    hir::{
        equation::Equation, identifier_creator::IdentifierCreator, node::Node,
        stream_expression::StreamExpression,
    },
};

use super::Union;

impl Node {
    /// Instantiate node's equations with inputs.
    ///
    /// It will return new equations where the input signals are instanciated by
    /// expressions.
    /// New equations should have fresh id according to the calling node.
    ///
    /// # Example
    ///
    /// ```GR
    /// node to_be_inlined(i: int) {
    ///     o: int = 0 fby j;
    ///     out j: int = i + 1;
    /// }
    ///
    /// node calling_node(i: int) {
    ///     out o: int = to_be_inlined(o);
    ///     j: int = i * o;
    /// }
    /// ```
    ///
    /// The call to `to_be_inlined` will generate th following equations:
    ///
    /// ```GR
    /// o: int = 0 fby j_1;
    /// j_1: int = o + 1;
    /// ```
    pub fn instantiate_equations(
        &self,
        called_signal: &String,
        identifier_creator: &mut IdentifierCreator,
        inputs: &Vec<StreamExpression>,
        output: &String,
        scope: &Scope,
    ) -> Vec<Equation> {
        // create the context with the given inputs
        let mut context_map = self
            .inputs
            .iter()
            .zip(inputs)
            .map(|((input, _), expression)| (input.clone(), Union::I2(expression.clone())))
            .collect::<HashMap<_, _>>();

        // add output to context
        context_map.insert(called_signal.clone(), Union::I1(output.clone())); // todo : the scope of the output might change

        // construct called_signal's subgraph
        let subgraph = self
            .graph
            .get()
            .unwrap()
            .clone()
            .subgraph_from_vertex(&called_signal);

        // get usefull equations
        let equations = subgraph
            .get_vertices()
            .iter()
            .filter_map(|signal| self.unscheduled_equations.get(signal))
            .map(|equation| equation.clone())
            .collect::<Vec<_>>();

        // add identifiers of the inlined equations to the context
        equations.iter().for_each(|equation| {
            if !(&equation.id == called_signal && called_signal == output) {
                equation.add_necessary_renaming(identifier_creator, &mut context_map)
            }
        });

        // reduce equations according to the context
        let mut reduced_equations = equations
            .iter()
            .map(|equation| equation.replace_by_context(&context_map))
            .collect::<Vec<_>>();

        // replace the output equation scope
        reduced_equations.iter_mut().for_each(|equation| {
            if &equation.id == output {
                equation.scope = scope.clone()
            }
        });

        reduced_equations
    }
}

#[cfg(test)]
mod instantiate_equations {
    use std::collections::HashMap;

    use once_cell::sync::OnceCell;

    use crate::ast::expression::Expression;
    use crate::common::{
        constant::Constant,
        graph::{color::Color, Graph},
        location::Location,
        r#type::Type,
        scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        node::Node, stream_expression::StreamExpression,
    };

    #[test]
    fn should_instantiate_nodes_equations_with_the_given_inputs() {
        // node calling_node(i: int) {
        //     o: int = to_be_inlined(o);
        //     out j: int = i * o;
        // }
        let mut identifier_creator = IdentifierCreator::from(vec![
            String::from("i"),
            String::from("j"),
            String::from("o"),
        ]);

        // node to_be_inlined(i: int) {
        //     out o: int = 0 fby j;
        //     j: int = i + 1;
        // }
        let node = Node {
            id: String::from("to_be_inlined"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("j"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("j"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("j"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+1"),
                                typing: Some(Type::Integer),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("i"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::White);
        graph.add_vertex(String::from("j"), Color::White);
        graph.add_vertex(String::from("i"), Color::White);
        graph.add_edge(&String::from("j"), String::from("i"), 0);
        graph.add_edge(&String::from("o"), String::from("j"), 1);
        node.graph.set(graph).unwrap();

        let equations = node.instantiate_equations(
            &String::from("o"),
            &mut identifier_creator,
            &vec![StreamExpression::SignalCall {
                id: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
            }],
            &String::from("o"),
            &Scope::Local,
        );

        // o: int = 0 fby j_1;
        // j_1: int = o + 1;
        let control = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("o"),
                signal_type: Type::Integer,
                expression: StreamExpression::FollowedBy {
                    constant: Constant::Integer(0),
                    expression: Box::new(StreamExpression::SignalCall {
                        id: String::from("j_1"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                    }),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Local,
                id: String::from("j_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("+1"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("o"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                },
                location: Location::default(),
            },
        ];

        assert_eq!(equations.len(), control.len());
        for equation in equations {
            assert!(control
                .iter()
                .any(|control_equation| &equation == control_equation))
        }
    }
}
