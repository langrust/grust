use crate::common::r#type::Type;
use crate::{
    error::TerminationError, frontend::typing_analysis::TypeAnalysis,
    hir::expression::ExpressionKind,
};

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the constant expression.
    pub fn typing_constant(&mut self) -> Result<Type, TerminationError> {
        match self {
            // typing a constant expression consist of getting the type of the constant
            ExpressionKind::Constant { ref constant } => {
                let constant_type = constant.get_type();
                Ok(constant_type)
            }
            _ => unreachable!(),
        }
    }
}
