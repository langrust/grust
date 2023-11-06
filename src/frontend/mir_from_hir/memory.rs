use itertools::Itertools;

use crate::{
    frontend::mir_from_hir::stream_expression::mir_from_hir as stream_expression_mir_from_hir,
    hir::memory::{Buffer, CalledNode, Memory},
    mir::{
        expression::Expression as MIRExpression,
        item::node_file::state::{init::StateElementInit, step::StateElementStep, StateElement},
    },
};

impl Memory {
    /// Get state elements from memory.
    pub fn get_state_elements(
        self,
    ) -> (
        Vec<StateElement>,
        Vec<StateElementInit>,
        Vec<StateElementStep>,
    ) {
        let Memory {
            buffers,
            called_nodes,
        } = self;

        let (mut elements, mut inits, mut steps) = (vec![], vec![], vec![]);
        buffers
            .into_iter()
            .sorted_by_key(|(x, _)| x.clone())
            .for_each(
                |(
                    id,
                    Buffer {
                        typing,
                        initial_value,
                        expression,
                    },
                )| {
                    elements.push(StateElement::Buffer {
                        identifier: id.clone(),
                        r#type: typing,
                    });
                    inits.push(StateElementInit::BufferInit {
                        identifier: id.clone(),
                        initial_value,
                    });
                    steps.push(StateElementStep {
                        identifier: id,
                        expression: stream_expression_mir_from_hir(expression),
                    });
                },
            );
        called_nodes
            .into_iter()
            .sorted_by_key(|(x, _)| x.clone())
            .for_each(|(id, CalledNode { node_id, signal_id })| {
                elements.push(StateElement::CalledNode {
                    identifier: id.clone(),
                    node_name: node_id.clone() + &signal_id,
                });
                inits.push(StateElementInit::CalledNodeInit {
                    identifier: id.clone(),
                    node_name: node_id + &signal_id,
                });
                steps.push(StateElementStep {
                    identifier: id.clone(),
                    expression: MIRExpression::Identifier { identifier: id },
                });
            });

        (elements, inits, steps)
    }
}

#[cfg(test)]
mod get_state_elements {
    use std::collections::HashMap;

    use crate::{
        ast::expression::Expression as ASTExpression,
        common::{constant::Constant, location::Location, r#type::Type, scope::Scope},
        hir::{
            dependencies::Dependencies,
            memory::{Buffer, CalledNode, Memory},
            signal::Signal,
            stream_expression::StreamExpression,
        },
        mir::{
            expression::Expression,
            item::node_file::state::{
                init::StateElementInit, step::StateElementStep, StateElement,
            },
        },
    };

    #[test]
    fn should_get_buffer_element_initialization_and_update() {
        let memory = Memory {
            buffers: HashMap::from([(
                format!("mem_i"),
                Buffer {
                    typing: Type::Integer,
                    initial_value: Constant::Integer(0),
                    expression: StreamExpression::MapApplication {
                        function_expression: ASTExpression::Call {
                            id: format!(" + "),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: format!("i"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(format!("i"), 0)]),
                            },
                            StreamExpression::Constant {
                                constant: Constant::Integer(1),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::new(),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(format!("i"), 0)]),
                    },
                },
            )]),
            called_nodes: HashMap::new(),
        };
        let control = (
            vec![StateElement::Buffer {
                identifier: format!("mem_i"),
                r#type: Type::Integer,
            }],
            vec![StateElementInit::BufferInit {
                identifier: format!("mem_i"),
                initial_value: Constant::Integer(0),
            }],
            vec![StateElementStep {
                identifier: format!("mem_i"),
                expression: Expression::FunctionCall {
                    function: Box::new(Expression::Identifier {
                        identifier: format!(" + "),
                    }),
                    arguments: vec![
                        Expression::Identifier {
                            identifier: format!("i"),
                        },
                        Expression::Literal {
                            literal: Constant::Integer(1),
                        },
                    ],
                },
            }],
        );
        assert_eq!(memory.get_state_elements(), control)
    }

    #[test]
    fn should_get_called_node_element_initialization_and_update() {
        let memory = Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::from([(
                format!("my_nodeox"),
                CalledNode {
                    node_id: format!("my_node"),
                    signal_id: format!("o"),
                },
            )]),
        };
        let control = (
            vec![StateElement::CalledNode {
                identifier: format!("my_nodeox"),
                node_name: format!("my_nodeo"),
            }],
            vec![StateElementInit::CalledNodeInit {
                identifier: format!("my_nodeox"),
                node_name: format!("my_nodeo"),
            }],
            vec![StateElementStep {
                identifier: format!("my_nodeox"),
                expression: Expression::Identifier {
                    identifier: format!("my_nodeox"),
                },
            }],
        );
        assert_eq!(memory.get_state_elements(), control)
    }
}
