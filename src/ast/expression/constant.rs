use crate::ast::expression::Expression;

impl Expression {
    /// Add a [Type] to the constant expression.
    pub fn typing_constant(&mut self) -> Result<(), ()> {
        match self {
            // typing a constant expression consist of getting the type of the constant
            Expression::Constant {
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
    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location};

    #[test]
    fn should_type_constant_expression() {
        let mut expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Constant {
            constant: Constant::Integer(0),
            typing: Some(Constant::Integer(0).get_type()),
            location: Location::default(),
        };

        expression.typing_constant().unwrap();

        assert_eq!(expression, control);
    }
}
