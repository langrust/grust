use std::fmt::{self, Display};

/// LanGrust type system.
///
/// [Type] enumeration is used when [typing](crate::typing) a LanGRust program.
///
/// It reprensents all possible types a LanGRust expression can take:
/// - [Type::Integer] are [i64] integers, if `n = 1` then `n: int`
/// - [Type::Float] are [f64] floats, if `r = 1.0` then `r: float`
/// - [Type::Boolean] is the [bool] type for booleans, if `b = true` then `b: bool`
/// - [Type::String] are strings of type [String], if `s = "hello world"` then `s: string`
/// - [Type::Unit] is the unit type, if `u = ()` then `u: unit`
/// - [Type::Array] is the array type, if `a = [1, 2, 3]` then `a: [int; 3]`
/// - [Type::Option] is the option type, if `n = some(1)` then `n: int?`
/// - [Type::Enumeration] is an user defined enumeration, if `c = Color::Yellow` then `c: Enumeration(Color)`
/// - [Type::Structure] is an user defined structure, if `p = Point { x: 1, y: 0}` then `p: Structure(Point)`
/// - [Type::NotDefinedYet] is not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
/// - [Type::Abstract] are functions types, if `f = |x| x+1` then `f: int -> int`
/// - [Type::Choice] is an inferable function type, if `add = |x, y| x+y` then `add: 't -> 't -> 't` with `t` in {`int`, `float`}
///
/// # Example
/// ```rust
/// use grustine::util::type_system::Type;
/// let number_types = vec![Type::Integer, Type::Float];
/// let addition_type = {
///     let v_t = number_types.into_iter()
///         .map(
///             |t| {
///                 Type::Abstract(
///                     Box::new(t.clone()),
///                     Box::new(Type::Abstract(
///                         Box::new(t.clone()),
///                         Box::new(t)
///                     ))
///                 )
///             }
///         ).collect();
///     Type::Choice(v_t)
/// };
/// ```
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    /// [i64] integers, if `n = 1` then `n: int`
    Integer,
    /// [f64] floats, if `r = 1.0` then `r: float`
    Float,
    /// [bool] type for booleans, if `b = true` then `b: bool`
    Boolean,
    /// Strings of type [String], if `s = "hello world"` then `s: string`
    String,
    /// Unit type, if `u = ()` then `u: unit`
    Unit,
    /// Array type, if `a = [1, 2, 3]` then `a: [int; 3]`
    Array(Box<Type>, usize),
    /// Option type, if `n = some(1)` then `n: int?`
    Option(Box<Type>),
    /// User defined enumeration, if `c = Color::Yellow` then `c: Enumeration(Color)`
    Enumeration(String),
    /// User defined structure, if `p = Point { x: 1, y: 0}` then `p: Structure(Point)`
    Structure(String),
    /// Not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
    NotDefinedYet(String),
    /// Functions types, if `f = |x| x+1` then `f: int -> int`
    Abstract(Box<Type>, Box<Type>),
    /// Inferable function type, if `add = |x, y| x+y` then `add: 't -> 't -> 't` with `t` in {`int`, `float`}
    Choice(Vec<Type>),
}
impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "i64"),
            Type::Float => write!(f, "f64"),
            Type::Boolean => write!(f, "bool"),
            Type::String => write!(f, "String"),
            Type::Unit => write!(f, "()"),
            Type::Array(t, n) => write!(f, "[{}; {n}]", *t),
            Type::Option(t) => write!(f, "Option<{}>", *t),
            Type::Enumeration(enumeration) => write!(f, "{enumeration}"),
            Type::Structure(structure) => write!(f, "{structure}"),
            Type::NotDefinedYet(s) => write!(f, "{s}"),
            Type::Abstract(t1, t2) => write!(f, "{} -> {}", *t1, *t2),
            Type::Choice(v_t) => write!(f, "{:#?}", v_t),
        }
    }
}
