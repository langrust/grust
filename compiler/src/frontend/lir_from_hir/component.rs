prelude! {
    hir::{Component, ComponentDefinition, ComponentImport},
    lir::{
        item::{Item, Import},
        state_machine::{Input, InputElm, Init, Step, State, StateMachine}
    },
}

use super::LIRFromHIR;

impl LIRFromHIR for Component {
    type LIR = Item;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self {
            hir::Component::Definition(comp_def) => {
                Item::StateMachine(comp_def.lir_from_hir(&symbol_table))
            }
            hir::Component::Import(comp_import) => {
                Item::Import(comp_import.lir_from_hir(&symbol_table))
            }
        }
    }
}

impl LIRFromHIR for ComponentDefinition {
    type LIR = StateMachine;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        // get node name
        let name = symbol_table.get_name(self.id);

        // get node inputs
        let inputs = symbol_table
            .get_node_inputs(self.id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(*id).clone(),
                    symbol_table.get_type(*id).clone(),
                )
            })
            .collect::<Vec<_>>();

        // get node output type
        let outputs = symbol_table.get_node_outputs(self.id);
        let output_type = {
            let mut types = outputs
                .iter()
                .map(|(_, output_id)| symbol_table.get_type(*output_id).clone())
                .collect::<Vec<_>>();
            if types.len() == 1 {
                types.pop().unwrap()
            } else {
                Typ::tuple(types)
            }
        };

        // get node output expression
        let outputs = symbol_table.get_node_outputs(self.id);
        let output_expression = {
            let mut identifiers = outputs
                .iter()
                .map(|(_, output_id)| lir::Expr::Identifier {
                    identifier: symbol_table.get_name(*output_id).clone(),
                })
                .collect::<Vec<_>>();
            if identifiers.len() == 1 {
                identifiers.pop().unwrap()
            } else {
                lir::Expr::Tuple {
                    elements: identifiers,
                }
            }
        };

        // get memory/state elements
        let (elements, state_elements_init, state_elements_step) =
            self.memory.get_state_elements(symbol_table);

        // transform contract
        let contract = self.contract.lir_from_hir(symbol_table);
        let invariant_initialization = vec![]; // TODO

        // 'init' method
        let init = Init::new(name, state_elements_init, invariant_initialization);

        // 'step' method
        let step = Step::new(
            name,
            output_type,
            self.statements
                .into_iter()
                .map(|equation| equation.lir_from_hir(symbol_table))
                .collect(),
            state_elements_step,
            output_expression,
            contract,
        );

        // 'input' structure
        let input = Input {
            node_name: name.clone(),
            elements: inputs
                .into_iter()
                .map(|(identifier, typ)| InputElm::new(identifier, typ))
                .collect(),
        };

        // 'state' structure
        let state = State {
            node_name: name.clone(),
            elements,
            step,
            init,
        };

        StateMachine::new(name, input, state)
    }
}

impl LIRFromHIR for ComponentImport {
    type LIR = Import;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        // get node name
        let name = symbol_table.get_name(self.id).clone();
        let path = self.path;

        Import { name, path }
    }
}
