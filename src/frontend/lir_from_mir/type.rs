use crate::common::r#type::Type;
use crate::lir::r#type::Type as LIRType;

/// Transform MIR type into LIR type.
pub fn lir_from_mir(r#type: Type) -> LIRType {
    LIRType::Owned(r#type)
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::r#type::Type;
    use crate::frontend::lir_from_mir::r#type::lir_from_mir;
    use crate::lir::r#type::Type as LIRType;

    #[test]
    fn should_create_an_owned_lir_type_containing_the_mir_type() {
        let r#type = Type::Integer;
        let control = LIRType::Owned(Type::Integer);
        assert_eq!(lir_from_mir(r#type), control)
    }
}
