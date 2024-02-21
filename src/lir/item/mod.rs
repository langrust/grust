use self::{
    array_alias::ArrayAlias, enumeration::Enumeration, function::Function, node_file::NodeFile,
    structure::Structure,
};

/// LIR [ArrayAlias](crate::lir::item::array_alias::ArrayAlias) module.
pub mod array_alias;
/// LIR [Enumeration](crate::lir::item::enumeration::Enumeration) module.
pub mod enumeration;
/// LIR [Function](crate::lir::item::function::Function) module.
pub mod function;
/// LIR [NodeFile](crate::lir::item::node_file::NodeFile) module.
pub mod node_file;
/// LIR [Structure](crate::lir::item::structure::Structure) module.
pub mod structure;
/// LIR [Import](crate::lir::item::import::Import) module.
pub mod import;

/// An item of the project.
#[derive(Debug, PartialEq, serde::Serialize)]
pub enum Item {
    /// A node file.
    NodeFile(NodeFile),
    /// A function definition.
    Function(Function),
    /// An enumeration definition.
    Enumeration(Enumeration),
    /// A structure definition.
    Structure(Structure),
    /// An array alias definition.
    ArrayAlias(ArrayAlias),
}
