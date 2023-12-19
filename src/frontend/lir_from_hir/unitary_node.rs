use itertools::Itertools;

use crate::{
    common::scope::Scope,
    hir::unitary_node::UnitaryNode,
    lir::{
        expression::Expression as LIRExpression,
        item::node_file::{
            input::{Input, InputElement},
            state::{init::Init, step::Step, State},
            NodeFile,
        },
    },
};

use super::equation::lir_from_hir as equation_lir_from_hir;

/// Transform HIR unitary node into LIR node file.
pub fn lir_from_hir(unitary_node: UnitaryNode) -> NodeFile {
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

    let output_expression = LIRExpression::Identifier {
        identifier: output_id.clone(),
    };

    let imports = equations
        .iter()
        .flat_map(|equation| equation.expression.get_imports())
        .unique()
        .collect();

    let (elements, state_elements_init, state_elements_step) = memory.get_state_elements();

    NodeFile {
        name: format!("{node_id}_{output_id}"),
        imports,
        input: Input {
            node_name: format!("{node_id}_{output_id}"),
            elements: inputs
                .into_iter()
                .map(|(identifier, r#type)| InputElement { identifier, r#type })
                .collect(),
        },
        state: State {
            node_name: format!("{node_id}_{output_id}"),
            elements,
            step: Step {
                node_name: format!("{node_id}_{output_id}"),
                output_type,
                body: equations.into_iter().map(equation_lir_from_hir).collect(),
                state_elements_step,
                output_expression,
            },
            init: Init {
                node_name: format!("{node_id}_{output_id}"),
                state_elements_init,
            },
        },
    }
}

#[cfg(test)]
mod lir_from_hir {
    use std::collections::HashMap;

    use crate::{
        ast::expression::Expression as ASTExpression,
        common::{constant::Constant, location::Location, r#type::Type, scope::Scope},
        frontend::lir_from_hir::unitary_node::lir_from_hir,
        hir::{
            dependencies::Dependencies,
            equation::Equation,
            memory::{Buffer, CalledNode, Memory},
            once_cell::OnceCell,
            signal::Signal,
            stream_expression::StreamExpression,
            unitary_node::UnitaryNode,
        },
        lir::{
            expression::Expression,
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
            statement::Statement,
        },
    };

    #[test]
    fn should_transform_hir_unitary_node_definition_into_lir_node_file() {
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
            called_nodes: HashMap::from([(
                format!("other_node_o_o"),
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
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: format!("mem_i"),
                            scope: Scope::Memory,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(format!("mem_i"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: format!("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::UnitaryNodeApplication {
                        id: Some(format!("other_node_o_o")),
                        node: format!("other_node"),
                        signal: format!("o"),
                        inputs: vec![
                            (
                                format!("a"),
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: format!("x"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(format!("x"), 0)]),
                                },
                            ),
                            (
                                format!("b"),
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: format!("i"),
                                        scope: Scope::Local,
                                    },
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
            name: format!("my_node_o"),
            imports: vec![Import::NodeFile(format!("other_node_o"))],
            input: Input {
                node_name: format!("my_node_o"),
                elements: vec![InputElement {
                    identifier: format!("x"),
                    r#type: Type::Integer,
                }],
            },
            state: State {
                node_name: format!("my_node_o"),
                elements: vec![
                    StateElement::Buffer {
                        identifier: format!("mem_i"),
                        r#type: Type::Integer,
                    },
                    StateElement::CalledNode {
                        identifier: format!("other_node_o_o"),
                        node_name: format!("other_node_o"),
                    },
                ],
                step: Step {
                    node_name: format!("my_node_o"),
                    output_type: Type::Integer,
                    body: vec![
                        Statement::Let {
                            identifier: format!("i"),
                            expression: Expression::MemoryAccess {
                                identifier: format!("mem_i"),
                            },
                        },
                        Statement::Let {
                            identifier: format!("o"),
                            expression: Expression::NodeCall {
                                node_identifier: format!("other_node_o_o"),
                                input_name: format!("OtherNodeOInput"),
                                input_fields: vec![
                                    (
                                        format!("a"),
                                        Expression::InputAccess {
                                            identifier: format!("x"),
                                        },
                                    ),
                                    (
                                        format!("b"),
                                        Expression::Identifier {
                                            identifier: format!("i"),
                                        },
                                    ),
                                ],
                            },
                        },
                    ],
                    state_elements_step: vec![StateElementStep {
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
                    output_expression: Expression::Identifier {
                        identifier: format!("o"),
                    },
                },
                init: Init {
                    node_name: format!("my_node_o"),
                    state_elements_init: vec![
                        StateElementInit::BufferInit {
                            identifier: format!("mem_i"),
                            initial_value: Constant::Integer(0),
                        },
                        StateElementInit::CalledNodeInit {
                            identifier: format!("other_node_o_o"),
                            node_name: format!("other_node_o"),
                        },
                    ],
                },
            },
        };
        assert_eq!(lir_from_hir(unitary_node), control)
    }
}
