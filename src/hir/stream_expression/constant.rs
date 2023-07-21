use crate::hir::stream_expression::StreamExpression;

impl StreamExpression {
    /// Get dependencies of a constant stream expression.
    pub fn get_constant_dependencies(&self) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // no dependencies for constant stream expression
            StreamExpression::Constant { .. } => Ok(vec![]),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod get_dependencies_constant {
    use crate::common::{constant::Constant, location::Location, r#type::Type};
    use crate::hir::dependencies::Dependencies;
    use crate::hir::stream_expression::StreamExpression;

    #[test]
    fn should_get_no_dependencies_from_constant_expression() {
        let stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(1),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        let dependencies = stream_expression.get_constant_dependencies().unwrap();

        let control = vec![];

        assert_eq!(dependencies, control)
    }
}
