prelude! {
    hir::{Component, ComponentDefinition, ComponentImport},
    lir::{
        item::{Item, Import},
        state_machine::{Input, InputElm, Init, Step, State, StateMachine}
    },
}

impl IntoLir<&'_ SymbolTable> for Component {
    type Lir = Item;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        match self {
            hir::Component::Definition(comp_def) => {
                Item::StateMachine(comp_def.into_lir(&symbol_table))
            }
            hir::Component::Import(comp_import) => {
                Item::Import(comp_import.into_lir(&symbol_table))
            }
        }
    }
}

impl IntoLir<&'_ SymbolTable> for ComponentDefinition {
    type Lir = StateMachine;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        // get node name
        let name = symbol_table.get_name(self.id);

        // get node inputs
        let inputs = symbol_table
            .get_node_inputs(self.id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(*id).clone(),
                    symbol_table.get_typ(*id).clone(),
                )
            })
            .collect::<Vec<_>>();

        // get node output type
        let outputs = symbol_table.get_node_outputs(self.id);
        let output_type = {
            let mut types = outputs
                .iter()
                .map(|(_, output_id)| symbol_table.get_typ(*output_id).clone())
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
            memory_state_elements(self.memory, symbol_table);

        // transform contract
        let contract = self.contract.into_lir(symbol_table);
        let invariant_initialization = vec![]; // TODO

        // 'init' method
        let init = Init::new(name, state_elements_init, invariant_initialization);

        // 'step' method
        let step = Step::new(
            name,
            output_type,
            self.statements
                .into_iter()
                .map(|equation| equation.into_lir(symbol_table))
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

impl IntoLir<&'_ SymbolTable> for ComponentImport {
    type Lir = Import;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        // get node name
        let name = symbol_table.get_name(self.id).clone();
        let path = self.path;

        Import { name, path }
    }
}

/// Get state elements from memory.
pub fn memory_state_elements(
    mem: hir::Memory,
    symbol_table: &SymbolTable,
) -> (
    Vec<lir::state_machine::StateElmInfo>,
    Vec<lir::state_machine::StateElmInit>,
    Vec<lir::state_machine::StateElmStep>,
) {
    use hir::memory::*;
    use itertools::Itertools;
    use lir::state_machine::{StateElmInfo, StateElmInit, StateElmStep};

    let (mut elements, mut inits, mut steps) = (vec![], vec![], vec![]);
    for (
        _,
        Buffer {
            ident,
            typing,
            init,
            id,
            ..
        },
    ) in mem.buffers.into_iter().sorted_by_key(|(id, _)| id.clone())
    {
        let scope = symbol_table.get_scope(id);
        let mem_ident = format!("last_{}", ident);
        elements.push(StateElmInfo::buffer(&mem_ident, typing));
        inits.push(StateElmInit::buffer(
            &mem_ident,
            init.into_lir(symbol_table),
        ));
        steps.push(StateElmStep::new(
            mem_ident,
            match scope {
                Scope::Input => lir::Expr::input_access(ident),
                Scope::Output | Scope::Local => lir::Expr::ident(ident),
                Scope::VeryLocal => unreachable!(),
            },
        ))
    }
    mem.called_nodes
        .into_iter()
        .sorted_by_key(|(id, _)| *id)
        .for_each(|(memory_id, CalledNode { node_id, .. })| {
            let memory_name = symbol_table.get_name(memory_id);
            let node_name = symbol_table.get_name(node_id);
            elements.push(StateElmInfo::called_node(memory_name, node_name));
            inits.push(StateElmInit::called_node(memory_name, node_name));
        });

    (elements, inits, steps)
}
