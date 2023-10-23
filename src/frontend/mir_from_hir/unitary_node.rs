use crate::{
    ast::expression::Expression,
    common::scope::Scope,
    frontend::mir_from_hir::stream_expression::mir_from_hir as stream_expression_mir_from_hir,
    hir::{
        memory::{Buffer, CalledNode, Memory},
        stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    },
    mir::{
        expression::Expression as MIRExpression,
        item::node_file::{
            import::Import,
            input::{Input, InputElement},
            state::{
                init::{Init, StateElementInit},
                step::{StateElementStep, Step},
                State, StateElement,
            },
            NodeFile,
        },
    },
};

use super::equation::mir_from_hir as equation_mir_from_hir;

fn get_imports(expression: &StreamExpression) -> Vec<Import> {
    match expression {
        StreamExpression::FollowedBy { expression, .. } => get_imports(expression),
        StreamExpression::MapApplication {
            function_expression: Expression::Call { id, .. },
            ..
        } => vec![Import::Function(id.clone())],
        StreamExpression::UnitaryNodeApplication { node, .. } => {
            vec![Import::NodeFile(node.clone())]
        }
        StreamExpression::Structure { fields, .. } => fields
            .iter()
            .flat_map(|(_, expression)| get_imports(expression))
            .collect(),
        StreamExpression::Array { elements, .. } => elements
            .iter()
            .flat_map(|expression| get_imports(expression))
            .collect(),
        StreamExpression::Match {
            expression, arms, ..
        } => {
            let mut arms_imports = arms
                .iter()
                .flat_map(|(_, guard, body, expression)| {
                    let mut guard_imports = guard
                        .as_ref()
                        .map_or(vec![], |expression| get_imports(expression));
                    let mut body_imports = body
                        .iter()
                        .flat_map(|equation| get_imports(&equation.expression))
                        .collect();
                    let mut expression_imports = get_imports(expression);

                    let mut imports = vec![];
                    imports.append(&mut guard_imports);
                    imports.append(&mut body_imports);
                    imports.append(&mut expression_imports);
                    imports
                })
                .collect();
            let mut expression_imports = get_imports(expression);

            let mut imports = vec![];
            imports.append(&mut arms_imports);
            imports.append(&mut expression_imports);
            imports
        }
        StreamExpression::When {
            option,
            present_body,
            present,
            default_body,
            default,
            ..
        } => {
            let mut option_imports = get_imports(option);
            let mut present_body_imports = present_body
                .iter()
                .flat_map(|equation| get_imports(&equation.expression))
                .collect();
            let mut present_imports = get_imports(present);
            let mut default_body_imports = default_body
                .iter()
                .flat_map(|equation| get_imports(&equation.expression))
                .collect();
            let mut default_imports = get_imports(default);

            let mut imports = vec![];
            imports.append(&mut option_imports);
            imports.append(&mut present_body_imports);
            imports.append(&mut present_imports);
            imports.append(&mut default_body_imports);
            imports.append(&mut default_imports);
            imports
        }
        StreamExpression::NodeApplication { .. } => unreachable!(),
        _ => vec![],
    }
}

fn get_state_elements(
    memory: Memory,
) -> (
    Vec<StateElement>,
    Vec<StateElementInit>,
    Vec<StateElementStep>,
) {
    let Memory {
        buffers,
        called_nodes,
    } = memory;

    let (mut elements, mut inits, mut steps) = (vec![], vec![], vec![]);
    buffers.into_iter().for_each(
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

/// Transform HIR unitary node into MIR node file.
pub fn mir_from_hir(unitary_node: UnitaryNode) -> NodeFile {
    let UnitaryNode {
        node_id,
        output_id,
        inputs,
        equations,
        memory,
        ..
    } = unitary_node;

    let output_type = equations
        .iter()
        .filter(|equation| equation.scope == Scope::Output)
        .map(|equation| equation.signal_type.clone())
        .next()
        .unwrap();

    let output_expression = MIRExpression::Identifier {
        identifier: output_id.clone(),
    };

    let imports = equations
        .iter()
        .flat_map(|equation| get_imports(&equation.expression))
        .collect();

    let (elements, state_elements_init, state_elements_step) = get_state_elements(memory);

    NodeFile {
        name: node_id.clone() + &output_id,
        imports,
        input: Input {
            node_name: node_id.clone() + &output_id,
            elements: inputs
                .into_iter()
                .map(|(identifier, r#type)| InputElement { identifier, r#type })
                .collect(),
        },
        state: State {
            node_name: node_id.clone() + &output_id,
            elements,
            step: Step {
                node_name: node_id.clone() + &output_id,
                output_type,
                body: equations
                    .into_iter()
                    .map(|equation| equation_mir_from_hir(equation))
                    .collect(),
                state_elements_step,
                output_expression,
            },
            init: Init {
                node_name: node_id + &output_id,
                state_elements_init,
            },
        },
    }
}

#[cfg(test)]
mod get_imports {
    use crate::{
        ast::expression::Expression,
        common::{location::Location, r#type::Type},
        frontend::mir_from_hir::unitary_node::get_imports,
        hir::{dependencies::Dependencies, stream_expression::StreamExpression},
        mir::item::node_file::import::Import,
    };

    #[test]
    fn should_get_function_import_from_function_call_expression() {
        let expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: format!("my_function"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: format!("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(format!("i"), 0)]),
            }],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("i"), 0)]),
        };
        let control = vec![Import::Function(format!("my_function"))];
        assert_eq!(get_imports(&expression), control)
    }

    #[test]
    fn should_get_node_import_from_node_call_expression() {
        let expression = StreamExpression::UnitaryNodeApplication {
            id: Some(format!("my_nodeox")),
            node: format!("my_node"),
            signal: format!("o"),
            inputs: vec![(
                format!("i"),
                StreamExpression::SignalCall {
                    id: format!("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(format!("i"), 0)]),
                },
            )],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(format!("i"), 0)]),
        };
        let control = vec![Import::NodeFile(format!("my_node"))];
        assert_eq!(get_imports(&expression), control)
    }
}

#[cfg(test)]
mod get_state_elements {
    use std::collections::HashMap;

    use crate::{
        ast::expression::Expression as ASTExpression,
        common::{constant::Constant, location::Location, r#type::Type},
        frontend::mir_from_hir::unitary_node::get_state_elements,
        hir::{
            dependencies::Dependencies,
            memory::{Buffer, CalledNode, Memory},
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
                                id: format!("i"),
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
        assert_eq!(get_state_elements(memory), control)
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
        assert_eq!(get_state_elements(memory), control)
    }
}

#[cfg(test)]
mod mir_from_hir {
    use std::collections::HashMap;

    use once_cell::sync::OnceCell;

    use crate::{
        ast::expression::Expression as ASTExpression,
        common::{constant::Constant, location::Location, r#type::Type, scope::Scope},
        frontend::mir_from_hir::unitary_node::mir_from_hir,
        hir::{
            dependencies::Dependencies,
            equation::Equation,
            memory::{Buffer, CalledNode, Memory},
            stream_expression::StreamExpression,
            unitary_node::UnitaryNode,
        },
        mir::{
            expression::Expression,
            item::node_file::{
                import::Import,
                input::{Input, InputElement},
                state::{
                    step::{StateElementStep, Step},
                    State, StateElement,
                },
                NodeFile,
            },
            statement::Statement,
        },
    };

    #[test]
    fn should_transform_hir_unitary_node_definition_into_mir_node_file() {
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
                                id: format!("i"),
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
            called_nodes: HashMap::from([(
                format!("other_nodeoo"),
                CalledNode {
                    node_id: format!("other_node"),
                    signal_id: format!("o"),
                },
            )]),
        };
        let unitary_node = UnitaryNode {
            node_id: format!("my_node"),
            output_id: format!("o"),
            inputs: vec![(format!("x"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: format!("i"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::MapApplication {
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
                                    id: format!("i"),
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
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(format!("i"), 1)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: format!("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::UnitaryNodeApplication {
                        id: Some(format!("other_nodeoo")),
                        node: format!("other_node"),
                        signal: format!("o"),
                        inputs: vec![
                            (
                                format!("a"),
                                StreamExpression::SignalCall {
                                    id: format!("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                                },
                            ),
                            (
                                format!("b"),
                                StreamExpression::SignalCall {
                                    id: format!("i"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(format!("i"), 0)]),
                                },
                            ),
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (format!("x"), 0),
                            (format!("i"), 0),
                        ]),
                    },
                    location: Location::default(),
                },
            ],
            memory,
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let control = NodeFile {
            name: format!("my_nodeo"),
            imports: vec![Import::NodeFile(format!("other_node"))],
            input: Input {
                node_name: format!("my_nodeo"),
                elements: vec![InputElement {
                    identifier: format!("x"),
                    r#type: Type::Integer,
                }],
            },
            state: State {
                node_name: format!("my_nodeo"),
                elements: vec![
                    StateElement::CalledNode {
                        identifier: format!("other_nodeoo"),
                        node_name: format!("other_node"),
                    },
                    StateElement::Buffer {
                        identifier: format!("mem_i"),
                        r#type: Type::Integer,
                    },
                ],
                step: Step {
                    node_name: format!("my_nodeo"),
                    output_type: Type::Integer,
                    body: vec![
                        Statement::Let {
                            identifier: format!("i"),
                            expression: Expression::MemoryAccess {
                                identifier: format!("mem_i"),
                            },
                        },
                        Statement::LetTuple {
                            identifiers: todo!(),
                            expression: todo!(),
                        },
                    ],
                    state_elements_step: vec![
                        StateElementStep {
                            identifier: format!("other_nodeoo"),
                            expression: Expression::Identifier {
                                identifier: format!("other_nodeoo"),
                            },
                        },
                        StateElementStep {
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
                        },
                    ],
                    output_expression: todo!(),
                },
                init: todo!(),
            },
        };
        assert_eq!(mir_from_hir(unitary_node), control)
    }
}
