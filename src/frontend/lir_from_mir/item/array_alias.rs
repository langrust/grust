use crate::common::r#type::Type;
use crate::lir::item::type_alias::TypeAlias;
use crate::lir::r#type::Type as LIRType;
use crate::mir::item::array_alias::ArrayAlias;

/// Transform MIR array alias into LIR type alias.
pub fn lir_from_mir(array_alias: ArrayAlias) -> TypeAlias {
    TypeAlias {
        public_visibility: true,
        name: array_alias.name,
        r#type: LIRType::Owned(Type::Array(
            Box::new(array_alias.array_type),
            array_alias.size,
        )),
    }
}
