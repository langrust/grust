use strum::EnumIter;

prelude! {
    syn::{Parse, BinOp},
}

/// GRust binary operators.
///
/// [BinaryOperator] enumeration represents all possible binary operations that can be used in a
/// GRust program:
///
/// - [BinaryOperator::Mul] is the multiplication `*`
/// - [BinaryOperator::Div], the division `/`
/// - [BinaryOperator::Add], addition `+`
/// - [BinaryOperator::Sub], subtraction `-`
/// - [BinaryOperator::And], logical "and" `&&`
/// - [BinaryOperator::Or], logical "or" `||`
/// - [BinaryOperator::Eq], equality test `==`
/// - [BinaryOperator::Dif], inequality test `!=`
/// - [BinaryOperator::Geq], "greater or equal" `>=`
/// - [BinaryOperator::Leq], "lower or equal" `<=`
/// - [BinaryOperator::Grt], "greater" `>`
/// - [BinaryOperator::Low], "lower" `<`
#[derive(EnumIter, Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    /// Multiplication, `x * y`.
    Mul,
    /// Division, `x / y`.
    Div,
    /// Modulo, `x % y`.
    Mod,
    /// Addition, `x + y`.
    Add,
    /// Subtraction, `x - y`.
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
    /// The `syn` version of an operator.
    pub fn into_syn(self) -> BinOp {
        match self {
            Self::Mul => BinOp::Mul(Default::default()),
            Self::Div => BinOp::Div(Default::default()),
            Self::Mod => BinOp::Rem(Default::default()),
            Self::Add => BinOp::Add(Default::default()),
            Self::Sub => BinOp::Sub(Default::default()),
            Self::And => BinOp::And(Default::default()),
            Self::Or => BinOp::Or(Default::default()),
            Self::Eq => BinOp::Eq(Default::default()),
            Self::Dif => BinOp::Ne(Default::default()),
            Self::Geq => BinOp::Ge(Default::default()),
            Self::Leq => BinOp::Le(Default::default()),
            Self::Grt => BinOp::Gt(Default::default()),
            Self::Low => BinOp::Lt(Default::default()),
        }
    }

    pub fn peek(input: ParseStream) -> bool {
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
    pub fn peek_prec1(input: ParseStream) -> bool {
        input.peek(Token![*]) || input.peek(Token![/])
    }
    pub fn peek_prec2(input: ParseStream) -> bool {
        input.peek(Token![+]) || input.peek(Token![-])
    }
    pub fn peek_prec3(input: ParseStream) -> bool {
        input.peek(Token![==])
            || input.peek(Token![!=])
            || input.peek(Token![>=])
            || input.peek(Token![<=])
            || input.peek(Token![>])
            || input.peek(Token![<])
    }
    pub fn peek_prec4(input: ParseStream) -> bool {
        input.peek(Token![&&]) || input.peek(Token![||])
    }
}
impl Parse for BinaryOperator {
    fn parse(input: ParseStream) -> syn::Res<Self> {
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
impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Mul => " * ".fmt(f),
            BinaryOperator::Div => " / ".fmt(f),
            BinaryOperator::Mod => " % ".fmt(f),
            BinaryOperator::Add => " + ".fmt(f),
            BinaryOperator::Sub => " - ".fmt(f),
            BinaryOperator::And => " && ".fmt(f),
            BinaryOperator::Or => " || ".fmt(f),
            BinaryOperator::Eq => " == ".fmt(f),
            BinaryOperator::Dif => " != ".fmt(f),
            BinaryOperator::Geq => " >= ".fmt(f),
            BinaryOperator::Leq => " <= ".fmt(f),
            BinaryOperator::Grt => " > ".fmt(f),
            BinaryOperator::Low => " < ".fmt(f),
        }
    }
}
impl BinaryOperator {
    fn numerical_operator(mut input_types: Vec<Typ>, location: Location) -> Res<Typ> {
        if input_types.len() == 2 {
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 != Typ::float() && type_1 != Typ::int() {
                let error = Error::ExpectNumber {
                    given_type: type_1,
                    location,
                };
                return Err(error);
            };
            if type_2 != Typ::float() && type_2 != Typ::int() {
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
            Ok(Typ::function(vec![type_1.clone(), type_2], type_1))
        } else {
            let error = Error::ArityMismatch {
                input_count: input_types.len(),
                arity: 2,
                location,
            };
            Err(error)
        }
    }

    fn numerical_comparison(mut input_types: Vec<Typ>, location: Location) -> Res<Typ> {
        if input_types.len() == 2 {
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 != Typ::float() && type_1 != Typ::int() {
                let error = Error::ExpectNumber {
                    given_type: type_1,
                    location,
                };
                return Err(error);
            };
            if type_2 != Typ::float() && type_2 != Typ::int() {
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
            Ok(Typ::function(vec![type_1, type_2], Typ::bool()))
        } else {
            let error = Error::ArityMismatch {
                input_count: input_types.len(),
                arity: 2,
                location,
            };
            Err(error)
        }
    }

    fn equality(mut input_types: Vec<Typ>, location: Location) -> Res<Typ> {
        if input_types.len() == 2 {
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 == type_2 {
                Ok(Typ::function(vec![type_1, type_2], Typ::bool()))
            } else {
                let error = Error::IncompatibleType {
                    given_type: type_2,
                    expected_type: type_1,
                    location,
                };
                Err(error)
            }
        } else {
            let error = Error::ArityMismatch {
                input_count: input_types.len(),
                arity: 2,
                location,
            };
            Err(error)
        }
    }

    /// Get the [Typ] of a binary operator.
    ///
    /// # Example
    ///
    /// ```rust
    /// # compiler_common::prelude! { operator::BinaryOperator }
    /// let add_type = BinaryOperator::Add.get_type();
    /// assert!(add_type.is_polymorphic());
    /// ```
    pub fn get_type(&self) -> Typ {
        match self {
            // If self is an operator over numbers then its type can either be `int -> int -> int`
            // or `float -> float -> float` then it is a [Typ::Polymorphism]
            BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Mod
            | BinaryOperator::Add
            | BinaryOperator::Sub => Typ::Polymorphism(BinaryOperator::numerical_operator),
            // If self is a comparison over numbers then its type can either be `int -> int -> bool`
            // or `float -> float -> bool` then it is a [Typ::Polymorphism]
            BinaryOperator::Geq
            | BinaryOperator::Leq
            | BinaryOperator::Grt
            | BinaryOperator::Low => Typ::Polymorphism(BinaryOperator::numerical_comparison),
            // If self is an equality or inequality test then its type can be `t -> t -> bool` for
            // any t then it is a [Typ::Polymorphism]
            BinaryOperator::Eq | BinaryOperator::Dif => Typ::Polymorphism(BinaryOperator::equality),
            // If self is a logical operator then its type is `bool -> bool -> bool`
            BinaryOperator::And | BinaryOperator::Or => {
                Typ::function(vec![Typ::bool(), Typ::bool()], Typ::bool())
            }
        }
    }
}

/// GRust unary operators.
///
/// [UnaryOperator] enumeration represents all possible unary operations that can be used in a GRust
/// program:
///
/// - [UnaryOperator::Neg] is the numerical negation `-`
/// - [UnaryOperator::Not], the logical negation `!`
#[derive(EnumIter, Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    /// Numerical negation, `-x`.
    Neg,
    /// Logical negation, `!x`.
    Not,
}
impl UnaryOperator {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![-]) || input.peek(Token![!])
    }

    /// The `syn` version of an operator.
    pub fn into_syn(self) -> syn::UnOp {
        match self {
            Self::Neg => syn::UnOp::Neg(Default::default()),
            Self::Not => syn::UnOp::Not(Default::default()),
        }
    }
}
impl Parse for UnaryOperator {
    fn parse(input: ParseStream) -> syn::Res<Self> {
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
impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Neg => "-".fmt(f),
            UnaryOperator::Not => "!".fmt(f),
        }
    }
}
impl UnaryOperator {
    fn numerical_negation(mut input_types: Vec<Typ>, location: Location) -> Res<Typ> {
        if input_types.len() == 1 {
            let type_1 = input_types.pop().unwrap();
            if type_1 == Typ::float() || type_1 == Typ::int() {
                Ok(Typ::function(vec![type_1.clone()], type_1))
            } else {
                let error = Error::ExpectNumber {
                    given_type: type_1,
                    location,
                };
                Err(error)
            }
        } else {
            let error = Error::ArityMismatch {
                input_count: input_types.len(),
                arity: 1,
                location,
            };
            Err(error)
        }
    }

    /// Get the [Typ] of a unary operator.
    ///
    /// # Example
    ///
    /// ```rust
    /// # compiler_common::prelude! { operator::UnaryOperator }
    /// let neg_type = UnaryOperator::Neg.get_type();
    /// assert!(neg_type.is_polymorphic());
    /// ```
    pub fn get_type(&self) -> Typ {
        match self {
            // If self is the numerical negation then its type can either
            // be `int -> int` or `float -> float`
            // then it is a [Typ::Polymorphism]
            UnaryOperator::Neg => Typ::Polymorphism(UnaryOperator::numerical_negation),
            // If self is the logical negation then its type is `bool -> bool`
            UnaryOperator::Not => Typ::function(vec![Typ::bool()], Typ::bool()),
        }
    }
}

/// Other builtin operators in GRust.
///
/// [OtherOperator] enumeration represents all other operations that can be used in a GRust program:
///
/// - [OtherOperator::IfThenElse] is `if _ then _ else _`
#[derive(EnumIter)]
pub enum OtherOperator {
    /// The `if b then x else y` GRust expression.
    IfThenElse,
}
impl std::fmt::Display for OtherOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OtherOperator::IfThenElse => "if_then_else".fmt(f),
        }
    }
}
impl OtherOperator {
    fn if_then_else(mut input_types: Vec<Typ>, location: Location) -> Res<Typ> {
        if input_types.len() == 3 {
            let type_3 = input_types.pop().unwrap();
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 != Typ::bool() {
                let error = Error::IncompatibleType {
                    given_type: type_1,
                    expected_type: Typ::bool(),
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
            Ok(Typ::function(vec![type_1, type_2.clone(), type_3], type_2))
        } else {
            let error = Error::ArityMismatch {
                input_count: input_types.len(),
                arity: 1,
                location,
            };
            Err(error)
        }
    }

    /// Get the [Typ] of the other operators.
    ///
    /// # Example
    ///
    /// ```rust
    /// # compiler_common::prelude! { operator::OtherOperator }
    /// let ifthenelse_type = OtherOperator::IfThenElse.get_type();
    /// assert!(ifthenelse_type.is_polymorphic());
    /// ```
    pub fn get_type(&self) -> Typ {
        match self {
            // If self is "if _ then _ else _" its type can be
            // `bool -> t -> t` for any type t
            // then it is a [Typ::Polymorphism]
            OtherOperator::IfThenElse => Typ::Polymorphism(OtherOperator::if_then_else),
        }
    }
}

#[cfg(test)]
mod to_string {
    prelude! { just
        operator::{BinaryOperator, OtherOperator, UnaryOperator},
    }

    #[test]
    fn should_convert_negation_operator_to_string() {
        assert_eq!("-", UnaryOperator::Neg.to_string());
    }
    #[test]
    fn should_convert_not_operator_to_string() {
        assert_eq!("!", UnaryOperator::Not.to_string());
    }

    #[test]
    fn should_convert_multiplication_operator_to_string() {
        assert_eq!(" * ", BinaryOperator::Mul.to_string());
    }
    #[test]
    fn should_convert_division_operator_to_string() {
        assert_eq!(" / ", BinaryOperator::Div.to_string());
    }
    #[test]
    fn should_convert_addition_operator_to_string() {
        assert_eq!(" + ", BinaryOperator::Add.to_string());
    }
    #[test]
    fn should_convert_substraction_operator_to_string() {
        assert_eq!(" - ", BinaryOperator::Sub.to_string());
    }
    #[test]
    fn should_convert_and_operator_to_string() {
        assert_eq!(" && ", BinaryOperator::And.to_string());
    }
    #[test]
    fn should_convert_or_operator_to_string() {
        assert_eq!(" || ", BinaryOperator::Or.to_string());
    }
    #[test]
    fn should_convert_equality_operator_to_string() {
        assert_eq!(" == ", BinaryOperator::Eq.to_string());
    }
    #[test]
    fn should_convert_difference_operator_to_string() {
        assert_eq!(" != ", BinaryOperator::Dif.to_string());
    }
    #[test]
    fn should_convert_greater_equal_operator_to_string() {
        assert_eq!(" >= ", BinaryOperator::Geq.to_string());
    }
    #[test]
    fn should_convert_lower_equal_operator_to_string() {
        assert_eq!(" <= ", BinaryOperator::Leq.to_string());
    }
    #[test]
    fn should_convert_greater_operator_to_string() {
        assert_eq!(" > ", BinaryOperator::Grt.to_string());
    }
    #[test]
    fn should_convert_lower_operator_to_string() {
        assert_eq!(" < ", BinaryOperator::Low.to_string());
    }

    #[test]
    fn should_convert_ifthenelse_operator_to_string() {
        assert_eq!("if_then_else", OtherOperator::IfThenElse.to_string());
    }
}
