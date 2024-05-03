use strum::EnumIter;
use syn::{parse::Parse, Token};

use crate::{common::r#type::Type, error::Error};

use super::location::Location;

/// GRust binary operators.
///
/// [BinaryOperator] enumeration represents all possible binary operations
/// that can be used in a GRust program:
/// - [BinaryOperator::Mul] is the multiplication `*`
/// - [BinaryOperator::Div], the division `/`
/// - [BinaryOperator::Add], addition `+`
/// - [BinaryOperator::Sub], substraction `-`
/// - [BinaryOperator::And], logical "and" `&&`
/// - [BinaryOperator::Or], logical "or" `||`
/// - [BinaryOperator::Eq], equality test `==`
/// - [BinaryOperator::Dif], inequality test `!=`
/// - [BinaryOperator::Geq], "greater or equal" `>=`
/// - [BinaryOperator::Leq], "lower or equal" `<=`
/// - [BinaryOperator::Grt], "greater" `>`
/// - [BinaryOperator::Low], "lower" `<`
#[derive(EnumIter, Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    /// Multiplication, `x * y`.
    Mul,
    /// Division, `x / y`.
    Div,
    /// Addition, `x + y`.
    Add,
    /// Substraction, `x - y`.
    Sub,
    /// Logical "and", `x && y`.
    And,
    /// Logical "or", `x || y`.
    Or,
    /// Equality test, `x == y`.
    Eq,
    /// Inequality test, `x != y`.
    Dif,
    /// Test "greater or equal", `x >= y`.
    Geq,
    /// Test "lower or equal", `x <= y`.
    Leq,
    /// Test "greater", `x > y`.
    Grt,
    /// Test "lower", `x < y`.
    Low,
}
impl BinaryOperator {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![*])
            || input.peek(Token![/])
            || input.peek(Token![+])
            || input.peek(Token![-])
            || input.peek(Token![&&])
            || input.peek(Token![||])
            || input.peek(Token![==])
            || input.peek(Token![!=])
            || input.peek(Token![>=])
            || input.peek(Token![<=])
            || input.peek(Token![>])
            || input.peek(Token![<])
    }
}
impl Parse for BinaryOperator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            Ok(BinaryOperator::Mul)
        } else if input.peek(Token![/]) {
            let _: Token![/] = input.parse()?;
            Ok(BinaryOperator::Div)
        } else if input.peek(Token![+]) {
            let _: Token![+] = input.parse()?;
            Ok(BinaryOperator::Add)
        } else if input.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(BinaryOperator::Sub)
        } else if input.peek(Token![&&]) {
            let _: Token![&&] = input.parse()?;
            Ok(BinaryOperator::And)
        } else if input.peek(Token![||]) {
            let _: Token![||] = input.parse()?;
            Ok(BinaryOperator::Or)
        } else if input.peek(Token![==]) {
            let _: Token![==] = input.parse()?;
            Ok(BinaryOperator::Eq)
        } else if input.peek(Token![!=]) {
            let _: Token![!=] = input.parse()?;
            Ok(BinaryOperator::Dif)
        } else if input.peek(Token![>=]) {
            let _: Token![>=] = input.parse()?;
            Ok(BinaryOperator::Geq)
        } else if input.peek(Token![<=]) {
            let _: Token![<=] = input.parse()?;
            Ok(BinaryOperator::Leq)
        } else if input.peek(Token![>]) {
            let _: Token![>] = input.parse()?;
            Ok(BinaryOperator::Grt)
        } else if input.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            Ok(BinaryOperator::Low)
        } else {
            Err(input.error("expected binary operators"))
        }
    }
}
impl ToString for BinaryOperator {
    fn to_string(&self) -> String {
        match self {
            BinaryOperator::Mul => String::from(" * "),
            BinaryOperator::Div => String::from(" / "),
            BinaryOperator::Add => String::from(" + "),
            BinaryOperator::Sub => String::from(" - "),
            BinaryOperator::And => String::from(" && "),
            BinaryOperator::Or => String::from(" || "),
            BinaryOperator::Eq => String::from(" == "),
            BinaryOperator::Dif => String::from(" != "),
            BinaryOperator::Geq => String::from(" >= "),
            BinaryOperator::Leq => String::from(" <= "),
            BinaryOperator::Grt => String::from(" > "),
            BinaryOperator::Low => String::from(" < "),
        }
    }
}
impl BinaryOperator {
    fn numerical_operator(mut input_types: Vec<Type>, location: Location) -> Result<Type, Error> {
        if input_types.len() == 2 {
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 != Type::Float && type_1 != Type::Integer {
                let error = Error::ExpectNumber {
                    given_type: type_1,
                    location,
                };
                return Err(error);
            };
            if type_2 != Type::Float && type_2 != Type::Integer {
                let error = Error::ExpectNumber {
                    given_type: type_2,
                    location,
                };
                return Err(error);
            };
            if type_1 != type_2 {
                let error = Error::IncompatibleType {
                    given_type: type_2,
                    expected_type: type_1,
                    location,
                };
                return Err(error);
            };
            Ok(Type::Abstract(
                vec![type_1.clone(), type_2],
                Box::new(type_1),
            ))
        } else {
            let error = Error::IncompatibleInputsNumber {
                given_inputs_number: input_types.len(),
                expected_inputs_number: 2,
                location,
            };
            Err(error)
        }
    }

    fn numerical_comparison(mut input_types: Vec<Type>, location: Location) -> Result<Type, Error> {
        if input_types.len() == 2 {
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 != Type::Float && type_1 != Type::Integer {
                let error = Error::ExpectNumber {
                    given_type: type_1,
                    location,
                };
                return Err(error);
            };
            if type_2 != Type::Float && type_2 != Type::Integer {
                let error = Error::ExpectNumber {
                    given_type: type_2,
                    location,
                };
                return Err(error);
            };
            if type_1 != type_2 {
                let error = Error::IncompatibleType {
                    given_type: type_2,
                    expected_type: type_1,
                    location,
                };
                return Err(error);
            };
            Ok(Type::Abstract(
                vec![type_1, type_2],
                Box::new(Type::Boolean),
            ))
        } else {
            let error = Error::IncompatibleInputsNumber {
                given_inputs_number: input_types.len(),
                expected_inputs_number: 2,
                location,
            };
            Err(error)
        }
    }

    fn equality(mut input_types: Vec<Type>, location: Location) -> Result<Type, Error> {
        if input_types.len() == 2 {
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 == type_2 {
                Ok(Type::Abstract(
                    vec![type_1, type_2],
                    Box::new(Type::Boolean),
                ))
            } else {
                let error = Error::IncompatibleType {
                    given_type: type_2,
                    expected_type: type_1,
                    location,
                };
                Err(error)
            }
        } else {
            let error = Error::IncompatibleInputsNumber {
                given_inputs_number: input_types.len(),
                expected_inputs_number: 2,
                location,
            };
            Err(error)
        }
    }

    /// Get the [Type] of a binary operator.
    ///
    /// # Example
    /// ```rust
    /// use grustine::common::operator::BinaryOperator;
    ///
    /// let add_type = BinaryOperator::Add.get_type();
    /// ```
    pub fn get_type(&self) -> Type {
        match self {
            // If self is an operator over numbers then its type can either
            // be `int -> int -> int` or `float -> float -> float`
            // then it is a [Type::Polymorphism]
            BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Add
            | BinaryOperator::Sub => Type::Polymorphism(BinaryOperator::numerical_operator),
            // If self is a comparison over numbers then its type can either
            // be `int -> int -> bool` or `float -> float -> bool`
            // then it is a [Type::Polymorphism]
            BinaryOperator::Geq
            | BinaryOperator::Leq
            | BinaryOperator::Grt
            | BinaryOperator::Low => Type::Polymorphism(BinaryOperator::numerical_comparison),
            // If self is an equality or inequality test then its type can
            // be `t -> t -> bool` for any t
            // then it is a [Type::Polymorphism]
            BinaryOperator::Eq | BinaryOperator::Dif => {
                Type::Polymorphism(BinaryOperator::equality)
            }
            // If self is a logical operator then its type
            // is `bool -> bool -> bool`
            BinaryOperator::And | BinaryOperator::Or => {
                Type::Abstract(vec![Type::Boolean, Type::Boolean], Box::new(Type::Boolean))
            }
        }
    }
}

/// GRust unary operators.
///
/// [UnaryOperator] enumeration represents all possible unary operations
/// that can be used in a GRust program:
/// - [UnaryOperator::Neg] is the numerical negation `-`
/// - [UnaryOperator::Not], the logical negation `!`
#[derive(EnumIter, Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    /// Numerical negation, `-x`.
    Neg,
    /// Logical negation, `!x`.
    Not,
}
impl UnaryOperator {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![-]) || input.peek(Token![!])
    }
}
impl Parse for UnaryOperator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(UnaryOperator::Neg)
        } else if input.peek(Token![!]) {
            let _: Token![!] = input.parse()?;
            Ok(UnaryOperator::Not)
        } else {
            Err(input.error("expected '-', or '!' unary operators"))
        }
    }
}
impl ToString for UnaryOperator {
    fn to_string(&self) -> String {
        match self {
            UnaryOperator::Neg => String::from("-"),
            UnaryOperator::Not => String::from("!"),
        }
    }
}
impl UnaryOperator {
    fn numerical_negation(mut input_types: Vec<Type>, location: Location) -> Result<Type, Error> {
        if input_types.len() == 1 {
            let type_1 = input_types.pop().unwrap();
            if type_1 == Type::Float || type_1 == Type::Integer {
                Ok(Type::Abstract(vec![type_1.clone()], Box::new(type_1)))
            } else {
                let error = Error::ExpectNumber {
                    given_type: type_1,
                    location,
                };
                Err(error)
            }
        } else {
            let error = Error::IncompatibleInputsNumber {
                given_inputs_number: input_types.len(),
                expected_inputs_number: 2,
                location,
            };
            Err(error)
        }
    }

    fn brackets(mut input_types: Vec<Type>, location: Location) -> Result<Type, Error> {
        if input_types.len() == 1 {
            let type_1 = input_types.pop().unwrap();
            Ok(Type::Abstract(vec![type_1.clone()], Box::new(type_1)))
        } else {
            let error = Error::IncompatibleInputsNumber {
                given_inputs_number: input_types.len(),
                expected_inputs_number: 2,
                location,
            };
            Err(error)
        }
    }

    /// Get the [Type] of a unary operator.
    ///
    /// # Example
    /// ```rust
    /// use grustine::common::operator::UnaryOperator;
    ///
    /// let neg_type = UnaryOperator::Neg.get_type();
    /// ```
    pub fn get_type(&self) -> Type {
        match self {
            // If self is the numerical negation then its type can either
            // be `int -> int` or `float -> float`
            // then it is a [Type::Polymorphism]
            UnaryOperator::Neg => Type::Polymorphism(UnaryOperator::numerical_negation),
            // If self is the logical negation then its type is `bool -> bool`
            UnaryOperator::Not => Type::Abstract(vec![Type::Boolean], Box::new(Type::Boolean)),
        }
    }
}

/// Other builtin operators in GRust.
///
/// [OtherOperator] enumeration represents all other operations
/// that can be used in a GRust program:
/// - [OtherOperator::IfThenElse] is `if _ then _ else _`
/// - [OtherOperator::Print] is the usual `print` function
#[derive(EnumIter)]
pub enum OtherOperator {
    /// The `if b then x else y` GRust expression.
    IfThenElse,
}
impl ToString for OtherOperator {
    fn to_string(&self) -> String {
        match self {
            OtherOperator::IfThenElse => String::from("if_then_else"),
        }
    }
}
impl OtherOperator {
    fn if_then_else(mut input_types: Vec<Type>, location: Location) -> Result<Type, Error> {
        if input_types.len() == 3 {
            let type_3 = input_types.pop().unwrap();
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 != Type::Boolean {
                let error = Error::IncompatibleType {
                    given_type: type_1,
                    expected_type: Type::Boolean,
                    location,
                };
                return Err(error);
            };
            if type_2 != type_3 {
                let error = Error::IncompatibleType {
                    given_type: type_3,
                    expected_type: type_2,
                    location,
                };
                return Err(error);
            };
            Ok(Type::Abstract(
                vec![type_1, type_2.clone(), type_3],
                Box::new(type_2),
            ))
        } else {
            let error = Error::IncompatibleInputsNumber {
                given_inputs_number: input_types.len(),
                expected_inputs_number: 2,
                location,
            };
            Err(error)
        }
    }

    /// Get the [Type] of the other operators.
    ///
    /// # Example
    /// ```rust
    /// use grustine::common::operator::OtherOperator;
    ///
    /// let ifthenelse_type = OtherOperator::IfThenElse.get_type();
    /// ```
    pub fn get_type(&self) -> Type {
        match self {
            // If self is "if _ then _ else _" its type can be
            // `bool -> t -> t` for any type t
            // then it is a [Type::Polymorphism]
            OtherOperator::IfThenElse => Type::Polymorphism(OtherOperator::if_then_else),
        }
    }
}

#[cfg(test)]
mod to_string {
    use crate::common::operator::{BinaryOperator, OtherOperator, UnaryOperator};

    #[test]
    fn should_convert_negation_operator_to_string() {
        assert_eq!(String::from("-"), UnaryOperator::Neg.to_string());
    }
    #[test]
    fn should_convert_not_operator_to_string() {
        assert_eq!(String::from("!"), UnaryOperator::Not.to_string());
    }

    #[test]
    fn should_convert_multiplication_operator_to_string() {
        assert_eq!(String::from(" * "), BinaryOperator::Mul.to_string());
    }
    #[test]
    fn should_convert_division_operator_to_string() {
        assert_eq!(String::from(" / "), BinaryOperator::Div.to_string());
    }
    #[test]
    fn should_convert_addition_operator_to_string() {
        assert_eq!(String::from(" + "), BinaryOperator::Add.to_string());
    }
    #[test]
    fn should_convert_substraction_operator_to_string() {
        assert_eq!(String::from(" - "), BinaryOperator::Sub.to_string());
    }
    #[test]
    fn should_convert_and_operator_to_string() {
        assert_eq!(String::from(" && "), BinaryOperator::And.to_string());
    }
    #[test]
    fn should_convert_or_operator_to_string() {
        assert_eq!(String::from(" || "), BinaryOperator::Or.to_string());
    }
    #[test]
    fn should_convert_equality_operator_to_string() {
        assert_eq!(String::from(" == "), BinaryOperator::Eq.to_string());
    }
    #[test]
    fn should_convert_difference_operator_to_string() {
        assert_eq!(String::from(" != "), BinaryOperator::Dif.to_string());
    }
    #[test]
    fn should_convert_greater_equal_operator_to_string() {
        assert_eq!(String::from(" >= "), BinaryOperator::Geq.to_string());
    }
    #[test]
    fn should_convert_lower_equal_operator_to_string() {
        assert_eq!(String::from(" <= "), BinaryOperator::Leq.to_string());
    }
    #[test]
    fn should_convert_greater_operator_to_string() {
        assert_eq!(String::from(" > "), BinaryOperator::Grt.to_string());
    }
    #[test]
    fn should_convert_lower_operator_to_string() {
        assert_eq!(String::from(" < "), BinaryOperator::Low.to_string());
    }

    #[test]
    fn should_convert_ifthenelse_operator_to_string() {
        assert_eq!(
            String::from("if_then_else"),
            OtherOperator::IfThenElse.to_string()
        );
    }
}
