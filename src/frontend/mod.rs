/// HIR construction from AST.
pub mod hir_from_ast;

/// Dependency graph construction algorithms.
pub mod dependency_graph;

/// Causality analysis of HIR.
pub mod causality_analysis;

/// Normalization module.
pub mod normalizing;

/// LIR construction from HIR.
pub mod lir_from_hir;
