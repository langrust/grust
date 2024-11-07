//! HIR [Dependencies](crate::hir::dependencies::Dependencies) module.

prelude! {
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
    /// # compiler_hir::prelude! {}
    /// let dependencies = Dependencies::new();
    /// assert!(dependencies.get().is_none());
    /// ```
    pub fn new() -> Self {
        Dependencies(OnceCell::new())
    }

    /// Create dependencies according to input.
    ///
    /// ```rust
    /// # compiler_hir::prelude! {}
    /// let v = vec![(1, graph::Label::Weight(0))];
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
    /// # compiler_hir::prelude! {}
    /// let v = vec![(1, graph::Label::Weight(0))];
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
    /// Get some dependencies if it as been previously resolved. Return `None` otherwise.
    ///
    /// ```rust
    /// # compiler_hir::prelude! {}
    /// let v = vec![(1, graph::Label::Weight(0))];
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

    /// Replace identifier occurrence by dependencies of element in context.
    ///
    /// It will modify the dependencies according to the context:
    ///
    /// - if an identifier is mapped to another identifier, then rename all occurrence of the
    ///   identifier by the new one
    /// - if the identifier is mapped to an expression, then replace all call to the identifier by
    ///   the dependencies of the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2]`, the expression `x + y` which depends on `x` and `y`
    /// will depends on `a` and `b`.
    pub fn replace_by_context(
        &mut self,
        context_map: &HashMap<usize, Either<usize, stream::Expr>>,
    ) {
        let new_dependencies = self
            .get()
            .unwrap()
            .iter()
            .flat_map(|(id, label)| match context_map.get(id) {
                Some(Either::Left(new_id)) => vec![(*new_id, label.clone())],
                Some(Either::Right(expression)) => expression
                    .get_dependencies()
                    .iter()
                    .map(|(new_id, new_label)| (new_id.clone(), label.add(new_label)))
                    .collect(),
                None => vec![],
            })
            .collect();

        *self = Dependencies::from(new_dependencies);
    }
}
impl Default for Dependencies {
    fn default() -> Self {
        Self::new()
    }
}
