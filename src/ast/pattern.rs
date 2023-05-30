use crate::ast::{constant::Constant, location::Location, type_system::Type};
use crate::error::Error;

use std::fmt::{self, Display};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust matching pattern AST.
pub enum Pattern {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier name.
        name: String,
        /// Pattern location.
        location: Location,
    },
    /// Constant pattern, matches le given constant.
    Constant {
        /// The matching constant.
        constant: Constant,
        /// Pattern location.
        location: Location,
    },
    /// Structure pattern that matches the structure and its fields.
    Structure {
        /// The structure name.
        name: String,
        /// The structure fields with the corresponding patterns to match.
        fields: Vec<(String, Pattern)>,
        /// Pattern location.
        location: Location,
    },
    /// Some pattern that matches when an optional has a value which match the pattern.
    Some {
        /// The pattern matching the value.
        pattern: Box<Pattern>,
        /// Pattern location.
        location: Location,
    },
    /// None pattern, matches when the optional does not have a value.
    None {
        /// Pattern location.
        location: Location,
    },
    /// The default pattern that matches anything.
    Default {
        /// Pattern location.
        location: Location,
    },
}
impl Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Identifier { name, location: _ } => write!(f, "{}", name),
            Pattern::Constant {
                constant,
                location: _,
            } => write!(f, "{}", constant),
            Pattern::Structure {
                name,
                fields,
                location: _,
            } => {
                write!(f, "{} {{ ", name)?;
                for (field, pattern) in fields.iter() {
                    write!(f, "{}: {},", field, pattern)?;
                }
                write!(f, " }}")
            }
            Pattern::Some {
                pattern,
                location: _,
            } => write!(f, "some({})", pattern),
            Pattern::None { location: _ } => write!(f, "none"),
            Pattern::Default { location: _ } => write!(f, "_"),
        }
    }
}

impl Pattern {
    pub fn type_check(
        &self,
        expected_type: &Type,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self {
            Pattern::Identifier { name: _, location: _ } => Ok(()),
            Pattern::Constant { constant, location } => {
                constant
                    .get_type()
                    .eq_check(expected_type, location.clone(), errors)
            },
            Pattern::Structure { name, fields: _, location } => {
                let found_type = Type::Structure(name.clone());
                found_type.eq_check(expected_type, location.clone(), errors)
            },
            Pattern::Some { pattern, location } => match expected_type {
                Type::Option(_) => todo!(),
                _ => {
                    let error = Error::IncompatiblePattern {
                        given_type: self.clone(),
                        expected_type: expected_type.clone(),
                        location: location,
                    };
                    errors.push(error.clone());
                    Err(error)
                }
            },
            Pattern::None { location } => todo!(),
            Pattern::Default { location } => todo!(),
        }
    }
}