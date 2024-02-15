use crate::ast::file::File;
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::{
    function::hir_from_ast as function_hir_from_ast, node::hir_from_ast as node_hir_from_ast,
    typedef::hir_from_ast as typedef_hir_from_ast,
};
use crate::hir::file::File as HIRFile;
use crate::symbol_table::SymbolTable;

/// Transform AST files into HIR files.
pub fn hir_from_ast(
    file: File,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRFile, TerminationError> {
    let File {
        typedefs,
        functions,
        nodes,
        component,
        location,
    } = file;

    // TODO: this is supposed to be in another function in order to call nodes in any order
    // let inputs = inputs
    //     .into_iter()
    //     .map(|(name, typing)| {
    //         let id = symbol_table.insert_signal(name, Scope::Input, true, location, errors)?;
    //         // TODO: add type to signal in symbol table
    //         Ok(id)
    //     })
    //     .collect::<Vec<Result<_, _>>>()
    //     .into_iter()
    //     .collect::<Result<Vec<_>, _>>()?;
    // let outputs = equations
    //     .into_iter()
    //     .filter(|(name, equation)| Scope::Output == equation.scope)
    //     .map(|(name, equation)| {
    //         let id =
    //             symbol_table.insert_signal(name.clone(), Scope::Output, true, location, errors)?;
    //         // TODO: add type to signal in symbol table
    //         Ok((name, id))
    //     })
    //     .collect::<Vec<Result<_, _>>>()
    //     .into_iter()
    //     .collect::<Result<HashMap<_, _>, _>>()?;
    // let locals = equations
    //     .into_iter()
    //     .filter(|(name, equation)| Scope::Local == equation.scope)
    //     .map(|(name, equation)| {
    //         let id =
    //             symbol_table.insert_signal(name.clone(), Scope::Local, true, location, errors)?;
    //         // TODO: add type to signal in symbol table
    //         Ok((name, id))
    //     })
    //     .collect::<Vec<Result<_, _>>>()
    //     .into_iter()
    //     .collect::<Result<HashMap<_, _>, _>>()?;
    // let id = symbol_table.insert_node(
    //     id,
    //     is_component,
    //     false,
    //     inputs,
    //     outputs,
    //     locals,
    //     location,
    //     errors,
    // )?;

    // let id = symbol_table.insert_function(
    //     id,
    //     is_component,
    //     false,
    //     inputs,
    //     outputs,
    //     locals,
    //     location,
    //     errors,
    // )?;

    Ok(HIRFile {
        typedefs: typedefs
            .into_iter()
            .map(|typedef| typedef_hir_from_ast(typedef, symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        functions: functions
            .into_iter()
            .map(|function| function_hir_from_ast(function, symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        nodes: nodes
            .into_iter()
            .map(|node| node_hir_from_ast(node, symbol_table, errors))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        component: component
            .map(|node| node_hir_from_ast(node, symbol_table, errors))
            .transpose()?,
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
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::frontend::hir_from_ast::file::hir_from_ast;
    use crate::hir::{
        dependencies::Dependencies, equation::Equation as HIREquation, file::File as HIRFile,
        node::Node as HIRNode, once_cell::OnceCell, signal::Signal,
        stream_expression::StreamExpression as HIRStreamExpression,
    };

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_expression = Expression::Application {
            function_expression: Box::new(Expression::Identifier {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Identifier {
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
        let ast_returned_expression = Expression::Identifier {
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
        let ast_expression = StreamExpression::FunctionApplication {
            function_expression: Expression::Identifier {
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
            contract: Default::default(),
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![(String::from("o"), ast_equation)],
            location: Location::default(),
        };
        let ast_file = File {
            typedefs: vec![],
            functions: vec![ast_function],
            nodes: vec![ast_node],
            component: None,
            location: Location::default(),
        };
        let hir_file = hir_from_ast(ast_file);

        let control_function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("x"), Type::Integer)],
            statements: vec![Statement {
                id: String::from("y"),
                element_type: Type::Integer,
                expression: Expression::Application {
                    function_expression: Box::new(Expression::Identifier {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    }),
                    inputs: vec![Expression::Identifier {
                        id: String::from("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    }],
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                Expression::Identifier {
                    id: String::from("y"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let control_node = HIRNode {
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
                            id: String::from("my_function"),
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
        let control = HIRFile {
            typedefs: vec![],
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
            function_expression: Box::new(Expression::Identifier {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Identifier {
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
        let ast_returned_expression = Expression::Identifier {
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
        let ast_expression = StreamExpression::FunctionApplication {
            function_expression: Expression::Identifier {
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
            contract: Default::default(),
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![(String::from("o"), ast_equation)],
            location: Location::default(),
        };
        let ast_file = File {
            typedefs: vec![],
            functions: vec![ast_function],
            nodes: vec![ast_node],
            component: None,
            location: Location::default(),
        };
        let _ = hir_from_ast(ast_file);
    }
}
