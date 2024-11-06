use strum::EnumIter;

prelude! {
    syn::{Parse, BinOp},
}

/// GRust binary operators.
///
/// [BOp] enumeration represents all possible binary operations that can be used in a
/// GRust program:
///
/// - [BOp::Mul] is the multiplication `*`
/// - [BOp::Div], the division `/`
/// - [BOp::Add], addition `+`
/// - [BOp::Sub], subtraction `-`
/// - [BOp::And], logical "and" `&&`
/// - [BOp::Or], logical "or" `||`
/// - [BOp::Eq], equality test `==`
/// - [BOp::Dif], inequality test `!=`
/// - [BOp::Geq], "greater or equal" `>=`
/// - [BOp::Leq], "lower or equal" `<=`
/// - [BOp::Grt], "greater" `>`
/// - [BOp::Low], "lower" `<`
#[derive(EnumIter, Debug, Clone, Copy, PartialEq)]
pub enum BOp {
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
impl BOp {
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
impl Parse for BOp {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            Ok(BOp::Mul)
        } else if input.peek(Token![/]) {
            let _: Token![/] = input.parse()?;
            Ok(BOp::Div)
        } else if input.peek(Token![+]) {
            let _: Token![+] = input.parse()?;
            Ok(BOp::Add)
        } else if input.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(BOp::Sub)
        } else if input.peek(Token![&&]) {
            let _: Token![&&] = input.parse()?;
            Ok(BOp::And)
        } else if input.peek(Token![||]) {
            let _: Token![||] = input.parse()?;
            Ok(BOp::Or)
        } else if input.peek(Token![==]) {
            let _: Token![==] = input.parse()?;
            Ok(BOp::Eq)
        } else if input.peek(Token![!=]) {
            let _: Token![!=] = input.parse()?;
            Ok(BOp::Dif)
        } else if input.peek(Token![>=]) {
            let _: Token![>=] = input.parse()?;
            Ok(BOp::Geq)
        } else if input.peek(Token![<=]) {
            let _: Token![<=] = input.parse()?;
            Ok(BOp::Leq)
        } else if input.peek(Token![>]) {
            let _: Token![>] = input.parse()?;
            Ok(BOp::Grt)
        } else if input.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            Ok(BOp::Low)
        } else {
            Err(input.error("expected binary operators"))
        }
    }
}
impl std::fmt::Display for BOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BOp::Mul => " * ".fmt(f),
            BOp::Div => " / ".fmt(f),
            BOp::Mod => " % ".fmt(f),
            BOp::Add => " + ".fmt(f),
            BOp::Sub => " - ".fmt(f),
            BOp::And => " && ".fmt(f),
            BOp::Or => " || ".fmt(f),
            BOp::Eq => " == ".fmt(f),
            BOp::Dif => " != ".fmt(f),
            BOp::Geq => " >= ".fmt(f),
            BOp::Leq => " <= ".fmt(f),
            BOp::Grt => " > ".fmt(f),
            BOp::Low => " < ".fmt(f),
        }
    }
}
impl BOp {
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
    /// # compiler_common::prelude! {}
    /// let add_type = BOp::Add.get_type();
    /// assert!(add_type.is_polymorphic());
    /// ```
    pub fn get_type(&self) -> Typ {
        match self {
            // If self is an operator over numbers then its type can either be `int -> int -> int`
            // or `float -> float -> float` then it is a [Typ::Polymorphism]
            BOp::Mul | BOp::Div | BOp::Mod | BOp::Add | BOp::Sub => {
                Typ::Polymorphism(BOp::numerical_operator)
            }
            // If self is a comparison over numbers then its type can either be `int -> int -> bool`
            // or `float -> float -> bool` then it is a [Typ::Polymorphism]
            BOp::Geq | BOp::Leq | BOp::Grt | BOp::Low => {
                Typ::Polymorphism(BOp::numerical_comparison)
            }
            // If self is an equality or inequality test then its type can be `t -> t -> bool` for
            // any t then it is a [Typ::Polymorphism]
            BOp::Eq | BOp::Dif => Typ::Polymorphism(BOp::equality),
            // If self is a logical operator then its type is `bool -> bool -> bool`
            BOp::And | BOp::Or => Typ::function(vec![Typ::bool(), Typ::bool()], Typ::bool()),
        }
    }
}

/// GRust unary operators.
///
/// [UOp] enumeration represents all possible unary operations that can be used in a GRust
/// program:
///
/// - [UOp::Neg] is the numerical negation `-`
/// - [UOp::Not], the logical negation `!`
#[derive(EnumIter, Debug, Clone, Copy, PartialEq)]
pub enum UOp {
    /// Numerical negation, `-x`.
    Neg,
    /// Logical negation, `!x`.
    Not,
}
impl UOp {
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
impl Parse for UOp {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(UOp::Neg)
        } else if input.peek(Token![!]) {
            let _: Token![!] = input.parse()?;
            Ok(UOp::Not)
        } else {
            Err(input.error("expected '-', or '!' unary operators"))
        }
    }
}
impl std::fmt::Display for UOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UOp::Neg => "-".fmt(f),
            UOp::Not => "!".fmt(f),
        }
    }
}
impl UOp {
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
    /// # compiler_common::prelude! {}
    /// let neg_type = UOp::Neg.get_type();
    /// assert!(neg_type.is_polymorphic());
    /// ```
    pub fn get_type(&self) -> Typ {
        match self {
            // If self is the numerical negation then its type can either
            // be `int -> int` or `float -> float`
            // then it is a [Typ::Polymorphism]
            UOp::Neg => Typ::Polymorphism(UOp::numerical_negation),
            // If self is the logical negation then its type is `bool -> bool`
            UOp::Not => Typ::function(vec![Typ::bool()], Typ::bool()),
        }
    }
}

/// Other builtin operators in GRust.
///
/// [OtherOp] enumeration represents all other operations that can be used in a GRust program:
///
/// - [OtherOp::IfThenElse] is `if _ then _ else _`
#[derive(EnumIter)]
pub enum OtherOp {
    /// The `if b then x else y` GRust expression.
    IfThenElse,
}
impl std::fmt::Display for OtherOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OtherOp::IfThenElse => "if_then_else".fmt(f),
        }
    }
}
impl OtherOp {
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
    /// # compiler_common::prelude! {}
    /// let ifthenelse_type = OtherOp::IfThenElse.get_type();
    /// assert!(ifthenelse_type.is_polymorphic());
    /// ```
    pub fn get_type(&self) -> Typ {
        match self {
            // If self is "if _ then _ else _" its type can be
            // `bool -> t -> t` for any type t
            // then it is a [Typ::Polymorphism]
            OtherOp::IfThenElse => Typ::Polymorphism(OtherOp::if_then_else),
        }
    }
}

#[cfg(test)]
mod to_string {
    use super::{BOp, OtherOp, UOp};

    #[test]
    fn should_convert_negation_operator_to_string() {
        assert_eq!("-", UOp::Neg.to_string());
    }
    #[test]
    fn should_convert_not_operator_to_string() {
        assert_eq!("!", UOp::Not.to_string());
    }

    #[test]
    fn should_convert_multiplication_operator_to_string() {
        assert_eq!(" * ", BOp::Mul.to_string());
    }
    #[test]
    fn should_convert_division_operator_to_string() {
        assert_eq!(" / ", BOp::Div.to_string());
    }
    #[test]
    fn should_convert_addition_operator_to_string() {
        assert_eq!(" + ", BOp::Add.to_string());
    }
    #[test]
    fn should_convert_substraction_operator_to_string() {
        assert_eq!(" - ", BOp::Sub.to_string());
    }
    #[test]
    fn should_convert_and_operator_to_string() {
        assert_eq!(" && ", BOp::And.to_string());
    }
    #[test]
    fn should_convert_or_operator_to_string() {
        assert_eq!(" || ", BOp::Or.to_string());
    }
    #[test]
    fn should_convert_equality_operator_to_string() {
        assert_eq!(" == ", BOp::Eq.to_string());
    }
    #[test]
    fn should_convert_difference_operator_to_string() {
        assert_eq!(" != ", BOp::Dif.to_string());
    }
    #[test]
    fn should_convert_greater_equal_operator_to_string() {
        assert_eq!(" >= ", BOp::Geq.to_string());
    }
    #[test]
    fn should_convert_lower_equal_operator_to_string() {
        assert_eq!(" <= ", BOp::Leq.to_string());
    }
    #[test]
    fn should_convert_greater_operator_to_string() {
        assert_eq!(" > ", BOp::Grt.to_string());
    }
    #[test]
    fn should_convert_lower_operator_to_string() {
        assert_eq!(" < ", BOp::Low.to_string());
    }

    #[test]
    fn should_convert_ifthenelse_operator_to_string() {
        assert_eq!("if_then_else", OtherOp::IfThenElse.to_string());
    }
}
