use crate::ast::stream_expression::StreamExpression;
use crate::ir::stream_expression::StreamExpression as IRStreamExpression;

use crate::frontend::hir_from_ast::expression::hir_from_ast as expression_hir_from_ast;

/// Transform AST stream expressions into IR stream expressions.
pub fn hir_from_ast(stream_expression: StreamExpression) -> IRStreamExpression {
    match stream_expression {
        StreamExpression::Constant {
            constant,
            typing,
            location,
        } => IRStreamExpression::Constant {
            constant,
            typing: typing.unwrap(),
            location,
        },
        StreamExpression::SignalCall {
            id,
            typing,
            location,
        } => IRStreamExpression::SignalCall {
            id,
            typing: typing.unwrap(),
            location,
        },
        StreamExpression::MapApplication {
            function_expression,
            inputs,
            typing,
            location,
        } => IRStreamExpression::MapApplication {
            function_expression: expression_hir_from_ast(function_expression),
            inputs: inputs
                .into_iter()
                .map(|input| hir_from_ast(input))
                .collect(),
            typing: typing.unwrap(),
            location,
        },
        StreamExpression::Structure {
            name,
            fields,
            typing,
            location,
        } => IRStreamExpression::Structure {
            name,
            fields: fields
                .into_iter()
                .map(|(field, expression)| (field, hir_from_ast(expression)))
                .collect(),
            typing: typing.unwrap(),
            location,
        },
        StreamExpression::Array {
            elements,
            typing,
            location,
        } => IRStreamExpression::Array {
            elements: elements
                .into_iter()
                .map(|expression| hir_from_ast(expression))
                .collect(),
            typing: typing.unwrap(),
            location,
        },
        StreamExpression::Match {
            expression,
            arms,
            typing,
            location,
        } => IRStreamExpression::Match {
            expression: Box::new(hir_from_ast(*expression)),
            arms: arms
                .into_iter()
                .map(|(pattern, optional_expression, expression)| {
                    (
                        pattern,
                        optional_expression.map(|expression| hir_from_ast(expression)),
                        vec![],
                        hir_from_ast(expression),
                    )
                })
                .collect(),
            typing: typing.unwrap(),
            location,
        },
        StreamExpression::When {
            id,
            option,
            present,
            default,
            typing,
            location,
        } => IRStreamExpression::When {
            id,
            option: Box::new(hir_from_ast(*option)),
            present_body: vec![],
            present: Box::new(hir_from_ast(*present)),
            default_body: vec![],
            default: Box::new(hir_from_ast(*default)),
            typing: typing.unwrap(),
            location,
        },
        StreamExpression::FollowedBy {
            constant,
            expression,
            typing,
            location,
        } => IRStreamExpression::FollowedBy {
            constant,
            expression: Box::new(hir_from_ast(*expression)),
            typing: typing.unwrap(),
            location,
        },
        StreamExpression::NodeApplication {
            node,
            inputs,
            signal,
            typing,
            location,
        } => IRStreamExpression::NodeApplication {
            node,
            inputs: inputs
                .into_iter()
                .map(|input| hir_from_ast(input))
                .collect(),
            signal,
            typing: typing.unwrap(),
            location,
        },
    }
}

#[cfg(test)]
mod hir_from_ast {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{location::Location, type_system::Type};
    use crate::frontend::hir_from_ast::stream_expression::hir_from_ast;
    use crate::ir::stream_expression::StreamExpression as IRStreamExpression;

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_stream_expression = StreamExpression::SignalCall {
            id: String::from("s"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let hir_stream_expression = hir_from_ast(ast_stream_expression);

        let control = IRStreamExpression::SignalCall {
            id: String::from("s"),
            typing: Type::Integer,
            location: Location::default(),
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
        let _ = hir_from_ast(ast_stream_expression);
    }
}
