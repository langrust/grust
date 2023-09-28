/// HIR construction from AST.
pub mod hir_from_ast;

/// Dependency graph construction algorithms.
pub mod dependency_graph;

/// Causality analysis of HIR.
pub mod causality_analysis;

/// Normalization module.
pub mod normalizing;

/// MIR construction from HIR.
pub mod mir_from_hir;

/// LIR construction from MIR.
pub mod lir_from_mir;
