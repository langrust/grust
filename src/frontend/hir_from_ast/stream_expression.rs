use std::collections::HashMap;

use crate::ast::stream_expression::StreamExpression;
use crate::common::scope::Scope;
use crate::hir::{
    dependencies::Dependencies, signal::Signal,
    stream_expression::StreamExpression as HIRStreamExpression,
};

/// Transform AST stream expressions into HIR stream expressions.
pub fn hir_from_ast(
    stream_expression: StreamExpression,
    signals_context: &HashMap<String, Scope>,
) -> HIRStreamExpression {
    match stream_expression {
        StreamExpression::Constant {
            constant,
            typing,
            location,
        } => HIRStreamExpression::Constant {
            constant,
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::SignalCall {
            id,
            typing,
            location,
        } => {
            let scope = signals_context.get(&id).unwrap().clone();
            HIRStreamExpression::SignalCall {
                signal: Signal { id, scope },
                typing: typing.unwrap(),
                location,
                dependencies: Dependencies::new(),
            }
        }
        StreamExpression::FunctionApplication {
            function_expression,
            inputs,
            typing,
            location,
        } => HIRStreamExpression::FunctionApplication {
            function_expression,
            inputs: inputs
                .into_iter()
                .map(|input| hir_from_ast(input, signals_context))
                .collect(),
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::Structure {
            name,
            fields,
            typing,
            location,
        } => HIRStreamExpression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(field, expression)| (field, hir_from_ast(expression, signals_context)))
                .collect(),
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::Array {
            elements,
            typing,
            location,
        } => HIRStreamExpression::Array {
            elements: elements
                .into_iter()
                .map(|expression| hir_from_ast(expression, signals_context))
                .collect(),
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::Match {
            expression,
            arms,
            typing,
            location,
        } => HIRStreamExpression::Match {
            expression: Box::new(hir_from_ast(*expression, signals_context)),
            arms: arms
                .into_iter()
                .map(|(pattern, optional_expression, expression)| {
                    let mut local_context = signals_context.clone();
                    pattern.fill_context(&mut local_context);
                    (
                        pattern,
                        optional_expression
                            .map(|expression| hir_from_ast(expression, &local_context)),
                        vec![],
                        hir_from_ast(expression, &local_context),
                    )
                })
                .collect(),
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::When {
            id,
            option,
            present,
            default,
            typing,
            location,
        } => HIRStreamExpression::When {
            id,
            option: Box::new(hir_from_ast(*option, signals_context)),
            present_body: vec![],
            present: Box::new(hir_from_ast(*present, signals_context)),
            default_body: vec![],
            default: Box::new(hir_from_ast(*default, signals_context)),
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::FollowedBy {
            constant,
            expression,
            typing,
            location,
        } => HIRStreamExpression::FollowedBy {
            constant,
            expression: Box::new(hir_from_ast(*expression, signals_context)),
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::NodeApplication {
            node,
            inputs,
            signal,
            typing,
            location,
        } => HIRStreamExpression::NodeApplication {
            node,
            inputs: inputs
                .into_iter()
                .map(|input| hir_from_ast(input, signals_context))
                .collect(),
            signal,
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::FieldAccess {
            expression,
            field,
            typing,
            location,
        } => HIRStreamExpression::FieldAccess {
            expression: Box::new(hir_from_ast(*expression, signals_context)),
            field,
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::Map {
            expression,
            function_expression,
            typing,
            location,
        } => HIRStreamExpression::Map {
            expression: Box::new(hir_from_ast(*expression, signals_context)),
            function_expression,
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::Fold {
            expression,
            initialization_expression,
            function_expression,
            typing,
            location,
        } => HIRStreamExpression::Fold {
            expression: Box::new(hir_from_ast(*expression, signals_context)),
            initialization_expression: Box::new(hir_from_ast(
                *initialization_expression,
                signals_context,
            )),
            function_expression,
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::Sort {
            expression,
            function_expression,
            typing,
            location,
        } => HIRStreamExpression::Sort {
            expression: Box::new(hir_from_ast(*expression, signals_context)),
            function_expression,
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::Zip {
            arrays,
            typing,
            location,
        } => HIRStreamExpression::Zip {
            arrays: arrays
                .into_iter()
                .map(|array| hir_from_ast(array, signals_context))
                .collect(),
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::TupleElementAccess {
            expression,
            element_number,
            typing,
            location,
        } => todo!(),
    }
}

#[cfg(test)]
mod hir_from_ast {
    use std::collections::HashMap;

    use crate::ast::stream_expression::StreamExpression;
    use crate::common::scope::Scope;
    use crate::common::{location::Location, r#type::Type};
    use crate::frontend::hir_from_ast::stream_expression::hir_from_ast;
    use crate::hir::{
        dependencies::Dependencies, signal::Signal,
        stream_expression::StreamExpression as HIRStreamExpression,
    };

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_stream_expression = StreamExpression::SignalCall {
            id: String::from("s"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let signals_context = HashMap::from([(format!("s"), Scope::Local)]);
        let hir_stream_expression = hir_from_ast(ast_stream_expression, &signals_context);

        let control = HIRStreamExpression::SignalCall {
            signal: Signal {
                id: String::from("s"),
                scope: Scope::Local,
            },
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };
        assert_eq!(hir_stream_expression, control);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_untyped_ast() {
        let ast_stream_expression = StreamExpression::SignalCall {
            id: String::from("s"),
            typing: None,
            location: Location::default(),
        };
        let signals_context = HashMap::from([(format!("s"), Scope::Local)]);
        let _ = hir_from_ast(ast_stream_expression, &signals_context);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_unknown_signal() {
        let ast_stream_expression = StreamExpression::SignalCall {
            id: String::from("s"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let signals_context = HashMap::from([(format!("x"), Scope::Local)]);
        let _ = hir_from_ast(ast_stream_expression, &signals_context);
    }
}
