/// A thread-safe cell which can be written to only once.
///
/// `OnceCell` provides `&` references to the contents.
///  From the `once_cell` crate API.
///
/// # Example
/// ```
/// use grustine::hir::once_cell::OnceCell;
///
/// let cell: OnceCell<String> = OnceCell::new();
/// assert!(cell.get().is_none());
///
/// cell.set("Hello, World!".to_string()).unwrap();
///
/// let value: Option<&String> = cell.get();
/// assert!(value.is_some());
/// assert_eq!(value.unwrap().as_str(), "Hello, World!");
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct OnceCell<T>(once_cell::sync::OnceCell<T>);

impl<T> OnceCell<T> {
    /// Creates a new empty cell.
    pub fn new() -> Self {
        OnceCell(once_cell::sync::OnceCell::<T>::new())
    }

    /// Converts to [OnceCell] from the input type.
    pub fn from(value: T) -> Self {
        OnceCell(once_cell::sync::OnceCell::<T>::from(value))
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized.
    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.0.get_mut()
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// Returns `Ok(())` if the cell was empty and `Err(value)` if it was
    /// full.
    pub fn set(&self, value: T) -> Result<(), T> {
        self.0.set(value)
    }
}

