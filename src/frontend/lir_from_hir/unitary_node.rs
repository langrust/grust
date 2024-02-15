use crate::{
    hir::unitary_node::UnitaryNode,
    lir::{
        expression::Expression as LIRExpression,
        item::node_file::{
            input::{Input, InputElement},
            state::{init::Init, step::Step, State},
            NodeFile,
        },
    },
    symbol_table::SymbolTable,
};

use super::{
    contract::lir_from_hir as contract_lir_from_hir,
    equation::lir_from_hir as equation_lir_from_hir,
};

/// Transform HIR unitary node into LIR node file.
pub fn lir_from_hir(unitary_node: UnitaryNode, symbol_table: &SymbolTable) -> NodeFile {
    let UnitaryNode {
        node_id,
        output_id,
        inputs,
        equations,
        memory,
        ..
    } = unitary_node;

    let output_type = symbol_table.get_output_type(&node_id).clone();

    let output_expression = LIRExpression::Identifier {
        identifier: symbol_table.get_name(&output_id).clone(),
    };

    // TODO: imports
    // let imports = equations
    //     .iter()
    //     .flat_map(|equation| equation.expression.get_imports())
    //     .unique()
    //     .collect();

    let (elements, state_elements_init, state_elements_step) =
        memory.get_state_elements(symbol_table);

    let name = symbol_table.get_name(&node_id);

    NodeFile {
        name: name.clone(),
        imports: vec![], // TODO
        input: Input {
            node_name: name.clone(),
            elements: inputs
                .into_iter()
                .map(|id| InputElement {
                    identifier: symbol_table.get_name(&id).clone(),
                    r#type: symbol_table.get_type(&id).clone(),
                })
                .collect(),
        },
        state: State {
            node_name: name.clone(),
            elements,
            step: Step {
                contract: contract_lir_from_hir(unitary_node.contract, symbol_table),
                node_name: name.clone(),
                output_type,
                body: equations
                    .into_iter()
                    .map(|equation| equation_lir_from_hir(equation, symbol_table))
                    .collect(),
                state_elements_step,
                output_expression,
            },
            init: Init {
                node_name: name.clone(),
                state_elements_init,
                invariant_initialisation: vec![], // TODO
            },
        },
    }
}
