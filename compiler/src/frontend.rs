/// HIR construction from AST.
pub mod into_hir;

/// Dependency graph construction algorithms.
pub mod dependency_graph;

/// Causality analysis of HIR.
pub mod causality_analysis;

/// Normalization module.
pub mod normalizing;

/// LIR construction from HIR.
pub mod into_lir;

/// Typing analysis from HIR.
pub mod typing_analysis;

pub use typing_analysis::TypeAnalysis;

pub mod ctx;

pub use into_hir::IntoHir;
pub use into_lir::IntoLir;
