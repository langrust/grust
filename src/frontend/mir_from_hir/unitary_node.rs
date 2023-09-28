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
