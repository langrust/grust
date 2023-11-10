/// LIR file construction from HIR project.
pub mod file;

/// LIR function construction from HIR function.
pub mod function;

/// LIR node files construction from HIR node.
pub mod node;

/// LIR expression construction from HIR expression.
pub mod expression;

/// LIR statement construction from HIR statement.
pub mod statement;

/// LIR expression construction from HIR stream expression.
pub mod stream_expression;

/// LIR statement construction from HIR equation.
pub mod equation;

/// LIR node file construction from HIR unitary node.
pub mod unitary_node;

/// LIR item construction from HIR typedef.
pub mod typedef;

/// LIR memory construction from HIR typedef.
pub mod memory;

/// LIR constant construction from HIR typedef.
pub mod constant;

/// LIR pattern construction from HIR typedef.
pub mod pattern;
