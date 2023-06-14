/// [Context] trait definition.
pub mod context;

/// [Graph] API.
pub mod graph;

/// [Color] enumeration used to identify the processing status of an element.
pub mod color;

/// LanGRust [UserDefinedType](crate::ast::user_defined_type::UserDefinedType) AST module.
pub mod user_defined_type;

/// LanGRust [Pattern](crate::ast::pattern::Pattern) AST module.
pub mod pattern;

/// Location handler module.
pub mod location;

/// Type system module.
pub mod type_system;

/// Constant module.
pub mod constant;

/// Operator module.
pub mod operator;

/// Scope module.
pub mod scope;
