use crate::frontend::lir_from_mir::r#type::lir_from_mir as type_lir_from_mir;
use crate::lir::item::type_alias::TypeAlias;
use crate::lir::r#type::Type as LIRType;
use crate::mir::item::array_alias::ArrayAlias;

/// Transform MIR array alias into LIR type alias.
pub fn lir_from_mir(array_alias: ArrayAlias) -> TypeAlias {
    TypeAlias {
        public_visibility: true,
        name: array_alias.name,
        r#type: LIRType::Array {
            element: Box::new(type_lir_from_mir(array_alias.array_type)),
            size: array_alias.size,
        },
    }
}
