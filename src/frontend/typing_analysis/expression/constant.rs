use crate::{
    error::{Error, TerminationError},
    hir::expression::{Expression, ExpressionKind},
    symbol_table::SymbolTable,
};

impl Expression {
    /// Add a [Type] to the constant expression.
    pub fn typing_constant(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // typing a constant expression consist of getting the type of the constant
            ExpressionKind::Constant { ref constant } => {
                let constant_type = constant.get_type();
                // TODO : faire expression enumeration
                // match &constant_type {
                //     Type::Enumeration(type_id) => match user_types_context.get(type_id) {
                //         Some(Typedef::Enumeration { .. }) => (),
                //         _ => {
                //             let error = Error::UnknownEnumeration {
                //                 name: type_id.clone(),
                //                 location: self.location.clone(),
                //             };
                //             errors.push(error);
                //             return Err(TerminationError);
                //         }
                //     },
                //     _ => (),
                // }
                self.typing = Some(constant_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
