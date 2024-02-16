use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::pattern::{Pattern, PatternKind};
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for Pattern {
    /// Check if `self` pattern matches the expected [Type]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{pattern::Pattern, typedef::Typedef};
    /// use grustine::common::{constant::Constant::Location, r#type::Type};
    ///
    /// let mut errors = vec![];
    /// let mut user_types_context = HashMap::new();
    /// let mut elements_context = HashMap::new();
    ///
    /// user_types_context.insert(
    ///    String::from("Point"),
    ///     Typedef::Structure {
    ///         id: String::from("Point"),
    ///         fields: vec![
    ///             (String::from("x"), Type::Integer),
    ///             (String::from("y"), Type::Integer),
    ///         ],
    ///         location: Location::default(),
    ///     },
    /// );
    ///
    /// let given_pattern = PatternKind::Structure {
    ///     name: String::from("Point"),
    ///     fields: vec![
    ///         (
    ///             String::from("x"),
    ///             PatternKind::Constant {
    ///                 constant: Constant::Integer(1),
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("y"),
    ///             PatternKind::Identifier {
    ///                 name: String::from("y"),
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    /// let expected_type = Type::Structure(String::from("Point"));
    ///
    /// given_pattern.construct_context(&expected_type, &mut elements_context, &user_types_context, &mut errors).unwrap();
    ///
    /// let y = String::from("y");
    /// assert_eq!(elements_context[&y], Type::Integer);
    /// assert!(errors.is_empty());
    /// ```
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            PatternKind::Constant { ref constant } => {
                self.typing = Some(constant.get_type());
                Ok(())
            }
            PatternKind::Identifier { ref id } => {
                self.typing = Some(symbol_table.get_type(id).clone());
                Ok(())
            }
            PatternKind::Structure {
                ref id,
                ref mut fields,
            } => {
                fields
                    .iter_mut()
                    .map(|(id, pattern)| {
                        pattern.typing(symbol_table, errors)?;
                        let pattern_type = pattern.get_type().unwrap();
                        let expected_type = symbol_table.get_type(id);
                        pattern_type.eq_check(expected_type, self.location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;
                self.typing = Some(Type::Structure {
                    name: symbol_table.get_name(id).clone(),
                    id: *id,
                });
                Ok(())
            }
            PatternKind::Enumeration { ref enum_id, .. } => {
                self.typing = Some(Type::Enumeration {
                    name: symbol_table.get_name(enum_id).clone(),
                    id: *enum_id,
                });
                Ok(())
            }
            PatternKind::Tuple { ref mut elements } => {
                elements
                    .iter_mut()
                    .map(|pattern| pattern.typing(symbol_table, errors))
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;
                let types = elements
                    .iter()
                    .map(|pattern| pattern.get_type().unwrap().clone())
                    .collect();
                self.typing = Some(Type::Tuple(types));
                Ok(())
            }
            PatternKind::Some { ref mut pattern } => {
                pattern.typing(symbol_table, errors)?;
                let pattern_type = pattern.get_type().unwrap().clone();
                self.typing = Some(Type::Option(Box::new(pattern_type)));
                Ok(())
            }
            PatternKind::None => {
                self.typing = Some(Type::Option(Box::new(Type::Any)));
                Ok(())
            }
            PatternKind::Default => Ok(()),
        }
    }
}
