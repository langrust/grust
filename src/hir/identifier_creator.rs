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
    ///
    /// This example is tested in the following code.
    ///
    /// ```rust
    /// use grustine::common::{location::Location, scope::Scope, r#type::Type};
    /// use grustine::hir::{
    ///     dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
    ///     memory::Memory, once_cell::OnceCell, identifier::identifier, stream_expression::StreamExpression,
    ///     unitary_node::UnitaryNode,
    /// };
    ///
    /// let unitary_node = UnitaryNode {
    ///     node_id: String::from("test"),
    ///     output_id: String::from("o1"),
    ///     inputs: vec![(String::from("i1"), Type::Integer)],
    ///     equations: vec![
    ///         Equation {
    ///             scope: Scope::Local,
    ///             id: String::from("x"),
    ///             identifier_type: Type::Integer,
    ///             expression: StreamExpression::identifierCall {
    ///                 identifier: identifier {
    ///                     id: String::from("i1"),
    ///                     scope: Scope::Input,
    ///                 },
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///                 dependencies: Dependencies::new(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///         Equation {
    ///             scope: Scope::Output,
    ///             id: String::from("o1"),
    ///             identifier_type: Type::Integer,
    ///             expression: StreamExpression::identifierCall {
    ///                 identifier: identifier {
    ///                     id: String::from("x"),
    ///                     scope: Scope::Local,
    ///                 },
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///                 dependencies: Dependencies::new(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     memory: Memory::new(),
    ///     location: Location::default(),
    ///     graph: OnceCell::new(),
    /// };
    /// let mut identifier_creator = IdentifierCreator::from(unitary_node.get_identifiers());
    ///
    /// let identifier = identifier_creator.new_identifier(String::from("mem"), String::from("x"), String::from(""));
    /// let control = String::from("mem_x");
    /// assert_eq!(identifier, control);
    ///
    /// let identifier = identifier_creator.new_identifier(String::from("mem"), String::from("x"), String::from(""));
    /// let control = String::from("mem_x_1");
    /// assert_eq!(identifier, control)
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
