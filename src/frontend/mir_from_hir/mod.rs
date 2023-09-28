/// MIR file construction from HIR project.
pub mod file;

/// MIR function construction from HIR function.
pub mod function;

/// MIR node files construction from HIR node.
pub mod node;

/// MIR expression construction from HIR expression.
pub mod expression;

/// MIR statement construction from HIR statement.
pub mod statement;

/// MIR expression construction from HIR stream expression.
pub mod stream_expression;

/// MIR statement construction from HIR equation.
pub mod equation;

/// MIR node file construction from HIR unitary node.
pub mod unitary_node;

/// MIR item construction from HIR typedef.
pub mod typedef;
