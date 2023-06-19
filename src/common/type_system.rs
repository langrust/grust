use std::collections::HashMap;
use std::fmt::{self, Display};

use crate::common::context::Context;
use crate::common::{location::Location, user_defined_type::UserDefinedType};
use crate::error::Error;

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
/// - [Type::Enumeration] is an user defined enumeration, if `c = Color.Yellow` then `c: Enumeration(Color)`
/// - [Type::Structure] is an user defined structure, if `p = Point { x: 1, y: 0}` then `p: Structure(Point)`
/// - [Type::NotDefinedYet] is not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
/// - [Type::Abstract] are functions types, if `f = |x| x+1` then `f: int -> int`
/// - [Type::Choice] is an inferable function type, if `add = |x, y| x+y` then `add: 't -> 't -> 't` with `t` in {`int`, `float`}
///
/// # Example
/// ```rust
/// use grustine::common::type_system::Type;
///
/// let number_types = vec![Type::Integer, Type::Float];
/// let addition_type = {
///     let v_t = number_types.into_iter()
///         .map(|t| Type::Abstract(vec![t.clone(), t.clone()], Box::new(t)))
///         .collect();
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
    /// User defined enumeration, if `c = Color.Yellow` then `c: Enumeration(Color)`
    Enumeration(String),
    /// User defined structure, if `p = Point { x: 1, y: 0}` then `p: Structure(Point)`
    Structure(String),
    /// Not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
    NotDefinedYet(String),
    /// Functions types, if `f = |x| x+1` then `f: int -> int`
    Abstract(Vec<Type>, Box<Type>),
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
            Type::Abstract(t1, t2) => write!(f, "{:#?} -> {}", t1, *t2),
            Type::Choice(v_t) => write!(f, "{:#?}", v_t),
        }
    }
}

impl Type {
    /// Type application with errors handling.
    ///
    /// This function tries to apply the input type to the self type.
    /// If types are incompatible for application then an error is raised.
    ///
    /// # Example
    /// ```rust
    /// use grustine::common::{location::Location, type_system::Type};
    ///
    /// let mut errors = vec![];
    ///
    /// let input_types = vec![Type::Integer];
    /// let output_type = Type::Boolean;
    /// let abstraction_type =
    ///     Type::Abstract(input_types.clone(), Box::new(output_type.clone()));
    ///
    /// let application_result = abstraction_type
    ///     .apply(input_types, Location::default(), &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(application_result, output_type);
    /// ```
    pub fn apply(
        self,
        input_types: Vec<Type>,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<Type, ()> {
        match self {
            // if self is an abstraction, check if the input types are equal
            // and return the output type as the type of the application
            Type::Abstract(inputs, output) => {
                if input_types.len() == inputs.len() {
                    input_types
                        .iter()
                        .zip(inputs)
                        .map(|(given_type, expected_type)| {
                            given_type.eq_check(&expected_type, location.clone(), errors)
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .collect::<Result<_, _>>()?;
                    Ok(*output)
                } else {
                    let error = Error::IncompatibleInputsNumber {
                        given_inputs_number: input_types.len(),
                        expected_inputs_number: inputs.len(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(());
                }
            }
            // if self is a choice type, it means that their are several options
            // then perform application for each option and keep succeeding ones
            Type::Choice(types) => {
                let given_type = Type::Choice(types.clone());

                let mut new_types = types
                    .into_iter()
                    .filter_map(|typing| {
                        let mut temp_errors = vec![];
                        typing
                            .apply(input_types.clone(), location.clone(), &mut temp_errors)
                            .ok()
                    })
                    .collect::<Vec<Type>>();

                if new_types.is_empty() {
                    let error = Error::ExpectAbstraction {
                        input_types,
                        given_type,
                        location,
                    };
                    errors.push(error);
                    Err(())
                } else if new_types.len() == 1 {
                    Ok(new_types.pop().unwrap())
                } else {
                    Ok(Type::Choice(new_types))
                }
            }
            _ => {
                let error = Error::ExpectAbstraction {
                    input_types,
                    given_type: self,
                    location,
                };
                errors.push(error);
                Err(())
            }
        }
    }

    /// Check if `self` matches the expected [Type]
    ///
    /// # Example
    /// ```rust
    /// use grustine::common::{location::Location, type_system::Type};
    /// use grustine::error::Error;
    ///
    /// let mut errors = vec![];
    ///
    /// let given_type = Type::Integer;
    /// let expected_type = Type::Integer;
    ///
    /// given_type.eq_check(&expected_type, Location::default(), &mut errors).unwrap();
    /// assert!(errors.is_empty());
    /// ```
    pub fn eq_check(
        &self,
        expected_type: &Type,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        if self.eq(expected_type) {
            Ok(())
        } else {
            let error = Error::IncompatibleType {
                given_type: self.clone(),
                expected_type: expected_type.clone(),
                location: location,
            };
            errors.push(error);
            Err(())
        }
    }

    /// Determine the type if undefined
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::common::{
    ///     location::Location, type_system::Type, user_defined_type::UserDefinedType,
    /// };
    ///
    /// let mut errors = vec![];
    /// let mut user_types_context = HashMap::new();
    /// user_types_context.insert(
    ///     String::from("Point"),
    ///     UserDefinedType::Structure {
    ///         id: String::from("Point"),
    ///         fields: vec![
    ///             (String::from("x"), Type::Integer),
    ///             (String::from("y"), Type::Integer),
    ///         ],
    ///         location: Location::default(),
    ///     }
    /// );
    ///
    /// let mut my_type = Type::NotDefinedYet(String::from("Point"));
    ///
    /// let control = Type::Structure(String::from("Point"));
    ///
    /// my_type
    ///     .resolve_undefined(Location::default(), &user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(my_type, control);
    /// ```
    pub fn resolve_undefined(
        &mut self,
        location: Location,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            Type::NotDefinedYet(name) => {
                let user_type =
                    user_types_context.get_user_type_or_error(name, location, errors)?;
                *self = user_type.into_type();
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod apply {
    use crate::common::{location::Location, type_system::Type};

    #[test]
    fn should_apply_input_to_abstraction_when_compatible() {
        let mut errors = vec![];

        let input_types = vec![Type::Integer];
        let output_type = Type::Boolean;
        let abstraction_type = Type::Abstract(input_types.clone(), Box::new(output_type.clone()));

        let application_result = abstraction_type
            .apply(input_types, Location::default(), &mut errors)
            .unwrap();

        assert_eq!(application_result, output_type);
    }

    #[test]
    fn should_raise_error_when_incompatible_abstraction() {
        let mut errors = vec![];

        let input_types = vec![Type::Integer];
        let output_type = Type::Boolean;
        let abstraction_type = Type::Abstract(input_types, Box::new(output_type));

        abstraction_type
            .apply(vec![Type::Float], Location::default(), &mut errors)
            .unwrap_err();
    }

    #[test]
    fn should_apply_input_to_choice_type_when_compatible() {
        let mut errors = vec![];

        let choice_type = Type::Choice(vec![
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
            Type::Abstract(vec![Type::Float], Box::new(Type::Float)),
        ]);

        let application_result = choice_type
            .apply(vec![Type::Integer], Location::default(), &mut errors)
            .unwrap();

        let control = Type::Choice(vec![Type::Integer, Type::Float]);

        assert_eq!(application_result, control);
    }

    #[test]
    fn should_return_nonchoice_when_only_one_choice_left() {
        let mut errors = vec![];

        let choice_type = Type::Choice(vec![
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
            Type::Abstract(vec![Type::Float], Box::new(Type::Float)),
        ]);

        let application_result = choice_type
            .apply(vec![Type::Integer], Location::default(), &mut errors)
            .unwrap();

        let control = Type::Integer;

        assert_eq!(application_result, control);
    }

    #[test]
    fn should_raise_error_when_incompatible_choice_type() {
        let mut errors = vec![];

        let choice_type = Type::Choice(vec![
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
            Type::Abstract(vec![Type::Float], Box::new(Type::Float)),
        ]);

        choice_type
            .apply(vec![Type::Boolean], Location::default(), &mut errors)
            .unwrap_err();
    }
}

#[cfg(test)]
mod resolve_undefined {
    use std::collections::HashMap;

    use crate::common::{
        location::Location, type_system::Type, user_defined_type::UserDefinedType,
    };

    #[test]
    fn should_determine_undefined_type_when_in_context() {
        let mut errors = vec![];
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            UserDefinedType::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut my_type = Type::NotDefinedYet(String::from("Point"));

        let control = Type::Structure(String::from("Point"));

        my_type
            .resolve_undefined(Location::default(), &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(my_type, control);
    }

    #[test]
    fn should_leave_already_determined_types_unchanged() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut my_type = Type::Integer;

        let control = Type::Integer;

        my_type
            .resolve_undefined(Location::default(), &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(my_type, control);
    }

    #[test]
    fn should_raise_error_for_undefined_type_when_not_in_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut my_type = Type::NotDefinedYet(String::from("Point"));

        my_type
            .resolve_undefined(Location::default(), &user_types_context, &mut errors)
            .unwrap_err();
    }
}
