use std::collections::HashMap;

use crate::ast::pattern::Pattern;
use crate::common::scope::Scope;

impl Pattern {
    /// Fill the context with locally defined elements in the pattern.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::pattern::Pattern;
    /// use grustine::common::{constant::Constant, location::Location, scope::Scope};
    ///
    /// let mut elements_context = HashMap::new();
    ///
    /// let given_pattern = Pattern::Structure {
    ///     name: String::from("Point"),
    ///     fields: vec![
    ///         (
    ///             String::from("x"),
    ///             Pattern::Constant {
    ///                 constant: Constant::Integer(1),
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("y"),
    ///             Pattern::Identifier {
    ///                 name: String::from("y"),
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// given_pattern.fill_context(&mut elements_context);
    ///
    /// let y = String::from("y");
    /// assert_eq!(elements_context[&y], Scope::Local);
    /// ```
    pub fn fill_context(
        &self,
        elements_context: &mut HashMap<String, Scope>,
    ) {
        match self {
            Pattern::Identifier { name, .. } => assert!(elements_context.insert(
                name.clone(),
                Scope::Local,
            ).is_none()),
            Pattern::Structure {
                fields,
                ..
            } => fields.iter().for_each(|(_, pattern)| pattern.fill_context(elements_context)),
            Pattern::Some { pattern, .. } => pattern.fill_context(elements_context),
            _ => ()
        }
    }
}
