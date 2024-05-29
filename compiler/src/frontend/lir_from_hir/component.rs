use itertools::Itertools;

use crate::{
    common::r#type::Type,
    hir::{identifier_creator::IdentifierCreator, node::Node},
    lir::{
        expression::Expression as LIRExpression,
        item::state_machine::{
            event::{Event, EventElement},
            input::{Input, InputElement},
            state::{init::Init, step::Step, State},
            StateMachine,
        },
    },
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for Node {
    type LIR = StateMachine;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let Node {
            id,
            statements,
            memory,
            contract,
            ..
        } = self;
        let mut identifier_creator = IdentifierCreator::from(vec![]);

        // get node name
        let name = symbol_table.get_name(id).clone();

        // get node inputs
        let mut inputs = symbol_table
            .get_node_inputs(id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(*id).clone(),
                    symbol_table.get_type(*id).clone(),
                )
            })
            .collect::<Vec<_>>();

        // create the event structure
        let event = if let Some(event_enum_id) = symbol_table.get_node_event_enum(id) {
            // add the event input
            let event_id = symbol_table.get_node_event(id).unwrap();
            inputs.push((
                symbol_table.get_name(event_id).clone(),
                Type::Enumeration {
                    name: symbol_table.get_name(event_enum_id).clone(),
                    id: event_enum_id,
                },
            ));

            // get enum elements
            let mut elements = symbol_table
                .get_event_enum_elements(event_enum_id)
                .iter()
                .map(|element_id| EventElement::InputEvent {
                    identifier: symbol_table.get_name(*element_id).clone(),
                    r#type: symbol_table.get_type(*element_id).clone(),
                })
                .collect::<Vec<_>>();

            // get generics
            let generics = elements
                .iter_mut()
                .flat_map(|event_elem| match event_elem {
                    EventElement::InputEvent { r#type, .. } => {
                        r#type.get_generics(&mut identifier_creator)
                    }
                    EventElement::NoEvent => vec![],
                })
                .collect::<Vec<_>>();

            Some(Event {
                node_name: name.clone(),
                elements,
                generics,
            })
        } else {
            None
        };

        // get node output type
        let outputs = symbol_table.get_node_outputs(id);
        let mut output_type = {
            let mut types = outputs
                .iter()
                .map(|(_, output_id)| symbol_table.get_type(*output_id).clone())
                .collect::<Vec<_>>();
            if types.len() == 1 {
                types.pop().unwrap()
            } else {
                Type::Tuple(types)
            }
        };

        // get node output expression
        let outputs = symbol_table.get_node_outputs(id);
        let output_expression = {
            let mut identifiers = outputs
                .iter()
                .map(|(_, output_id)| LIRExpression::Identifier {
                    identifier: symbol_table.get_name(*output_id).clone(),
                })
                .collect::<Vec<_>>();
            if identifiers.len() == 1 {
                identifiers.pop().unwrap()
            } else {
                LIRExpression::Tuple {
                    elements: identifiers,
                }
            }
        };

        // collect imports from statements, inputs and output types, memory and contracts
        let mut imports = statements
            .iter()
            .flat_map(|equation| equation.get_imports(symbol_table))
            .unique()
            .collect::<Vec<_>>();
        let mut inputs_type_imports = inputs
            .iter()
            .flat_map(|(_, typing)| typing.get_imports(symbol_table))
            .unique()
            .collect::<Vec<_>>();
        let mut output_type_imports = output_type.get_imports(symbol_table);
        let mut memory_imports = memory.get_imports(symbol_table);
        let mut contract_imports = contract.get_imports(symbol_table);

        // combining all imports and eliminate duplicates
        imports.append(&mut inputs_type_imports);
        imports.append(&mut output_type_imports);
        imports.append(&mut memory_imports);
        imports.append(&mut contract_imports);
        let imports = imports.into_iter().unique().collect::<Vec<_>>();

        // get input's generics: function types in inputs
        let mut generics = inputs
            .iter_mut()
            .flat_map(|(_, typing)| typing.get_generics(&mut identifier_creator))
            .collect::<Vec<_>>();
        let mut output_generics = output_type.get_generics(&mut identifier_creator);
        generics.append(&mut output_generics);

        // get memory/state elements
        let (elements, state_elements_init, state_elements_step) =
            memory.get_state_elements(symbol_table);

        // transform contract
        let contract = contract.lir_from_hir(symbol_table);

        StateMachine {
            name: name.clone(),
            imports,
            input: Input {
                node_name: name.clone(),
                elements: inputs
                    .into_iter()
                    .map(|(identifier, r#type)| InputElement { identifier, r#type })
                    .collect(),
                generics: generics.clone(),
            },
            event,
            state: State {
                node_name: name.clone(),
                elements,
                step: Step {
                    contract,
                    node_name: name.clone(),
                    generics,
                    output_type,
                    body: statements
                        .into_iter()
                        .map(|equation| equation.lir_from_hir(symbol_table))
                        .collect(),
                    state_elements_step,
                    output_expression,
                },
                init: Init {
                    node_name: name,
                    state_elements_init,
                    invariant_initialisation: vec![], // TODO
                },
            },
        }
    }
}
