use std::fmt::{self, Display};

use crate::common::r#type::Type;

/// LanGrust constants.
///
/// [Constant] enumeration is used to describe LanGRust expressions.
///
/// It reprensents all possible constant:
/// - [Constant::Integer] are [i64] integers, `1` becomes `Constant::Integer(1)`
/// - [Constant::Float] are [f64] floats, `1.0` becomes `Constant::Float(1.0)`
/// - [Constant::Boolean] is the [bool] type for booleans, `true` becomes
/// `Constant::Boolean(true)`
/// - [Constant::String] are strings of type [String], `"hello world"` becomes
/// `Constant::String(String::from("hello world"))`
/// - [Constant::Unit] is the unit type, `()` becomes `Constant::Unit`
///
/// # Example
/// ```rust
/// use grustine::common::constant::Constant;
///
/// let constant = Constant::String(String::from("Hello world"));
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Constant {
    /// [i64] integers
    Integer(i64),
    /// [f64] floats
    Float(f64),
    /// [bool] booleans
    Boolean(bool),
    /// [String] strings
    String(String),
    /// Unit constant
    Unit,
}
impl Constant {
    /// Get the [Type] of the constant.
    ///
    /// # Example
    /// ```rust
    /// use grustine::common::{constant::Constant, r#type::Type};
    ///
    /// let c: Constant = Constant::Integer(6);
    /// assert_eq!(c.get_type(), Type::Integer);
    /// ```
    pub fn get_type(&self) -> Type {
        match self {
            Constant::Integer(_) => Type::Integer,
            Constant::Float(_) => Type::Float,
            Constant::Boolean(_) => Type::Boolean,
            Constant::String(_) => Type::String,
            Constant::Unit => Type::Unit,
        }
    }
}
impl Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Constant::Integer(i) => write!(f, "{i}i64"),
            Constant::Float(l) => write!(f, "{l}f64"),
            Constant::Boolean(b) => write!(f, "{b}"),
            Constant::String(s) => write!(f, "\"{s}\""),
            Constant::Unit => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod get_type {
    use crate::common::{constant::Constant, r#type::Type};

    #[test]
    fn should_return_integer_type_to_integer_constant() {
        assert_eq!(Type::Integer, Constant::Integer(0).get_type());
    }
    #[test]
    fn should_return_float_type_to_float_constant() {
        assert_eq!(Type::Float, Constant::Float(0.0).get_type());
    }
    #[test]
    fn should_return_boolean_type_to_boolean_constant() {
        assert_eq!(Type::Boolean, Constant::Boolean(true).get_type());
    }
    #[test]
    fn should_return_string_type_to_string_constant() {
        assert_eq!(
            Type::String,
            Constant::String(String::from("Hello world!")).get_type()
        );
    }
    #[test]
    fn should_return_unit_type_to_unit_constant() {
        assert_eq!(Type::Unit, Constant::Unit.get_type());
    }
}
