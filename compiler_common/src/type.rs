use std::fmt::{self, Display};

prelude! {
    syn::{
        parse::Parse,
        punctuated::Punctuated,
        Token,
    },
}

/// GRust type system.
///
/// [Typ] enumeration is used when [typing](crate::ast::file::File) a GRust program.
///
/// It represents all possible types a GRust expression can take:
///
/// - [Typ::Integer] are [i64] integers, if `n = 1` then `n: int`
/// - [Typ::Float] are [f64] floats, if `r = 1.0` then `r: float`
/// - [Typ::Boolean] is the [bool] type for booleans, if `b = true` then `b: bool`
/// - [Typ::Unit] is the unit type, if `u = ()` then `u: unit`
/// - [Typ::Array] is the array type, if `a = [1, 2, 3]` then `a: [int; 3]`
/// - [Typ::SMEvent] is the event type for StateMachine, noted `n: int?`
/// - [Typ::SMTimeout] is the timeout type for StateMachine, noted `n: int!`
/// - [Typ::Enumeration] is an user defined enumeration, if `c = Color.Yellow` then `c:
///   Enumeration(Color)`
/// - [Typ::Structure] is an user defined structure, if `p = Point { x: 1, y: 0}` then `p:
///   Structure(Point)`
/// - [Typ::NotDefinedYet] is not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
/// - [Typ::Abstract] are functions types, if `f = |x| x+1` then `f: int -> int`
/// - [Typ::Polymorphism] is an inferable function type, if `add = |x, y| x+y` then
///   `add: 't -> 't -> 't` with `'t` in `{int, float}`
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Typ {
    /// [i64] integers, if `n = 1` then `n: int`
    Integer,
    /// [f64] floats, if `r = 1.0` then `r: float`
    Float,
    /// [bool] type for booleans, if `b = true` then `b: bool`
    Boolean,
    /// Unit type, if `u = ()` then `u: unit`
    Unit,
    /// Array type, if `a = [1, 2, 3]` then `a: [int; 3]`
    Array(Box<Typ>, usize),
    /// SMEvent type, noted `n: int?`
    SMEvent(Box<Typ>),
    /// SMTimeout type, noted `n: int!`
    SMTimeout(Box<Typ>),
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
    Abstract(Vec<Typ>, Box<Typ>),
    /// Tuple type, if `z = zip(a, b)` with `a: [int; 5]` and `b: [float; 5]` then
    /// `z: [(int, float); 5]`
    Tuple(Vec<Typ>),
    /// Generic type.
    Generic(String),
    /// Signal type, in interface if `s' = map(s, |x| x + 1)` then `s': signal int`
    Signal(Box<Typ>),
    /// Event type, in interface if `e' = map(e, |x| x + 1)` then `e': event int`
    Event(Box<Typ>),
    /// Timeout type, in interface if `e' = timeout(e, 10)` then `e': event timeout(int)`
    Timeout(Box<Typ>),
    /// Time type.
    Time,
    /// Not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
    NotDefinedYet(String),
    /// Polymorphic type, if `add = |x, y| x+y` then `add: 't : Typ -> t -> 't -> 't`
    Polymorphism(fn(Vec<Typ>, Location) -> Res<Typ>),
    /// Match any type.
    Any,
}
impl Display for Typ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Typ::Integer => write!(f, "i64"),
            Typ::Float => write!(f, "f64"),
            Typ::Boolean => write!(f, "bool"),
            Typ::Unit => write!(f, "()"),
            Typ::Array(t, n) => write!(f, "[{}; {n}]", *t),
            Typ::SMEvent(t) => write!(f, "SMEvent<{}>", *t),
            Typ::SMTimeout(t) => write!(f, "SMTimeout<{}>", *t),
            Typ::Enumeration { name, .. } => write!(f, "{name}"),
            Typ::Structure { name, .. } => write!(f, "{name}"),
            Typ::Abstract(t1, t2) => write!(
                f,
                "({}) -> {}",
                t1.into_iter()
                    .map(|input_type| input_type.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                *t2
            ),
            Typ::Tuple(ts) => write!(
                f,
                "({})",
                ts.into_iter()
                    .map(|input_type| input_type.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Typ::Generic(name) => write!(f, "{name}"),
            Typ::Signal(ty) => write!(f, "Signal<{}>", *ty),
            Typ::Event(ty) => write!(f, "Event<{}>", *ty),
            Typ::Timeout(ty) => write!(f, "Timeout<{}>", *ty),
            Typ::Time => write!(f, "Time"),
            Typ::NotDefinedYet(s) => write!(f, "{s}"),
            Typ::Polymorphism(v_t) => write!(f, "{:#?}", v_t),
            Typ::Any => write!(f, "any"),
        }
    }
}
impl Parse for Typ {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ty = if input.peek(keyword::int) {
            let _: keyword::int = input.parse()?;
            Typ::Integer
        } else if input.peek(keyword::float) {
            let _: keyword::float = input.parse()?;
            Typ::Float
        } else if input.peek(keyword::bool) {
            let _: keyword::bool = input.parse()?;
            Typ::Boolean
        } else if input.peek(syn::token::Paren) {
            let content;
            let _ = syn::parenthesized!(content in input);
            if content.is_empty() {
                Typ::Unit
            } else {
                let types: Punctuated<Typ, Token![,]> = Punctuated::parse_terminated(&content)?;
                Typ::Tuple(types.into_iter().collect())
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
                Typ::Array(Box::new(ty), size.base10_parse().unwrap())
            }
        } else if input.peek(keyword::timeout) {
            let _: keyword::timeout = input.parse()?;
            let content;
            let _ = syn::parenthesized!(content in input);
            let ty = content.parse()?;
            Typ::Timeout(Box::new(ty))
        } else {
            let ident: syn::Ident = input.parse()?;
            Typ::NotDefinedYet(ident.to_string())
        };

        loop {
            if input.peek(Token![?]) {
                let _: Token![?] = input.parse()?;
                ty = Typ::SMEvent(Box::new(ty))
            } else if input.peek(Token![!]) {
                let _: Token![!] = input.parse()?;
                ty = Typ::SMTimeout(Box::new(ty))
            } else if input.peek(Token![->]) {
                let _: Token![->] = input.parse()?;
                let out_ty = input.parse()?;
                ty = match ty {
                    Typ::Tuple(v_ty) => Typ::Abstract(v_ty, Box::new(out_ty)),
                    _ => Typ::Abstract(vec![ty], Box::new(out_ty)),
                }
            } else {
                break;
            }
        }

        Ok(ty)
    }
}

mk_new! { impl Typ =>
    Integer: int()
    Float: float()
    Boolean: bool()
    Unit: unit()
    Array: array(
        ty: Self = Box::new(ty),
        size: usize = size,
    )
    SMEvent: sm_event(ty: Self = Box::new(ty))
    SMTimeout: sm_timeout(ty: Self = Box::new(ty))
    Enumeration: enumeration {
        name: impl Into<String> = name.into(),
        id: usize,
    }
    Structure: structure {
        name: impl Into<String> = name.into(),
        id: usize,
    }
    Abstract: function(
        args: Vec<Self> = args,
        ret: Self = Box::new(ret),
    )
    Tuple: tuple(tys: Vec<Self> = tys)
    Generic: generic(name: impl Into<String> = name.into())
    Signal: signal(ty: Self = Box::new(ty))
    Event: event(ty: Self = Box::new(ty))
    Timeout: timeout(ty: Self = Box::new(ty))
    Time: time()
    NotDefinedYet: undef(name: impl Into<String> = name.into())
    Polymorphism: poly(f : fn(Vec<Self>, Location) -> Res<Self> = f)
    Any: any()
}

impl Typ {
    /// Typ application with errors handling.
    ///
    /// This function tries to apply the input type to the self type. If types are incompatible for
    /// application then an error is raised.
    ///
    /// In case of a [Typ::Polymorphism], it redefines the type according to the inputs.
    ///
    /// # Example
    /// ```rust
    /// # compiler_common::prelude! {}
    /// let mut errors = vec![];
    ///
    /// let input_types = vec![Typ::Integer];
    /// let output_type = Typ::Boolean;
    /// let mut abstraction_type =
    ///     Typ::Abstract(input_types.clone(), Box::new(output_type.clone()));
    ///
    /// let application_result = abstraction_type
    ///     .apply(input_types, Location::default(), &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(application_result, output_type);
    /// ```
    pub fn apply(
        &mut self,
        input_types: Vec<Typ>,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // if self is an abstraction, check if the input types are equal
            // and return the output type as the type of the application
            Typ::Abstract(inputs, output) => {
                if input_types.len() == inputs.len() {
                    input_types
                        .iter()
                        .zip(inputs)
                        .map(|(given_type, expected_type)| {
                            given_type.eq_check(expected_type, location.clone(), errors)
                        })
                        .collect::<TRes<_>>()?;
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
            // if self is a polymorphic type, apply the function returning the function_type with
            // the input_types, then apply the function_type with the input_type just like any other
            // type
            Typ::Polymorphism(fn_type) => {
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

    /// Check if `self` matches the expected [Typ]
    ///
    /// # Example
    ///
    /// ```rust
    /// # compiler_common::prelude! {}
    /// let mut errors = vec![];
    ///
    /// let given_type = Typ::Integer;
    /// let expected_type = Typ::Integer;
    ///
    /// given_type.eq_check(&expected_type, Location::default(), &mut errors).unwrap();
    /// assert!(errors.is_empty());
    /// ```
    pub fn eq_check(
        &self,
        expected_type: &Typ,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
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
    /// - returns a copy of abstraction type inputs;
    /// - panics if not abstraction type.
    ///
    /// # Example
    ///
    /// ```rust
    /// # compiler_common::prelude! {}
    /// let abstraction_type = Typ::Abstract(
    ///     vec![Typ::Integer, Typ::Integer],
    ///     Box::new(Typ::Integer)
    /// );
    ///
    /// assert_eq!(abstraction_type.get_inputs(), vec![Typ::Integer, Typ::Integer]);
    /// ```
    pub fn get_inputs(&self) -> Vec<Typ> {
        match self {
            Typ::Abstract(inputs, _) => inputs.clone(),
            _ => unreachable!(),
        }
    }

    /// Conversion from FRP types to StateMachine types.
    ///
    /// Converts `signal T` into `T`, `event T` into `T?` and `event timeout T` into `T!`.
    ///
    /// **NB:** this function panics on any other input.
    ///
    /// ```rust
    /// # compiler_common::prelude! {}
    /// let s_type = Typ::Signal(Box::new(Typ::Integer));
    /// let e_type = Typ::Event(Box::new(Typ::Boolean));
    /// let t_type = Typ::Event(Box::new(Typ::Timeout(Box::new(Typ::Float))));
    ///
    /// assert_eq!(s_type, Typ::signal(Typ::int()));
    /// assert_eq!(e_type, Typ::event(Typ::bool()));
    /// assert_eq!(t_type, Typ::event(Typ::timeout(Typ::float())));
    ///
    /// assert_eq!(s_type.convert(), Typ::int());
    /// assert_eq!(e_type.convert(), Typ::sm_event(Typ::bool()));
    /// assert_eq!(t_type.convert(), Typ::sm_timeout(Typ::float()));
    /// ```
    ///
    /// # Example
    ///
    /// In the example bellow, when calling the component `my_comp`, the integer signal `s` is
    /// converted into an integer `x` and the boolean event `e` is converted into an optional
    /// boolean `c`.
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
            Typ::Signal(t) => t.as_ref().clone(),
            Typ::Event(t) => match t.as_ref() {
                Typ::Timeout(t) => Typ::SMTimeout(t.clone()),
                _ => Typ::SMEvent(t.clone()),
            },
            _ => unreachable!(),
        }
    }

    pub fn is_event(&self) -> bool {
        match self {
            Typ::Event(_) | Typ::SMEvent(_) | Typ::SMTimeout(_) => true,
            _ => false,
        }
    }

    pub fn is_polymorphic(&self) -> bool {
        use Typ::*;
        let mut stack = vec![];
        let mut curr = self;

        'go_down: loop {
            match curr {
                // early return, bypass the whole stack
                Polymorphism { .. } | NotDefinedYet(_) | Generic(_) => return true,
                // leaves that don't require going down
                Integer
                | Float
                | Boolean
                | Unit
                | Enumeration { .. }
                | Structure { .. }
                | Time
                | Any => (),
                // nodes we need to go down into
                Array(ty, _)
                | SMEvent(ty)
                | SMTimeout(ty)
                | Signal(ty)
                | Event(ty)
                | Timeout(ty) => {
                    curr = ty;
                    continue 'go_down;
                }
                Abstract(tys, ty) => {
                    for ty in tys {
                        stack.push(ty);
                    }
                    curr = ty;
                    continue 'go_down;
                }
                Tuple(tys) => {
                    let mut tys = tys.iter();
                    if let Some(ty) = tys.next() {
                        curr = ty;
                        for ty in tys {
                            stack.push(ty);
                        }
                        continue 'go_down;
                    }
                    // otherwise just go up
                }
            }

            if let Some(next) = stack.pop() {
                curr = next;
                continue 'go_down;
            } else {
                debug_assert!(stack.is_empty());
                return false;
            }
        }
    }
}

#[cfg(test)]
mod apply {
    use super::*;

    fn equality(mut input_types: Vec<Typ>, location: Location) -> Res<Typ> {
        if input_types.len() == 2 {
            let type_2 = input_types.pop().unwrap();
            let type_1 = input_types.pop().unwrap();
            if type_1 == type_2 {
                Ok(Typ::Abstract(vec![type_1, type_2], Box::new(Typ::Boolean)))
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

        let input_types = vec![Typ::Integer];
        let output_type = Typ::Boolean;
        let mut abstraction_type =
            Typ::Abstract(input_types.clone(), Box::new(output_type.clone()));

        let application_result = abstraction_type
            .apply(input_types, Location::default(), &mut errors)
            .unwrap();

        assert_eq!(application_result, output_type);
    }

    #[test]
    fn should_raise_error_when_incompatible_abstraction() {
        let mut errors = vec![];

        let input_types = vec![Typ::Integer];
        let output_type = Typ::Boolean;
        let mut abstraction_type = Typ::Abstract(input_types, Box::new(output_type));

        abstraction_type
            .apply(vec![Typ::Float], Location::default(), &mut errors)
            .unwrap_err();
    }

    #[test]
    fn should_return_nonpolymorphic() {
        let mut errors = vec![];

        let mut polymorphic_type = Typ::Polymorphism(equality);

        let application_result = polymorphic_type
            .apply(
                vec![Typ::Integer, Typ::Integer],
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = Typ::Boolean;

        assert_eq!(application_result, control);
    }

    #[test]
    fn should_raise_error_when_incompatible_polymorphic_type() {
        let mut errors = vec![];

        let mut polymorphic_type = Typ::Polymorphism(equality);

        let _ = polymorphic_type
            .apply(
                vec![Typ::Integer, Typ::Float],
                Location::default(),
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_modify_polymorphic_type_to_nonpolymorphic() {
        let mut errors = vec![];

        let mut polymorphic_type = Typ::Polymorphism(equality);

        let _ = polymorphic_type
            .apply(
                vec![Typ::Integer, Typ::Integer],
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = Typ::Abstract(vec![Typ::Integer, Typ::Integer], Box::new(Typ::Boolean));

        assert_eq!(polymorphic_type, control);
    }
}

#[cfg(test)]
mod get_inputs {
    use super::*;

    #[test]
    fn should_return_inputs_from_abstraction_type() {
        let abstraction_type =
            Typ::Abstract(vec![Typ::Integer, Typ::Integer], Box::new(Typ::Integer));

        assert_eq!(
            abstraction_type.get_inputs(),
            vec![Typ::Integer, Typ::Integer]
        );
    }

    #[test]
    #[should_panic]
    fn should_panic_when_not_abstraction_type() {
        let not_abstraction_type = Typ::Integer;
        not_abstraction_type.get_inputs();
    }
}
