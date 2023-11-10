use self::{
    array_alias::ArrayAlias, enumeration::Enumeration, function::Function, node_file::NodeFile,
    structure::Structure,
};

/// LIR [ArrayAlias](crate::mir::item::array_alias::ArrayAlias) module.
pub mod array_alias;
/// LIR [Enumeration](crate::mir::item::enumeration::Enumeration) module.
pub mod enumeration;
/// LIR [Function](crate::mir::item::function::Function) module.
pub mod function;
/// LIR [NodeFile](crate::mir::item::node_file::NodeFile) module.
pub mod node_file;
/// LIR [Structure](crate::mir::item::structure::Structure) module.
pub mod structure;

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
