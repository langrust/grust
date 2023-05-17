/// LanGrust binary operators.
///
/// [BinaryOperator] enumeration represents all possible binary operations
/// that can be used in a LanGRust program:
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

/// LanGrust unary operators.
///
/// [UnaryOperator] enumeration represents all possible unary operations
/// that can be used in a LanGRust program:
/// - [UnaryOperator::Neg] is the numerical negation `-`
/// - [UnaryOperator::Not], the logical negation `!`
/// - [UnaryOperator::Brackets], is the use of brackets `(_)`
pub enum UnaryOperator {
    /// Numerical negation, `-x`.
    Neg,
    /// Logical negation, `!x`.
    Not,
    /// Use of brackets, `(x)`.
    Brackets,
}
impl ToString for UnaryOperator {
    fn to_string(&self) -> String {
        match self {
            UnaryOperator::Neg => String::from("-"),
            UnaryOperator::Not => String::from("!"),
            UnaryOperator::Brackets => String::from("(_)"),
        }
    }
}

/// Other builtin operators in LanGrust.
///
/// [OtherOperator] enumeration represents all other operations
/// that can be used in a LanGRust program:
/// - [OtherOperator::IfThenElse] is `if _ then _ else _`
/// - [OtherOperator::Print] is the usual `print` function
pub enum OtherOperator {
    /// The `if b then x else y` LanGRust expression.
    IfThenElse,
    /// The `print(my_message)` LanGRust expression.
    Print,
}
impl ToString for OtherOperator {
    fn to_string(&self) -> String {
        match self {
            OtherOperator::IfThenElse => String::from("if_then_else"),
            OtherOperator::Print => String::from("print"),
        }
    }
}

#[cfg(test)]
mod to_string {
    use crate::ast::operator::{BinaryOperator, OtherOperator, UnaryOperator};

    #[test]
    fn should_convert_negation_operator_to_string() {
        assert_eq!(String::from("-"), UnaryOperator::Neg.to_string());
    }
    #[test]
    fn should_convert_not_operator_to_string() {
        assert_eq!(String::from("!"), UnaryOperator::Not.to_string());
    }
    #[test]
    fn should_convert_brackets_operator_to_string() {
        assert_eq!(String::from("(_)"), UnaryOperator::Brackets.to_string());
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
    #[test]
    fn should_convert_print_operator_to_string() {
        assert_eq!(String::from("print"), OtherOperator::Print.to_string());
    }
}
