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

use super::LIRFromHIR;

impl LIRFromHIR for UnitaryNode {
    type LIR = NodeFile;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let UnitaryNode {
            unitary_node_id,
            statements,
            memory,
            ..
        } = self;

        let inputs = symbol_table.get_node_inputs(&unitary_node_id);
        let output_type = symbol_table
            .get_unitary_node_output_type(&unitary_node_id)
            .clone();

        let output_expression = LIRExpression::Identifier {
            identifier: symbol_table
                .get_unitary_node_output_name(&unitary_node_id)
                .clone(),
        };

        // TODO: imports
        // let imports = statements
        //     .iter()
        //     .flat_map(|equation| equation.expression.get_imports())
        //     .unique()
        //     .collect();

        let (elements, state_elements_init, state_elements_step) =
            memory.get_state_elements(symbol_table);

        let name = symbol_table.get_name(&unitary_node_id);

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
                    contract: self.contract.lir_from_hir(symbol_table),
                    node_name: name.clone(),
                    output_type,
                    body: statements
                        .into_iter()
                        .map(|equation| equation.lir_from_hir(symbol_table))
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
}
