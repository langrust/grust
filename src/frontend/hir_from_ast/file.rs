use crate::ast::file::File;
use crate::frontend::hir_from_ast::function::hir_from_ast as function_hir_from_ast;
use crate::frontend::hir_from_ast::node::hir_from_ast as node_hir_from_ast;
use crate::ir::file::File as IRFile;

/// Transform AST files into IR files.
pub fn hir_from_ast(file: File) -> IRFile {
    let File {
        user_defined_types,
        functions,
        nodes,
        component,
        location,
    } = file;

    IRFile {
        user_defined_types,
        functions: functions
            .into_iter()
            .map(|function| function_hir_from_ast(function))
            .collect(),
        nodes: nodes
            .into_iter()
            .map(|node| node_hir_from_ast(node))
            .collect(),
        component: component.map(|component| node_hir_from_ast(component)),
        location,
    }
}

#[cfg(test)]
mod hir_from_ast {
    use std::collections::HashMap;

    use crate::ast::{
        equation::Equation, expression::Expression, file::File, function::Function, node::Node,
        statement::Statement, stream_expression::StreamExpression,
    };
    use crate::common::{location::Location, scope::Scope, type_system::Type};
    use crate::frontend::hir_from_ast::file::hir_from_ast;
    use crate::ir::{
        equation::Equation as IREquation, expression::Expression as IRExpression,
        file::File as IRFile, function::Function as IRFunction, node::Node as IRNode,
        statement::Statement as IRStatement,
        stream_expression::StreamExpression as IRStreamExpression,
    };

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_statement = Statement {
            id: String::from("y"),
            element_type: Type::Integer,
            expression: ast_expression,
            location: Location::default(),
        };
        let ast_returned_expression = Expression::Call {
            id: String::from("y"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("x"), Type::Integer)],
            statements: vec![ast_statement],
            returned: (Type::Integer, ast_returned_expression),
            location: Location::default(),
        };
        let ast_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("my_function"),
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
        let ast_file = File {
            user_defined_types: vec![],
            functions: vec![ast_function],
            nodes: vec![ast_node],
            component: None,
            location: Location::default(),
        };
        let hir_file = hir_from_ast(ast_file);

        let control_function = IRFunction {
            id: String::from("my_function"),
            inputs: vec![(String::from("x"), Type::Integer)],
            statements: vec![IRStatement {
                id: String::from("y"),
                element_type: Type::Integer,
                expression: IRExpression::Application {
                    function_expression: Box::new(IRExpression::Call {
                        id: String::from("f"),
                        typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                        location: Location::default(),
                    }),
                    inputs: vec![IRExpression::Call {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                IRExpression::Call {
                    id: String::from("y"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let control_node = IRNode {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                IREquation {
                    id: String::from("o"),
                    scope: Scope::Output,
                    signal_type: Type::Integer,
                    expression: IRStreamExpression::MapApplication {
                        function_expression: IRExpression::Call {
                            id: String::from("my_function"),
                            typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                            location: Location::default(),
                        },
                        inputs: vec![IRStreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };
        let control = IRFile {
            user_defined_types: vec![],
            functions: vec![control_function],
            nodes: vec![control_node],
            component: None,
            location: Location::default(),
        };
        assert_eq!(hir_file, control);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_untyped_ast() {
        let ast_expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_statement = Statement {
            id: String::from("y"),
            element_type: Type::Integer,
            expression: ast_expression,
            location: Location::default(),
        };
        let ast_returned_expression = Expression::Call {
            id: String::from("y"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let ast_function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("x"), Type::Integer)],
            statements: vec![ast_statement],
            returned: (Type::Integer, ast_returned_expression),
            location: Location::default(),
        };
        let ast_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("my_function"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("i"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: None,
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
        let ast_file = File {
            user_defined_types: vec![],
            functions: vec![ast_function],
            nodes: vec![ast_node],
            component: None,
            location: Location::default(),
        };
        let _ = hir_from_ast(ast_file);
    }
}
