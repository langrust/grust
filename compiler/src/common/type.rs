use std::fmt::{self, Display};

use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::ast::keyword;
use crate::common::location::Location;
use crate::error::{Error, TerminationError};

/// GRust type system.
///
/// [Type] enumeration is used when [typing](crate::ast::file::File) a GRust program.
///
/// It reprensents all possible types a GRust expression can take:
/// - [Type::Integer] are [i64] integers, if `n = 1` then `n: int`
/// - [Type::Float] are [f64] floats, if `r = 1.0` then `r: float`
/// - [Type::Boolean] is the [bool] type for booleans, if `b = true` then `b: bool`
/// - [Type::Unit] is the unit type, if `u = ()` then `u: unit`
/// - [Type::Array] is the array type, if `a = [1, 2, 3]` then `a: [int; 3]`
/// - [Type::SMEvent] is the event type for StateMachine, noted `n: int?`
/// - [Type::SMTimeout] is the timeout type for StateMachine, noted `n: int!`
/// - [Type::Enumeration] is an user defined enumeration, if `c = Color.Yellow` then `c: Enumeration(Color)`
/// - [Type::Structure] is an user defined structure, if `p = Point { x: 1, y: 0}` then `p: Structure(Point)`
/// - [Type::NotDefinedYet] is not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
/// - [Type::Abstract] are functions types, if `f = |x| x+1` then `f: int -> int`
/// - [Type::Polymorphism] is an inferable function type, if `add = |x, y| x+y` then `add: 't -> 't -> 't` with `t` in {`int`, `float`}
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Type {
    /// [i64] integers, if `n = 1` then `n: int`
    Integer,
    /// [f64] floats, if `r = 1.0` then `r: float`
    Float,
    /// [bool] type for booleans, if `b = true` then `b: bool`
    Boolean,
    /// Unit type, if `u = ()` then `u: unit`
    Unit,
    /// Array type, if `a = [1, 2, 3]` then `a: [int; 3]`
    Array(Box<Type>, usize),
    /// SMEvent type, noted `n: int?`
    SMEvent(Box<Type>),
    /// SMTimeout type, noted `n: int!`
    SMTimeout(Box<Type>),
    /// Component event
    ComponentEvent,
    /// User defined enumeration, if `c = Color.Yellow` then `c: Enumeration(Color)`
    Enumeration {
        /// Enumeration's name.
        name: String,
        /// Enumeration's identifier.
        id: usize,
    },
    /// User defined structure, if `p = Point { x: 1, y: 0}` then `p: Structure(Point)`
    Structure {
        /// Structure's name.
        name: String,
        /// Structure's identifier.
        id: usize,
    },
    /// Functions types, if `f = |x| x+1` then `f: int -> int`
    Abstract(Vec<Type>, Box<Type>),
    /// Tuple type, if `z = zip(a, b)` with `a: [int; 5]` and `b: [float; 5]` then `z: [(int, float); 5]`
    Tuple(Vec<Type>),
    /// Generic type.
    Generic(String),
    /// Signal type, in interface if `s' = map(s, |x| x + 1)` then `s': signal int`
    Signal(Box<Type>),
    /// Event type, in interface if `e' = map(e, |x| x + 1)` then `e': event int`
    Event(Box<Type>),
    /// Timeout type, in interface if `e' = timeout(e, 10)` then `e': event timeout(int)`
    Timeout(Box<Type>),
    /// Time type.
    Time,
    /// Not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
    NotDefinedYet(String),
    /// Polymorphic type, if `add = |x, y| x+y` then `add: 't : Type -> t -> 't -> 't`
    Polymorphism(fn(Vec<Type>, Location) -> Result<Type, Error>),
    /// Match any type.
    Any,
}
impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "i64"),
            Type::Float => write!(f, "f64"),
            Type::Boolean => write!(f, "bool"),
            Type::Unit => write!(f, "()"),
            Type::Array(t, n) => write!(f, "[{}; {n}]", *t),
            Type::SMEvent(t) => write!(f, "SMEvent<{}>", *t),
            Type::SMTimeout(t) => write!(f, "SMTimeout<{}>", *t),
            Type::ComponentEvent => write!(f, "ComponentEvent"),
            Type::Enumeration { name, .. } => write!(f, "{name}"),
            Type::Structure { name, .. } => write!(f, "{name}"),
            Type::Abstract(t1, t2) => write!(
                f,
                "({}) -> {}",
                t1.into_iter()
                    .map(|input_type| input_type.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                *t2
            ),
            Type::Tuple(ts) => write!(
                f,
                "({})",
                ts.into_iter()
                    .map(|input_type| input_type.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Type::Generic(name) => write!(f, "{name}"),
            Type::Signal(ty) => write!(f, "Signal<{}>", *ty),
            Type::Event(ty) => write!(f, "Event<{}>", *ty),
            Type::Timeout(ty) => write!(f, "Timeout<{}>", *ty),
            Type::Time => write!(f, "Time"),
            Type::NotDefinedYet(s) => write!(f, "{s}"),
            Type::Polymorphism(v_t) => write!(f, "{:#?}", v_t),
            Type::Any => write!(f, "any"),
        }
    }
}
impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ty = if input.peek(keyword::int) {
            let _: keyword::int = input.parse()?;
            Type::Integer
        } else if input.peek(keyword::float) {
            let _: keyword::float = input.parse()?;
            Type::Float
        } else if input.peek(keyword::bool) {
            let _: keyword::bool = input.parse()?;
            Type::Boolean
        } else if input.peek(syn::token::Paren) {
            let content;
            let _ = syn::parenthesized!(content in input);
            if content.is_empty() {
                Type::Unit
            } else {
                let types: Punctuated<Type, Token![,]> = Punctuated::parse_terminated(&content)?;
                Type::Tuple(types.into_iter().collect())
            }
        } else if input.peek(syn::token::Bracket) {
            let content;
            let _ = syn::bracketed!(content in input);
            if content.is_empty() {
                return Err(input.error("expected type: `int`, `float`, etc"));
            } else {
                let ty = content.parse()?;
                let _: Token![;] = content.parse()?;
                let size: syn::LitInt = content.parse()?;
                Type::Array(Box::new(ty), size.base10_parse().unwrap())
            }
        } else if input.peek(keyword::timeout) {
            let _: keyword::timeout = input.parse()?;
            let content;
            let _ = syn::parenthesized!(content in input);
            let ty = content.parse()?;
            Type::Timeout(Box::new(ty))
        } else {
            let ident: syn::Ident = input.parse()?;
            Type::NotDefinedYet(ident.to_string())
        };

        loop {
            if input.peek(Token![?]) {
                let _: Token![?] = input.parse()?;
                ty = Type::SMEvent(Box::new(ty))
            } else if input.peek(Token![!]) {
                let _: Token![!] = input.parse()?;
                ty = Type::SMTimeout(Box::new(ty))
            } else if input.peek(Token![->]) {
                let _: Token![->] = input.parse()?;
                let out_ty = input.parse()?;
                ty = match ty {
                    Type::Tuple(v_ty) => Type::Abstract(v_ty, Box::new(out_ty)),
                    _ => Type::Abstract(vec![ty], Box::new(out_ty)),
                }
            } else {
                break;
            }
        }

        Ok(ty)
    }
}

impl Type {
    /// Type application with errors handling.
    ///
    /// This function tries to apply the input type to the self type.
    /// If types are incompatible for application then an error is raised.
    ///
    /// In case of a [Type::Polymorphism], it redefines the type according to the inputs.
    ///
    /// # Example
    /// ```rust
    /// use grustine::common::{location::Location, r#type::Type};
    ///
    /// let mut errors = vec![];
    ///
    /// let input_types = vec![Type::Integer];
    /// let output_type = Type::Boolean;
    /// let mut abstraction_type =
    ///     Type::Abstract(input_types.clone(), Box::new(output_type.clone()));
    ///
    /// let application_result = abstraction_type
    ///     .apply(input_types, Location::default(), &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(application_result, output_type);
    /// ```
    pub fn apply(
        &mut self,
        input_types: Vec<Type>,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // if self is an abstraction, check if the input types are equal
            // and return the output type as the type of the application
            Type::Abstract(inputs, output) => {
                if input_types.len() == inputs.len() {
                    input_types
                        .iter()
                        .zip(inputs)
                        .map(|(given_type, expected_type)| {
                            given_type.eq_check(expected_type, location.clone(), errors)
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .collect::<Result<_, _>>()?;
                    Ok((**output).clone())
                } else {
                    let error = Error::IncompatibleInputsNumber {
                        given_inputs_number: input_types.len(),
                        expected_inputs_number: inputs.len(),
                        location,
                    };
                    errors.push(error);
                    Err(TerminationError)
                }
            }
            // if self is a polymorphic type, apply the function returning the function_type
            // with the input_types, then apply the function_type with the input_type
            // just like any other type
            Type::Polymorphism(fn_type) => {
                let mut function_type =
                    fn_type(input_types.clone(), location.clone()).map_err(|error| {
                        errors.push(error);
                        TerminationError
                    })?;
                let result = function_type.apply(input_types.clone(), location, errors)?;

                *self = function_type;
                Ok(result)
            }
            _ => {
                let error = Error::ExpectAbstraction {
                    input_types,
                    given_type: self.clone(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Check if `self` matches the expected [Type]
    ///
    /// # Example
    /// ```rust
    /// use grustine::common::{location::Location, r#type::Type};
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
    ) -> Result<(), TerminationError> {
        if self.eq(expected_type) {
            Ok(())
        } else {
            let error = Error::IncompatibleType {
                given_type: self.clone(),
                expected_type: expected_type.clone(),
                location,
            };
            errors.push(error);
            Err(TerminationError)
        }
    }

    /// Get inputs from abstraction type.
    ///
    /// Return a copy of abstraction type inputs.
    /// Panic if not abstraction type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grustine::common::r#type::Type;
    ///
    /// let abstraction_type = Type::Abstract(
    ///     vec![Type::Integer, Type::Integer],
    ///     Box::new(Type::Integer)
    /// );
    ///
    /// assert_eq!(abstraction_type.get_inputs(), vec![Type::Integer, Type::Integer]);
    /// ```
    pub fn get_inputs(&self) -> Vec<Type> {
        match self {
            Type::Abstract(inputs, _) => inputs.clone(),
            _ => unreachable!(),
        }
    }

    /// Convert from FRP types into StateMachine types.
    ///
    /// Convertes `signal T` into `T`, `event T` into `T?` and `timeout T` into `T!`.
    ///
    /// ```rust
    /// use grustine::common::r#type::Type;
    ///
    /// let s_type = Type::Signal(Box::new(Type::Integer));
    /// let e_type = Type::Event(Box::new(Type::Boolean));
    /// let t_type = Type::Timeout(Box::new(Type::Float));
    ///
    /// assert_eq!(s_type.convert(), Type::Integer);
    /// assert_eq!(e_type.convert(), Type::SMEvent(Box::new(Type::Boolean)));
    /// assert_eq!(t_type.convert(), Type::SMTimeout(Box::new(Type::Float)));
    /// ```
    ///
    /// # Example
    ///
    /// In the example bellow, when calling the component `my_comp`,
    /// the integer signal `s` is converted into an integer `x` and
    /// the boolean event `e` is converted into an optional boolean `c`.
    ///
    /// ```gr
    /// component my_comp(int x, bool? c) {
    ///     out res: int = when c then x else prev_res;
    ///     prev_res: int = 0 fby res;
    /// }
    ///
    ///
    /// interface exemple {
    ///     import signal int  s;
    ///     import event  bool e;
    ///
    ///     signal int res = my_comp(s, e);
    ///
    ///     export res;
    /// }
    /// ```
    pub fn convert(&self) -> Self {
        match self {
            Type::Signal(t) => t.as_ref().clone(),
            Type::Event(t) => match t.as_ref() {
                Type::Timeout(t) => Type::SMTimeout(t.clone()),
                _ => Type::SMEvent(t.clone()),
            },
            _ => unreachable!(),
        }
    }

    pub fn is_event(&self) -> bool {
        match self {
            Type::Event(_) | Type::SMEvent(_) | Type::SMTimeout(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod apply {
    use crate::{
        common::{location::Location, r#type::Type},
        error::Error,
    };

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

    #[test]
    fn should_apply_input_to_abstraction_when_compatible() {
        let mut errors = vec![];

        let input_types = vec![Type::Integer];
        let output_type = Type::Boolean;
        let mut abstraction_type =
            Type::Abstract(input_types.clone(), Box::new(output_type.clone()));

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
        let mut abstraction_type = Type::Abstract(input_types, Box::new(output_type));

        abstraction_type
            .apply(vec![Type::Float], Location::default(), &mut errors)
            .unwrap_err();
    }

    #[test]
    fn should_return_nonpolymorphic() {
        let mut errors = vec![];

        let mut polymorphic_type = Type::Polymorphism(equality);

        let application_result = polymorphic_type
            .apply(
                vec![Type::Integer, Type::Integer],
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = Type::Boolean;

        assert_eq!(application_result, control);
    }

    #[test]
    fn should_raise_error_when_incompatible_polymorphic_type() {
        let mut errors = vec![];

        let mut polymorphic_type = Type::Polymorphism(equality);

        let _ = polymorphic_type
            .apply(
                vec![Type::Integer, Type::Float],
                Location::default(),
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_modify_polymorphic_type_to_nonpolymorphic() {
        let mut errors = vec![];

        let mut polymorphic_type = Type::Polymorphism(equality);

        let _ = polymorphic_type
            .apply(
                vec![Type::Integer, Type::Integer],
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Boolean));

        assert_eq!(polymorphic_type, control);
    }
}

#[cfg(test)]
mod get_inputs {
    use crate::common::r#type::Type;

    #[test]
    fn should_return_inputs_from_abstraction_type() {
        let abstraction_type =
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer));

        assert_eq!(
            abstraction_type.get_inputs(),
            vec![Type::Integer, Type::Integer]
        );
    }

    #[test]
    #[should_panic]
    fn should_panic_when_not_abstraction_type() {
        let not_abstraction_type = Type::Integer;
        not_abstraction_type.get_inputs();
    }
}
