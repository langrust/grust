//! Rust AST construction from [LIR types](mod@crate::lir).

/// RustAST block construction from LIR block.
pub mod block;
/// RustAST expression construction from LIR expression.
pub mod expression;
/// RustAST item construction from LIR item.
pub mod item;
/// RustAST pattern construction from LIR pattern.
pub mod pattern;
/// RustAST project construction from LIR project.
pub mod project;
/// RustAST statement construction from LIR statement.
pub mod statement;
/// RustAST type construction from LIR type.
pub mod typ;
