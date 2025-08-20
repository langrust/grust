//! [IdentifierCreator] module.

prelude! {}

use std::collections::HashSet;

/// Identifier creator used to create fresh identifiers.
#[derive(Debug, PartialEq)]
pub struct IdentifierCreator {
    /// Already known identifiers.
    pub identifiers: HashSet<Ident>,
}
impl IdentifierCreator {
    /// Create a new identifier creator from a list of identifiers.
    ///
    /// It will store all existing id from the list.
    pub fn from(identifiers: impl IntoIterator<Item = Ident>) -> Self {
        IdentifierCreator {
            identifiers: HashSet::from_iter(identifiers),
        }
    }
    fn already_defined(&self, identifier: &Ident) -> bool {
        self.identifiers.contains(identifier)
    }
    fn add_identifier(&mut self, identifier: Ident) {
        self.identifiers.insert(identifier);
    }

    /// Create new identifier from request.
    ///
    /// If the requested identifier is not used then return it. Otherwise, it creates a fresh
    /// identifier from this request.
    ///
    /// # Example
    ///
    /// If `mem_x` is requested as new identifier for the component defined below, then it will return it
    /// as it is.
    ///
    /// But if it request `mem_x` a second time, then it will return `mem_x_1`.
    ///
    /// ```GR
    /// component test(i1: int) {
    ///     x: int = i1;
    ///     out o1: int = x;
    /// }
    /// ```
    pub fn new_identifier_with(
        &mut self,
        loc: impl Into<Loc>,
        prefix: impl AsRef<str>,
        name: impl AsRef<str>,
        suffix: impl AsRef<str>,
    ) -> Ident {
        let (prefix, name, suffix) = (prefix.as_ref(), name.as_ref(), suffix.as_ref());
        let loc = loc.into();
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
        loop {
            let ident = Ident::new(&identifier, loc.into());
            if self.already_defined(&ident) {
                identifier = format!("{prefix}{sep1}{name}_{}{sep2}{suffix}", counter);
                counter += 1;
                continue;
            } else {
                self.add_identifier(ident.clone());
                return ident;
            }
        }
    }

    /// Same as [Self::new_identifier_with] with empty prefix and suffix.
    pub fn new_identifier(&mut self, loc: impl Into<Loc>, name: impl AsRef<str>) -> Ident {
        self.new_identifier_with(loc, "", name, "")
    }

    pub fn fresh_identifier(
        &mut self,
        loc: impl Into<Loc>,
        kind: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> Ident {
        self.new_identifier_with(loc, kind, name, "")
    }

    /// Create new type identifier.
    pub fn new_type_identifier(
        &mut self,
        loc: impl Into<Loc>,
        type_name: impl Into<String>,
    ) -> Ident {
        let loc = loc.into();
        let mut type_name = type_name.into();
        let mut counter = 1;
        loop {
            let ident = Ident::new(&type_name, loc.into());
            if self.already_defined(&ident) {
                type_name = format!("{type_name}{counter}");
                counter += 1;
                continue;
            } else {
                self.add_identifier(ident.clone());
                return ident;
            }
        }
    }
}
