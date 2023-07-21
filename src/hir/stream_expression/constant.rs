use crate::hir::stream_expression::StreamExpression;

impl StreamExpression {
    /// Compute dependencies of a constant stream expression.
    pub fn compute_dependencies_constant(&self) -> Result<(), ()> {
        match self {
            // no dependencies for constant stream expression
            StreamExpression::Constant { dependencies, .. } => {
                dependencies.set(vec![]);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_dependencies_constant {
    use crate::common::{constant::Constant, location::Location, r#type::Type};
    use crate::hir::dependencies::Dependencies;
    use crate::hir::stream_expression::StreamExpression;

    #[test]
    fn should_compute_no_dependencies_from_constant_expression() {
        let stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(1),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression.compute_dependencies_constant().unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![];

        assert_eq!(dependencies, control)
    }
}
