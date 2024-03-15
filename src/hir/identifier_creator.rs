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
    pub fn from(identifiers: Vec<String>) -> Self {
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
    /// If the requested identifier is not used then return it.
    /// Otherwise, it create a fresh identifier from this request.
    ///
    /// # Example
    ///
    /// If `mem_x` is requested as new identifier for the node defined bellow,
    /// then it will return it as it is.
    ///
    /// But if it request `mem_x` a second time, then it will return `mem_x_1`.
    ///  
    /// ```GR
    /// node test(i1: int) {
    ///     x: int = i1;
    ///     out o1: int = x;
    /// }
    /// ```
    pub fn new_identifier(
        &mut self,
        mut prefix: String,
        name: String,
        mut suffix: String,
    ) -> String {
        if !(prefix.is_empty() || prefix.ends_with('_')) {
            prefix.push('_');
        }
        if !(suffix.is_empty() || suffix.starts_with('_')) {
            suffix.insert(0, '_');
        }
        let mut identifier = format!("{prefix}{name}{suffix}");

        let mut counter = 1;
        while self.already_defined(&identifier) {
            identifier = format!("{prefix}{name}_{}{suffix}", counter);
            counter += 1;
        }

        self.add_identifier(&identifier);
        identifier
    }

    /// Create new type identifier.
    pub fn new_type_identifier(&mut self, mut type_name: String) -> String {
        let mut counter = 1;
        while self.already_defined(&type_name) {
            type_name = format!("{type_name}{}", counter);
            counter += 1;
        }

        self.add_identifier(&type_name);
        type_name
    }
}
