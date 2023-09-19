use self::{
    array_alias::ArrayAlias, enumeration::Enumeration, function::Function, node_file::NodeFile,
    structure::Structure,
};

/// MIR [ArrayAlias](crate::mir::item::array_alias::ArrayAlias) module.
pub mod array_alias;
/// MIR [Enumeration](crate::mir::item::enumeration::Enumeration) module.
pub mod enumeration;
/// MIR [Function](crate::mir::item::function::Function) module.
pub mod function;
/// MIR [NodeFile](crate::mir::item::node_file::NodeFile) module.
pub mod node_file;
/// MIR [Structure](crate::mir::item::structure::Structure) module.
pub mod structure;

/// An item of the project.
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
