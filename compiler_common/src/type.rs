use std::fmt::{self, Display};

use macro2::Span;

prelude! {
    syn::{
        parse::Parse,
        punctuated::Punctuated,
        Token,
    },
}

/// GRust type system.
///
/// [Typ] enumeration used when typing a GRust program.
///
/// It represents all possible types a GRust expression can take:
///
/// - [Typ::Integer] are [i64] integers, if `n = 1` then `n: int`
/// - [Typ::Float] are [f64] floats, if `r = 1.0` then `r: float`
/// - [Typ::Boolean] is the [bool] type for booleans, if `b = true` then `b: bool`
/// - [Typ::Unit] is the unit type, if `u = ()` then `u: unit`
/// - [Typ::Array] is the array type, if `a = [1, 2, 3]` then `a: [int; 3]`
/// - [Typ::SMEvent] is the event type for StateMachine, noted `n: int?`
/// - [Typ::Enumeration] is an user defined enumeration, if `c = Color.Yellow` then `c: Enumeration(Color)`
/// - [Typ::Structure] is an user defined structure, if `p = Point { x: 1, y: 0}` then `p: Structure(Point)`
/// - [Typ::NotDefinedYet] is not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
/// - [Typ::Abstract] are functions types, if `f = |x| x+1` then `f: int -> int`
/// - [Typ::Polymorphism] is an inferable function type, if `add = |x, y| x+y` then `add: 't -> 't -> 't` with `'t` in `{int, float}`
#[derive(Debug, Eq, Hash, Clone)]
pub enum Typ {
    /// [i64] integers, if `n = 1` then `n: int`
    Integer(keyword::int),
    /// [f64] floats, if `r = 1.0` then `r: float`
    Float(keyword::float),
    /// [bool] type for booleans, if `b = true` then `b: bool`
    Boolean(keyword::bool),
    /// Unit type, if `u = ()` then `u: unit`
    Unit(keyword::unit),
    /// Array type, if `a = [1, 2, 3]` then `a: [int; 3]`
    Array {
        bracket_token: syn::token::Bracket,
        ty: Box<Typ>,
        semi_token: Token![;],
        size: syn::LitInt,
    },
    /// SMEvent type, noted `n: int?`
    SMEvent {
        ty: Box<Typ>,
        question_token: Token![?],
    },
    /// User defined enumeration, if `c = Color.Yellow` then `c: Enumeration(Color)`
    Enumeration {
        /// Enumeration's name.
        name: syn::Ident,
        /// Enumeration's identifier.
        id: usize,
    },
    /// User defined structure, if `p = Point { x: 1, y: 0}` then `p: Structure(Point)`
    Structure {
        /// Structure's name.
        name: syn::Ident,
        /// Structure's identifier.
        id: usize,
    },
    /// Functions types, if `f = |x| x+1` then `f: int -> int`
    Abstract {
        paren_token: Option<syn::token::Paren>,
        inputs: Punctuated<Typ, Token![,]>,
        arrow_token: Token![->],
        output: Box<Typ>,
    },
    /// Tuple type, if `z = zip(a, b)` with `a: [int; 5]` and `b: [float; 5]` then
    /// `z: [(int, float); 5]`
    Tuple {
        paren_token: syn::token::Paren,
        elements: Punctuated<Typ, Token![,]>,
    },
    /// Signal type, in interface if `s' = map(s, |x| x + 1)` then `s': signal int`
    Signal {
        signal_token: keyword::signal,
        ty: Box<Typ>,
    },
    /// Event type, in interface if `e' = map(e, |x| x + 1)` then `e': event int`
    Event {
        event_token: keyword::event,
        ty: Box<Typ>,
    },
    /// Not defined yet, if `x: Color` then `x: NotDefinedYet(Color)`
    NotDefinedYet(syn::Ident),
    /// Polymorphic type, if `add = |x, y| x+y` then `add: 't : Typ -> t -> 't -> 't`
    Polymorphism(fn(Vec<Typ>, Location) -> Res<Typ>),
    /// Match any type.
    Any,
}
impl PartialEq for Typ {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(_), Self::Integer(_))
            | (Self::Float(_), Self::Float(_))
            | (Self::Boolean(_), Self::Boolean(_))
            | (Self::Unit(_), Self::Unit(_))
            | (Self::NotDefinedYet(_), Self::NotDefinedYet(_))
            | (Self::Polymorphism(_), Self::Polymorphism(_))
            | (Self::Any, Self::Any) => true,
            (
                Self::Array {
                    ty: l_ty,
                    size: l_size,
                    ..
                },
                Self::Array {
                    ty: r_ty,
                    size: r_size,
                    ..
                },
            ) => l_ty == r_ty && l_size == r_size,
            (Self::SMEvent { ty: l_ty, .. }, Self::SMEvent { ty: r_ty, .. }) => l_ty == r_ty,
            (
                Self::Enumeration {
                    name: l_name,
                    id: l_id,
                },
                Self::Enumeration {
                    name: r_name,
                    id: r_id,
                },
            ) => l_name.to_string() == r_name.to_string() && l_id == r_id,
            (
                Self::Structure {
                    name: l_name,
                    id: l_id,
                },
                Self::Structure {
                    name: r_name,
                    id: r_id,
                },
            ) => l_name.to_string() == r_name.to_string() && l_id == r_id,
            (
                Self::Abstract {
                    inputs: l_inputs,
                    output: l_output,
                    ..
                },
                Self::Abstract {
                    inputs: r_inputs,
                    output: r_output,
                    ..
                },
            ) => l_inputs.iter().zip(r_inputs).all(|(a, b)| a == b) && l_output == r_output,
            (
                Self::Tuple {
                    elements: l_elements,
                    ..
                },
                Self::Tuple {
                    elements: r_elements,
                    ..
                },
            ) => l_elements.iter().zip(r_elements).all(|(a, b)| a == b),
            (Self::Signal { ty: l_ty, .. }, Self::Signal { ty: r_ty, .. }) => l_ty == r_ty,
            (Self::Event { ty: l_ty, .. }, Self::Event { ty: r_ty, .. }) => l_ty == r_ty,
            _ => false,
        }
    }
}
impl Display for Typ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Typ::Integer(_) => write!(f, "i64"),
            Typ::Float(_) => write!(f, "f64"),
            Typ::Boolean(_) => write!(f, "bool"),
            Typ::Unit(_) => write!(f, "unit"),
            Typ::Array { ty, size, .. } => write!(f, "[{}; {size}]", *ty),
            Typ::SMEvent { ty, .. } => write!(f, "SMEvent<{}>", *ty),
            Typ::Enumeration { name, .. } => write!(f, "{name}"),
            Typ::Structure { name, .. } => write!(f, "{name}"),
            Typ::Abstract { inputs, output, .. } => write!(
                f,
                "({}) -> {}",
                inputs
                    .into_iter()
                    .map(|input_type| input_type.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                *output
            ),
            Typ::Tuple { elements, .. } => write!(
                f,
                "({})",
                elements
                    .into_iter()
                    .map(|input_type| input_type.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Typ::Signal { ty, .. } => write!(f, "Signal<{}>", *ty),
            Typ::Event { ty, .. } => write!(f, "Event<{}>", *ty),
            Typ::NotDefinedYet(s) => write!(f, "{s}"),
            Typ::Polymorphism(v_t) => write!(f, "{:#?}", v_t),
            Typ::Any => write!(f, "any"),
        }
    }
}
impl Parse for Typ {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ty = if input.peek(keyword::int) {
            let keyword: keyword::int = input.parse()?;
            Typ::Integer(keyword)
        } else if input.peek(keyword::float) {
            let keyword: keyword::float = input.parse()?;
            Typ::Float(keyword)
        } else if input.peek(keyword::bool) {
            let keyword: keyword::bool = input.parse()?;
            Typ::Boolean(keyword)
        } else if input.peek(keyword::unit) {
            let keyword: keyword::unit = input.parse()?;
            Typ::Unit(keyword)
        } else if input.peek(syn::token::Paren) {
            let content;
            let paren_token = syn::parenthesized!(content in input);
            let elements: Punctuated<Typ, Token![,]> = Punctuated::parse_terminated(&content)?;
            Typ::Tuple {
                paren_token,
                elements,
            }
        } else if input.peek(syn::token::Bracket) {
            let content;
            let bracket_token = syn::bracketed!(content in input);
            if content.is_empty() {
                return Err(input.error("expected type: `int`, `float`, etc"));
            } else {
                let ty = content.parse()?;
                let semi_token: Token![;] = content.parse()?;
                let size: syn::LitInt = content.parse()?;
                Typ::Array {
                    bracket_token,
                    ty: Box::new(ty),
                    semi_token,
                    size,
                }
            }
        } else {
            let ident: syn::Ident = input.parse()?;
            Typ::NotDefinedYet(ident)
        };

        loop {
            if input.peek(Token![?]) {
                let question_token: Token![?] = input.parse()?;
                ty = Typ::SMEvent {
                    ty: Box::new(ty),
                    question_token,
                }
            } else if input.peek(Token![->]) {
                let arrow_token: Token![->] = input.parse()?;
                let out_ty = input.parse()?;
                ty = match ty {
                    Typ::Tuple {
                        paren_token,
                        elements,
                    } => Typ::Abstract {
                        paren_token: Some(paren_token),
                        inputs: elements,
                        arrow_token,
                        output: Box::new(out_ty),
                    },
                    _ => {
                        let mut inputs = syn::punctuated::Punctuated::new();
                        inputs.push_value(ty);
                        Typ::Abstract {
                            paren_token: None,
                            inputs,
                            arrow_token,
                            output: Box::new(out_ty),
                        }
                    }
                }
            } else {
                break;
            }
        }

        Ok(ty)
    }
}

mk_new! { impl Typ =>
    Integer: int (
        keyword = Default::default()
    )
    Float: float (
        keyword = Default::default()
    )
    Boolean: bool (
        keyword = Default::default()
    )
    Unit: unit (
        keyword = Default::default()
    )
    Array: array {
        bracket_token = Default::default(),
        ty: Typ = ty.into(),
        semi_token = Default::default(),
        size: usize = syn::LitInt::new(&format!("{size}"), Span::call_site()),
    }
    Enumeration: enumeration {
        name: impl Into<String> = syn::Ident::new(&name.into(), Span::call_site()),
        id: usize,
    }
    Structure: structure {
        name: impl Into<String> = syn::Ident::new(&name.into(), Span::call_site()),
        id: usize,
    }
    Abstract: function {
        paren_token = Default::default(),
        inputs: Vec<Typ> = {
            let mut args = Punctuated::new();
            args.extend(inputs);
            args
        },
        arrow_token = Default::default(),
        output: Typ = output.into(),
    }
    Tuple: tuple {
        paren_token = Default::default(),
        elements: Vec<Typ> = {
            let mut tys = Punctuated::new();
            tys.extend(elements);
            tys
        },
    }
    SMEvent: sm_event {
        ty: Typ = ty.into(),
        question_token = Default::default(),
    }
    Signal: signal {
        signal_token = Default::default(),
        ty: Typ = ty.into(),
    }
    Event: event {
        event_token = Default::default(),
        ty: Typ = ty.into(),
    }
    NotDefinedYet: undef(
        name: impl Into<String> = syn::Ident::new(&name.into(), Span::call_site())
    )
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
    /// let input_types = vec![Typ::int()];
    /// let output_type = Typ::bool();
    /// let mut abstraction_type = Typ::function(input_types.clone(), output_type.clone());
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
            Typ::Abstract { inputs, output, .. } => {
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
    /// let given_type = Typ::int();
    /// let expected_type = Typ::int();
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
    /// let abstraction_type = Typ::function(vec![Typ::int(), Typ::int()], Typ::int());
    /// assert!(abstraction_type.get_inputs().all(|ty| ty == &Typ::int()));
    /// ```
    pub fn get_inputs<'a>(&'a self) -> impl Iterator<Item = &'a Typ> + 'a {
        match self {
            Typ::Abstract { inputs, .. } => inputs.iter(),
            _ => unreachable!(),
        }
    }

    /// Conversion from FRP types to StateMachine types.
    ///
    /// Converts `signal T` into `T` and `event T` into `T?`.
    ///
    /// **NB:** this function panics on any other input.
    ///
    /// ```rust
    /// # compiler_common::prelude! {}
    /// let s_type = Typ::signal(Typ::int());
    /// let e_type = Typ::event(Typ::bool());
    ///
    /// assert_eq!(s_type, Typ::signal(Typ::int()));
    /// assert_eq!(e_type, Typ::event(Typ::bool()));
    ///
    /// assert_eq!(s_type.convert(), Typ::int());
    /// assert_eq!(e_type.convert(), Typ::sm_event(Typ::bool()));
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
            Typ::Signal { ty, .. } => ty.as_ref().clone(),
            Typ::Event { ty, event_token } => Typ::SMEvent {
                ty: ty.clone(),
                question_token: Token![?](event_token.span),
            },
            _ => unreachable!(),
        }
    }
    /// Conversion from StateMachine types to FRP types.
    ///
    /// Converts `T` into `signal T` and `T?` into `event T`.
    ///
    /// **NB:** this function panics on `signal T` and `event T`.
    ///
    /// ```rust
    /// # compiler_common::prelude! {}
    /// let s_type = Typ::signal(Typ::int());
    /// let e_type = Typ::event(Typ::bool());
    ///
    /// assert_eq!(s_type, Typ::signal(Typ::int()));
    /// assert_eq!(e_type, Typ::event(Typ::bool()));
    ///
    /// assert_eq!(s_type.convert(), Typ::int());
    /// assert_eq!(e_type.convert(), Typ::sm_event(Typ::bool()));
    /// ```
    pub fn rev_convert(&self) -> Self {
        match self {
            Typ::Signal { .. } | Typ::Event { .. } => unreachable!(),
            Typ::SMEvent { ty, .. } => Typ::event((**ty).clone()),
            ty => Typ::signal(ty.clone()),
        }
    }

    pub fn is_event(&self) -> bool {
        match self {
            Typ::Event { .. } | Typ::SMEvent { .. } => true,
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
                Polymorphism { .. } | NotDefinedYet(_) => return true,
                // leaves that don't require going down
                Integer(_)
                | Float(_)
                | Boolean(_)
                | Unit(_)
                | Enumeration { .. }
                | Structure { .. }
                | Any => (),
                // nodes we need to go down into
                Array { ty, .. } | SMEvent { ty, .. } | Signal { ty, .. } | Event { ty, .. } => {
                    curr = ty;
                    continue 'go_down;
                }
                Abstract { inputs, output, .. } => {
                    for ty in inputs {
                        stack.push(ty);
                    }
                    curr = output;
                    continue 'go_down;
                }
                Tuple { elements, .. } => {
                    let mut tys = elements.iter();
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

        let input_types = vec![Typ::int()];
        let output_type = Typ::bool();
        let mut abstraction_type = Typ::function(input_types.clone(), output_type.clone());

        let application_result = abstraction_type
            .apply(input_types, Location::default(), &mut errors)
            .unwrap();

        assert_eq!(application_result, output_type);
    }

    #[test]
    fn should_raise_error_when_incompatible_abstraction() {
        let mut errors = vec![];

        let input_types = vec![Typ::int()];
        let output_type = Typ::bool();
        let mut abstraction_type = Typ::function(input_types, output_type);

        abstraction_type
            .apply(vec![Typ::float()], Location::default(), &mut errors)
            .unwrap_err();
    }

    #[test]
    fn should_return_nonpolymorphic() {
        let mut errors = vec![];

        let mut polymorphic_type = Typ::poly(equality);

        let application_result = polymorphic_type
            .apply(
                vec![Typ::int(), Typ::int()],
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = Typ::bool();

        assert_eq!(application_result, control);
    }

    #[test]
    fn should_raise_error_when_incompatible_polymorphic_type() {
        let mut errors = vec![];

        let mut polymorphic_type = Typ::poly(equality);

        let _ = polymorphic_type
            .apply(
                vec![Typ::int(), Typ::float()],
                Location::default(),
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_modify_polymorphic_type_to_nonpolymorphic() {
        let mut errors = vec![];

        let mut polymorphic_type = Typ::poly(equality);

        let _ = polymorphic_type
            .apply(
                vec![Typ::int(), Typ::int()],
                Location::default(),
                &mut errors,
            )
            .unwrap();

        let control = Typ::function(vec![Typ::int(), Typ::int()], Typ::bool());

        assert_eq!(polymorphic_type, control);
    }
}

#[cfg(test)]
mod get_inputs {
    use super::*;

    #[test]
    fn should_return_inputs_from_abstraction_type() {
        let abstraction_type = Typ::function(vec![Typ::int(), Typ::int()], Typ::int());

        assert_eq!(
            abstraction_type.get_inputs().cloned().collect::<Vec<_>>(),
            vec![Typ::int(), Typ::int()]
        );
    }

    #[test]
    #[should_panic]
    fn should_panic_when_not_abstraction_type() {
        let not_abstraction_type = Typ::int();
        let _ = not_abstraction_type.get_inputs();
    }
}
