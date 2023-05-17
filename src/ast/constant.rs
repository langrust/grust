use std::fmt::{self, Display};

use crate::ast::type_system::Type;

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
/// use grustine::ast::constant::Constant;
/// let constant = Constant::String(String::from("Hello world"));
/// ```
#[derive(Debug, PartialEq, Clone)]
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
    /// Enumeration constant
    Enumeration(String, String),
}
impl Constant {
    /// Get the [Type] of the constant.
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{constant::Constant, type_system::Type};
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
            Constant::Enumeration(e, _) => Type::Enumeration(e.clone()),
        }
    }
}
impl Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Integer(i) => write!(f, "{i}i64"),
            Constant::Float(l) => write!(f, "{l}f64"),
            Constant::Boolean(b) => write!(f, "{b}"),
            Constant::String(s) => write!(f, "\"{s}\""),
            Constant::Unit => write!(f, "()"),
            Constant::Enumeration(enu, elem) => write!(f, "{enu}::{elem}"),
        }
    }
}
