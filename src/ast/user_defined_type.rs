use crate::util::location::Location;

#[derive(Debug, PartialEq)]
/// LanGRust user defined type AST.
pub enum UserDefinedType {
    /// Represents a structure definition.
    Structure {
        // todo!()
        /// Structure location.
        location: Location,
    },
    /// Represents an enumeration definition.
    Enumeration {
        // todo!()
        /// Structure location.
        location: Location,
    },
    /// Represents an array definition.
    Array {
        // todo!()
        /// Structure location.
        location: Location,
    }
}
