use std::collections::HashMap;

use crate::ast::node::Node;
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::{
    contract::hir_from_ast as contract_hir_from_ast,
    equation::hir_from_ast as equation_hir_from_ast,
};
use crate::hir::{node::Node as HIRNode, once_cell::OnceCell};
use crate::symbol_table::{SymbolKind, SymbolTable};

/// Transform AST nodes into HIR nodes.
pub fn hir_from_ast(
    node: Node,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRNode, TerminationError> {
    let Node {
        id,
        is_component,
        inputs,
        equations,
        contract,
        location,
    } = node;

    let id = symbol_table.get_node_id(&id, false, location, errors)?;
    let node_symbol = symbol_table
        .get_symbol(&id)
        .expect("there should be a symbol");
    match node_symbol.kind() {
        SymbolKind::Node {
            inputs,
            outputs,
            locals,
            ..
        } => {
            // create local context with all signals
            symbol_table.local();
            symbol_table.restore_context(inputs.iter());
            symbol_table.restore_context(outputs.values());
            symbol_table.restore_context(locals.values());

    HIRNode {
        id,
        is_component,
        inputs,
        unscheduled_equations: equations
            .into_iter()
            .map(|(signal, equation)| (signal, equation_hir_from_ast(equation, &signals_context)))
            .collect(),
        unitary_nodes: HashMap::new(),
        contract: contract_hir_from_ast(contract, &signals_context),
        location,
        graph: OnceCell::new(),
    }
}

#[cfg(test)]
mod hir_from_ast {

    use std::collections::HashMap;

    use crate::ast::{
        equation::Equation, expression::Expression, node::Node, stream_expression::StreamExpression,
    };
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::hir_from_ast::node::hir_from_ast;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation as HIREquation, node::Node as HIRNode,
        once_cell::OnceCell, signal::Signal,
        stream_expression::StreamExpression as HIRStreamExpression,
    };

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_expression = StreamExpression::FunctionApplication {
            function_expression: Expression::Identifier {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("i"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_equation = Equation {
            id: String::from("o"),
            scope: Scope::Output,
            signal_type: Type::Integer,
            expression: ast_expression,
            location: Location::default(),
        };
        let ast_node = Node {
            contract: Default::default(),
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![(String::from("o"), ast_equation)],
            location: Location::default(),
        };
        let hir_node = hir_from_ast(ast_node);

        let control = HIRNode {
            contract: Default::default(),
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                HIREquation {
                    id: String::from("o"),
                    scope: Scope::Output,
                    signal_type: Type::Integer,
                    expression: HIRStreamExpression::FunctionApplication {
                        function_expression: Expression::Identifier {
                            id: String::from("f"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![HIRStreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("i"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        assert_eq!(hir_node, control);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_untyped_ast() {
        let ast_expression = StreamExpression::FunctionApplication {
            function_expression: Expression::Identifier {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("i"),
                typing: None,
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_equation = Equation {
            id: String::from("o"),
            scope: Scope::Output,
            signal_type: Type::Integer,
            expression: ast_expression,
            location: Location::default(),
        };
        let ast_node = Node {
            contract: Default::default(),
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![(String::from("o"), ast_equation)],
            location: Location::default(),
        };
        let _ = hir_from_ast(ast_node);
    }
}
