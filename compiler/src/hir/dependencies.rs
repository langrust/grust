//! HIR [Dependencies](crate::hir::dependencies::Dependencies) module.

prelude! { just
    graph::Label,
}

use super::once_cell::OnceCell;

/// Dependencies structure.
///
/// Dependencies are stored in a OnceCell.
/// This allows to set dependencies after creating the the structure.
/// After setting the value, the dependencies are immutable.
#[derive(Debug, PartialEq, Clone)]
pub struct Dependencies(OnceCell<Vec<(usize, Label)>>);
impl Dependencies {
    /// Create unset dependencies.
    ///
    /// ```rust
    /// # use compiler::hir::Dependencies;
    /// let dependencies = Dependencies::new();
    /// assert!(dependencies.get().is_none());
    /// ```
    pub fn new() -> Self {
        Dependencies(OnceCell::new())
    }

    /// Create dependencies according to input.
    ///
    /// ```rust
    /// # compiler::prelude! {graph::Label, hir::Dependencies}
    /// let v = vec![(1, Label::Weight(0))];
    /// let dependencies = Dependencies::from(v.clone());
    /// assert_eq!(*dependencies.get().unwrap(), v);
    /// ```
    pub fn from(v: Vec<(usize, Label)>) -> Self {
        let cell = OnceCell::new();
        cell.set(v).unwrap();
        Dependencies(cell)
    }

    /// Set dependencies according to input.
    ///
    /// Terminate nicely only if it is the first time
    /// setting the dependencies.
    ///
    /// ```rust
    /// # compiler::prelude! { graph::Label, hir::Dependencies}
    /// let v = vec![(1, Label::Weight(0))];
    /// let dependencies = Dependencies::new();
    /// dependencies.set(v.clone());
    /// assert_eq!(*dependencies.get().unwrap(), v);
    /// ```
    pub fn set(&self, v: Vec<(usize, Label)>) {
        self.0
            .set(v)
            .expect("should be the first time setting dependencies")
    }

    /// Get optional dependencies.
    ///
    /// Get some dependencies if it as been previously setted.
    /// Return `None` otherwise.
    ///
    /// ```rust
    /// # compiler::prelude! {graph::Label, hir::Dependencies}
    /// let v = vec![(1, Label::Weight(0))];
    /// let dependencies = Dependencies::new();
    /// dependencies.set(v.clone());
    /// assert_eq!(*dependencies.get().unwrap(), v);
    ///
    /// let dependencies = Dependencies::new();
    /// assert!(dependencies.get().is_none());
    /// ```
    pub fn get(&self) -> Option<&Vec<(usize, Label)>> {
        self.0.get()
    }
}
impl Default for Dependencies {
    fn default() -> Self {
        Self::new()
    }
}
