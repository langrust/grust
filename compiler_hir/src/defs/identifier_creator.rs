//! HIR [IdentifierCreator](crate::hir::identifier_creator::IdentifierCreator) module.

use std::collections::HashSet;

/// Identifier creator used to create fresh identifiers.
#[derive(Debug, PartialEq)]
pub struct IdentifierCreator {
    /// Already known identifiers.
    pub identifiers: HashSet<String>,
}
impl IdentifierCreator {
    /// Create a new identifier creator from a list of identifiers.
    ///
    /// It will store all existing id from the list.
    pub fn from(identifiers: impl IntoIterator<Item = String>) -> Self {
        IdentifierCreator {
            identifiers: HashSet::from_iter(identifiers),
        }
    }
    fn already_defined(&self, identifier: &String) -> bool {
        self.identifiers.contains(identifier)
    }
    fn add_identifier(&mut self, identifier: &str) {
        self.identifiers.insert(identifier.to_string());
    }

    /// Create new identifier from request.
    ///
    /// If the requested identifier is not used then return it. Otherwise, it creates a fresh
    /// identifier from this request.
    ///
    /// # Example
    ///
    /// If `mem_x` is requested as new identifier for the node defined below, then it will return it
    /// as it is.
    ///
    /// But if it request `mem_x` a second time, then it will return `mem_x_1`.
    ///
    /// ```GR
    /// node test(i1: int) {
    ///     x: int = i1;
    ///     out o1: int = x;
    /// }
    /// ```
    pub fn new_identifier_with(&mut self, prefix: &str, name: &str, suffix: &str) -> String {
        let sep1 = if !(prefix.is_empty() || prefix.ends_with('_')) {
            "_"
        } else {
            ""
        };
        let sep2 = if !(suffix.is_empty() || suffix.starts_with('_')) {
            "_"
        } else {
            ""
        };
        let mut identifier = format!("{prefix}{sep1}{name}{sep2}{suffix}");

        let mut counter = 1;
        while self.already_defined(&identifier) {
            identifier = format!("{prefix}{sep1}{name}_{}{sep2}{suffix}", counter);
            counter += 1;
        }

        self.add_identifier(&identifier);
        identifier
    }

    /// Same as [Self::new_identifier_with] with empty prefix and suffix.
    pub fn new_identifier(&mut self, name: &str) -> String {
        self.new_identifier_with("", name, "")
    }

    pub fn fresh_identifier(&mut self, kind: &str, name: &str) -> String {
        self.new_identifier_with(kind, name, "")
    }

    /// Create new type identifier.
    pub fn new_type_identifier(&mut self, type_name: impl Into<String>) -> String {
        let mut type_name = type_name.into();
        let mut counter = 1;
        while self.already_defined(&type_name) {
            type_name = format!("{type_name}{counter}");
            counter += 1;
        }

        self.add_identifier(&type_name);
        type_name
    }
}
