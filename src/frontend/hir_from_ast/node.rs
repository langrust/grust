use std::collections::HashMap;

use once_cell::sync::OnceCell;

use crate::ast::node::Node;
use crate::frontend::hir_from_ast::equation::hir_from_ast as equation_hir_from_ast;
use crate::hir::node::Node as HIRNode;

/// Transform AST nodes into HIR nodes.
pub fn hir_from_ast(node: Node) -> HIRNode {
    let Node {
        id,
        is_component,
        inputs,
        equations,
        location,
    } = node;

    HIRNode {
        id,
        is_component,
        inputs,
        unscheduled_equations: equations
            .into_iter()
            .map(|(signal, equation)| (signal, equation_hir_from_ast(equation)))
            .collect(),
        unitary_nodes: HashMap::new(),
        location,
        graph: OnceCell::new(),
    }
}

#[cfg(test)]
mod hir_from_ast {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;

    use crate::ast::{
        equation::Equation, expression::Expression, node::Node, stream_expression::StreamExpression,
    };
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::hir_from_ast::node::hir_from_ast;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation as HIREquation, node::Node as HIRNode,
        stream_expression::StreamExpression as HIRStreamExpression,
    };

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
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
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![(String::from("o"), ast_equation)],
            location: Location::default(),
        };
        let hir_node = hir_from_ast(ast_node);

        let control = HIRNode {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                HIREquation {
                    id: String::from("o"),
                    scope: Scope::Output,
                    signal_type: Type::Integer,
                    expression: HIRStreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("f"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![HIRStreamExpression::SignalCall {
                            id: String::from("i"),
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
        let ast_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
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
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![(String::from("o"), ast_equation)],
            location: Location::default(),
        };
        let _ = hir_from_ast(ast_node);
    }
}
