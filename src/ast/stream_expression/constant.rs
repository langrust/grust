use crate::ast::stream_expression::StreamExpression;

impl StreamExpression {
    /// Add a [Type] to the constant stream expression.
    pub fn typing_constant(&mut self) -> Result<(), ()> {
        match self {
            // typing a constant stream expression consist of getting the type of the constant
            StreamExpression::Constant {
                constant,
                typing,
                location: _,
            } => {
                *typing = Some(constant.get_type());
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_constant {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{constant::Constant, location::Location};

    #[test]
    fn should_type_constant_stream_expression() {
        let mut stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: Some(Constant::Integer(0).get_type()),
            location: Location::default(),
        };

        stream_expression.typing_constant().unwrap();

        assert_eq!(stream_expression, control);
    }
}
