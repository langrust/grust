use crate::ast::stream_expression::StreamExpression;
use crate::hir::{
    dependencies::Dependencies, stream_expression::StreamExpression as HIRStreamExpression,
};

/// Transform AST stream expressions into HIR stream expressions.
pub fn hir_from_ast(stream_expression: StreamExpression) -> HIRStreamExpression {
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
        } => HIRStreamExpression::SignalCall {
            id,
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
        StreamExpression::MapApplication {
            function_expression,
            inputs,
            typing,
            location,
        } => HIRStreamExpression::MapApplication {
            function_expression: function_expression,
            inputs: inputs
                .into_iter()
                .map(|input| hir_from_ast(input))
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
                .map(|(field, expression)| (field, hir_from_ast(expression)))
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
                .map(|expression| hir_from_ast(expression))
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
            option: Box::new(hir_from_ast(*option)),
            present_body: vec![],
            present: Box::new(hir_from_ast(*present)),
            default_body: vec![],
            default: Box::new(hir_from_ast(*default)),
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
            expression: Box::new(hir_from_ast(*expression)),
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
                .map(|input| hir_from_ast(input))
                .collect(),
            signal,
            typing: typing.unwrap(),
            location,
            dependencies: Dependencies::new(),
        },
    }
}

#[cfg(test)]
mod hir_from_ast {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{location::Location, r#type::Type};
    use crate::frontend::hir_from_ast::stream_expression::hir_from_ast;
    use crate::hir::{
        dependencies::Dependencies, stream_expression::StreamExpression as HIRStreamExpression,
    };

    #[test]
    fn should_construct_hir_structure_from_typed_ast() {
        let ast_stream_expression = StreamExpression::SignalCall {
            id: String::from("s"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };
        let hir_stream_expression = hir_from_ast(ast_stream_expression);

        let control = HIRStreamExpression::SignalCall {
            id: String::from("s"),
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
        let _ = hir_from_ast(ast_stream_expression);
    }
}
