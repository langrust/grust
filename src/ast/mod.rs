/// LanGRust [File](crate::ast::file::File) AST module.
pub mod file;

/// LanGRust [Component](crate::ast::component::Component) AST module.
pub mod component;

/// LanGRust [Node](crate::ast::node::Node) AST module.
pub mod node;

/// Describe a node from its input, output and local signals module.
pub mod node_description;

/// LanGRust [Function](crate::ast::function::Function) AST module.
pub mod function;

/// LanGRust global context definition module.
pub mod global_context;

/// LanGRust [UserDefinedType](crate::ast::user_defined_type::UserDefinedType) AST module.
pub mod user_defined_type;

/// LanGRust [StreamExpression](crate::ast::stream_expression::StreamExpression) AST module.
pub mod stream_expression;

/// LanGRust [Expression](crate::ast::expression::Expression) AST module.
pub mod expression;

/// LanGRust [Equation](crate::ast::equation::Equation) AST module.
pub mod equation;

/// LanGRust [Calculus](crate::ast::calculus::Calculus) AST module.
pub mod calculus;

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
