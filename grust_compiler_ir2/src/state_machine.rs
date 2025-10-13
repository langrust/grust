//! [StateMachine] module.

prelude! {
    ir2::contract::Term,
}

/// A state element structure.
///
/// The type parameter is the type of the data for the `Self::Buffer` case.
#[derive(Debug, PartialEq)]
pub enum StateElm<T> {
    /// A buffer identifier and some data.
    Buffer {
        /// Identifier of the buffer.
        ident: Ident,
        /// Buffer data.
        data: T,
    },
    /// A component.
    CalledComponent {
        /// Identifier of the memory storage.
        memory_ident: Ident,
        /// Name of the component called.
        comp_name: Ident,
        /// Component's path.
        path_opt: Option<syn::Path>,
    },
}

mk_new! { impl{T} StateElm<T> =>
    Buffer : buffer {
        ident : impl Into<Ident> = ident.into(),
        data : T,
    }
    CalledComponent : called_comp {
        memory_ident : impl Into<Ident> = memory_ident.into(),
        comp_name : impl Into<Ident> = comp_name.into(),
        path_opt: Option<syn::Path>,
    }
}

pub type StateElmInfo = StateElm<Typ>;

pub type StateElmInit = StateElm<Expr>;

impl ToTokens for StateElmInit {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            StateElmInit::Buffer { ident, data } => quote!(#ident : #data).to_tokens(tokens),
            StateElmInit::CalledComponent {
                memory_ident,
                comp_name,
                path_opt,
            } => {
                memory_ident.to_tokens(tokens);
                token![:].to_tokens(tokens);
                token![<].to_tokens(tokens);
                if let Some(path) = path_opt.as_ref() {
                    let seg_len = path.segments.len();
                    if 0 < seg_len {
                        for i in 0..seg_len - 1 {
                            path.segments[i].to_tokens(tokens);
                            token![::].to_tokens(tokens)
                        }
                    }
                }
                let called_state_ty = comp_name.to_state_ty();
                quote!(#called_state_ty as grust::core::Component>::init()).to_tokens(tokens)
            }
        }
    }
}

/// A component input structure.
#[derive(Debug, PartialEq)]
pub struct Input {
    /// The component's name.
    pub comp_name: Ident,
    /// The input's elements.
    pub elements: Vec<(Ident, Typ)>,
}

mk_new! { impl Input =>
    new {
        comp_name : impl Into<Ident> = comp_name.into(),
        elements : Vec<(Ident, Typ)>,
    }
}

pub struct InputTokens<'a> {
    i: &'a Input,
    public: bool,
    tracing: bool,
}
impl Input {
    pub fn prepare_tokens(&self, public: bool, tracing: bool) -> InputTokens<'_> {
        InputTokens {
            i: self,
            public,
            tracing,
        }
    }
}

impl ToTokens for InputTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let pub_token = if self.public {
            quote! {pub}
        } else {
            TokenStream2::new()
        };
        let debug_attr = if self.tracing {
            quote! {#[derive(Debug)]}
        } else {
            TokenStream2::new()
        };
        let fields = self
            .i
            .elements
            .iter()
            .map(|(identifier, typ)| quote!(#pub_token #identifier : #typ));
        let input_ty = self.i.comp_name.to_input_ty();
        quote!(
            #debug_attr
            #pub_token struct #input_ty {
                #(#fields,)*
            }
        )
        .to_tokens(tokens)
    }
}

/// A component output structure.
#[derive(Debug, PartialEq)]
pub struct Output {
    /// The component's name.
    pub comp_name: Ident,
    /// The output's elements.
    pub elements: Vec<(Ident, Typ)>,
}

mk_new! { impl Output =>
    new {
        comp_name : impl Into<Ident> = comp_name.into(),
        elements : Vec<(Ident, Typ)>,
    }
}

pub struct OutputTokens<'a> {
    i: &'a Output,
    public: bool,
    tracing: bool,
}
impl Output {
    pub fn prepare_tokens(&self, public: bool, tracing: bool) -> OutputTokens<'_> {
        OutputTokens {
            i: self,
            public,
            tracing,
        }
    }
}

impl ToTokens for OutputTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let pub_token = if self.public {
            quote! {pub}
        } else {
            TokenStream2::new()
        };
        let debug_attr = if self.tracing {
            quote! {#[derive(Debug)]}
        } else {
            TokenStream2::new()
        };
        let fields = self
            .i
            .elements
            .iter()
            .map(|(identifier, typ)| quote!(#pub_token #identifier : #typ));
        let output_ty = self.i.comp_name.to_output_ty();
        quote!(
            #debug_attr
            #pub_token struct #output_ty {
                #(#fields,)*
            }
        )
        .to_tokens(tokens)
    }
}

/// A init function.
#[derive(Debug, PartialEq)]
pub struct Init {
    /// The component's name.
    pub comp_name: Ident,
    /// The initialization of the component's state.
    pub state_init: Vec<StateElmInit>,
    /// The invariant initialization to prove.
    pub invariant_init: Vec<Term>,
}

mk_new! { impl Init =>
    new {
        comp_name : impl Into<Ident> = comp_name.into(),
        state_init : Vec<StateElmInit>,
        invariant_init : Vec<Term>,
    }
}

pub struct InitTokens<'a> {
    init: &'a Init,
    with_contracts: bool,
}

impl Init {
    pub fn prepare_tokens(&self, with_contracts: bool) -> InitTokens {
        InitTokens {
            init: self,
            with_contracts,
        }
    }
}

impl ToTokens for InitTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.with_contracts {
            for term in self.init.invariant_init.iter() {
                let term = term.prepare_tokens(false, false);
                quote!(#[ensures(#term)]).to_tokens(tokens);
            }
        }

        let state_ty = self.init.comp_name.to_state_ty();
        let fields = self.init.state_init.iter();
        let id = quote_spanned!(self.init.comp_name.span() => init);

        quote!(
            fn #id() -> #state_ty {
                #state_ty {
                    #(#fields),*
                }
            }
        )
        .to_tokens(tokens)
    }
}

/// A step function.
#[derive(Debug, PartialEq)]
pub struct Step {
    /// The component's name.
    pub comp_name: Ident,
    /// The output type.
    /// The body of the step function.
    pub body: para::Stmts,
    /// The update of the component's state.
    pub state_elements_step: Vec<StateElmStep>,
    /// Logs.
    pub logs: Vec<Stmt>,
    /// The output expression.
    pub outputs: Vec<Ident>,
    /// The contract to prove.
    pub contract: Contract,
}

mk_new! { impl Step =>
    new {
        comp_name: impl Into<Ident> = comp_name.into(),
        body: para::Stmts,
        state_elements_step: Vec<StateElmStep>,
        logs: impl Iterator<Item= Stmt> = logs.collect(),
        outputs: impl Iterator<Item= Ident> = outputs.collect(),
        contract: Contract,
    }
}

pub struct StepTokens<'a> {
    step: &'a Step,
    with_contracts: bool,
    tracing: bool,
}
impl Step {
    pub fn prepare_tokens(&self, with_contracts: bool, tracing: bool) -> StepTokens {
        StepTokens {
            step: self,
            with_contracts,
            tracing,
        }
    }
}

impl ToTokens for StepTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.with_contracts {
            self.step.contract.prepare_tokens(false).to_tokens(tokens);
        }

        let input_ty = self.step.comp_name.to_input_ty();
        let output_ty = self.step.comp_name.to_output_ty();
        let id = quote_spanned!(self.step.comp_name.span() => step);

        let statements = {
            let mut tokens = TokenStream2::new();

            self.step.body.to_tokens(&mut tokens);
            for StateElmStep {
                identifier,
                expression,
            } in self.step.state_elements_step.iter()
            {
                quote! { self.#identifier = #expression; }.to_tokens(&mut tokens)
            }
            // add logs
            for l in self.step.logs.iter() {
                l.to_tokens(&mut tokens)
            }
            // add output expression
            let outputs = self.step.outputs.iter();
            quote! { #output_ty { #(#outputs),* } }.to_tokens(&mut tokens);

            tokens
        };

        let tracing_attr = if self.tracing {
            quote! {#[grust::tracing::instrument]}
        } else {
            TokenStream2::new()
        };

        quote! {
            #tracing_attr
            fn #id(&mut self, input: #input_ty) -> #output_ty {
                #statements
            }
        }
        .to_tokens(tokens)
    }
}

/// A state element structure for the step update.
#[derive(Debug, PartialEq)]
pub struct StateElmStep {
    /// The name of the memory storage.
    pub identifier: Ident,
    /// The expression that will update the memory.
    pub expression: Expr,
}

mk_new! { impl StateElmStep =>
    new {
        identifier: impl Into<Ident> = identifier.into(),
        expression: Expr,
    }
}

/// A component state structure.
#[derive(Debug, PartialEq)]
pub struct State {
    /// The component's name.
    pub comp_name: Ident,
    /// The state's elements.
    pub elements: Vec<StateElmInfo>,
    /// The init function.
    pub init: Init,
    /// The step function.
    pub step: Step,
}

mk_new! { impl State => new {
    comp_name : impl Into<Ident> = comp_name.into(),
    elements : Vec<StateElmInfo>,
    init : Init,
    step : Step,
} }

pub struct StateTokens<'a> {
    state: &'a State,
    with_contracts: bool,
    align: bool,
    public: bool,
    tracing: bool,
}
impl State {
    pub fn prepare_tokens(
        &self,
        with_contracts: bool,
        align: bool,
        public: bool,
        tracing: bool,
    ) -> StateTokens {
        StateTokens {
            state: self,
            with_contracts,
            align,
            public,
            tracing,
        }
    }
}

impl StateTokens<'_> {
    fn to_struct_and_impl_tokens(&self) -> (TokenStream2, TokenStream2) {
        let fields = self.state.elements.iter().map(|element| match element {
            StateElm::Buffer { ident, data: typ } => quote!(#ident : #typ),
            StateElm::CalledComponent {
                memory_ident,
                comp_name,
                path_opt,
            } => {
                let name = comp_name.to_state_ty();

                if let Some(mut path) = path_opt.clone() {
                    path.segments.pop();
                    path.segments.push(name.into());
                    quote!(#memory_ident : #path)
                } else {
                    quote!(#memory_ident : #name)
                }
            }
        });

        let input_ty = self.state.comp_name.to_input_ty();
        let output_ty = &self.state.step.comp_name.to_output_ty();
        let state_ty = self.state.comp_name.to_state_ty();
        let align_conf = if self.align {
            quote! { #[repr(align(64))]}
        } else {
            TokenStream2::new()
        };
        let pub_token = if self.public {
            quote! {pub}
        } else {
            TokenStream2::new()
        };
        let debug_attr = if self.tracing {
            quote! {#[derive(Debug)]}
        } else {
            TokenStream2::new()
        };

        let structure = quote! {
            #align_conf
            #debug_attr
            #pub_token struct #state_ty { #(#fields),* }
        };

        let init = &self.state.init.prepare_tokens(self.with_contracts);
        let step = self
            .state
            .step
            .prepare_tokens(self.with_contracts, self.tracing);
        let implementation = quote!(
            impl grust::core::Component for #state_ty {
                type Input = #input_ty;
                type Output = #output_ty;
                #init
                #step
            }
        );

        (structure, implementation)
    }
}

/// A state-machine structure.
#[derive(Debug, PartialEq)]
pub struct StateMachine {
    /// The component's name.
    pub name: Ident,
    /// The input structure.
    pub input: Input,
    /// The output structure.
    pub output: Output,
    /// The state structure.
    pub state: State,
}

mk_new! { impl StateMachine => new {
    name : impl Into<Ident> = name.into(),
    input : Input,
    output: Output,
    state : State,
} }

pub struct StateMachineTokens<'a> {
    sm: &'a StateMachine,
    with_contracts: bool,
    align: bool,
    public: bool,
    tracing: bool,
}
impl StateMachine {
    pub fn prepare_tokens(
        &self,
        with_contracts: bool,
        align: bool,
        public: bool,
        tracing: bool,
    ) -> StateMachineTokens {
        StateMachineTokens {
            sm: self,
            with_contracts,
            align,
            public,
            tracing,
        }
    }
}

impl ToTokens for StateMachineTokens<'_> {
    /// Transform [ir2] state_machine into items.
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let input_structure = &self.sm.input;
        input_structure
            .prepare_tokens(self.public, self.tracing)
            .to_tokens(tokens);

        let output_structure = &self.sm.output;
        output_structure
            .prepare_tokens(self.public, self.tracing)
            .to_tokens(tokens);

        let (state_structure, state_implementation) = self
            .sm
            .state
            .prepare_tokens(self.with_contracts, self.align, self.public, self.tracing)
            .to_struct_and_impl_tokens();
        state_structure.to_tokens(tokens);
        state_implementation.to_tokens(tokens);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_component_init() {
        let binding = Init::new(
            Loc::test_id("component"),
            vec![
                StateElmInit::buffer(
                    Loc::test_id("mem_i"),
                    Expr::lit(Constant::int(parse_quote!(0i64))),
                ),
                StateElmInit::called_comp(
                    Loc::test_id("called_component_state"),
                    Loc::test_id("CalledComponent"),
                    None,
                ),
            ],
            vec![],
        );
        let init = binding.prepare_tokens(false);

        let control = parse_quote! {
            fn init() -> ComponentState {
                ComponentState {
                    mem_i: 0i64,
                    called_component_state: <CalledComponentState as grust::core::Component>::init()
                }
            }
        };
        let f: syn::ItemFn = parse_quote!(#init);
        assert_eq!(f, control)
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_ext_component_init() {
        let binding = Init::new(
            Loc::test_id("component"),
            vec![
                StateElmInit::buffer(
                    Loc::test_id("mem_i"),
                    Expr::lit(Constant::int(parse_quote!(0i64))),
                ),
                StateElmInit::called_comp(
                    Loc::test_id("called_component_state"),
                    Loc::test_id("CalledComponent"),
                    Some(parse_quote!(path::to::called_comp)),
                ),
            ],
            vec![],
        );
        let init = binding.prepare_tokens(false);

        let control = parse_quote! {
            fn init() -> ComponentState {
                ComponentState {
                    mem_i: 0i64,
                    called_component_state: <path::to::CalledComponentState as grust::core::Component>::init()
                }
            }
        };
        let f: syn::ItemFn = parse_quote!(#init);
        assert_eq!(f, control)
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_component_step() {
        let step = Step {
            contract: Default::default(),
            comp_name: Loc::test_id("component"),
            body: para::Stmts::seq_of_pairs(vec![
                (
                    Loc::test_id("o"),
                    Expr::field_access(
                        Expr::test_ident("self"),
                        FieldIdentifier::named(Loc::test_id("mem_i")),
                    ),
                ),
                (
                    Loc::test_id("y"),
                    Expr::comp_call(
                        Loc::test_id("called_component_state"),
                        Loc::test_id("called_component"),
                        vec![],
                        std::iter::once(Loc::test_id("out")),
                        None,
                    ),
                ),
            ]),
            state_elements_step: vec![
                StateElmStep::new(
                    Loc::test_id("mem_i"),
                    Expr::binop(
                        BOp::Add,
                        Expr::test_ident("o"),
                        Expr::lit(Constant::Integer(parse_quote!(1i64))),
                    ),
                ),
                StateElmStep::new(
                    Loc::test_id("called_component_state"),
                    Expr::test_ident("new_called_component_state"),
                ),
            ],
            logs: vec![],
            outputs: vec![Loc::test_id("out")],
        };

        let control = parse_quote! {
            fn step(&mut self, input: ComponentInput) -> ComponentOutput {
                let o = self.mem_i;
                let y =  {
                    let CalledComponentOutput { out } = <CalledComponentState as grust::core::Component>::step(&mut self.called_component_state, CalledComponentInput {});
                    (out)
                };
                self.mem_i = o + 1i64;
                self.called_component_state = new_called_component_state;
                ComponentOutput { out }
            }
        };
        let step = step.prepare_tokens(false, false);
        let f: syn::ItemFn = parse_quote!(#step);
        assert_eq!(f, control)
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_ext_component_step() {
        let step = Step {
            contract: Default::default(),
            comp_name: Loc::test_id("component"),
            body: para::Stmts::seq_of_pairs(vec![
                (
                    Loc::test_id("o"),
                    Expr::field_access(
                        Expr::test_ident("self"),
                        FieldIdentifier::named(Loc::test_id("mem_i")),
                    ),
                ),
                (
                    Loc::test_id("y"),
                    Expr::comp_call(
                        Loc::test_id("called_component_state"),
                        Loc::test_id("called_component"),
                        vec![],
                        std::iter::once(Loc::test_id("out")),
                        Some(parse_quote!(path::to::called_comp)),
                    ),
                ),
            ]),
            state_elements_step: vec![StateElmStep::new(
                Loc::test_id("mem_i"),
                Expr::binop(
                    BOp::Add,
                    Expr::test_ident("o"),
                    Expr::lit(Constant::Integer(parse_quote!(1i64))),
                ),
            )],
            logs: vec![],
            outputs: vec![Loc::test_id("out")],
        };

        let control = parse_quote! {
            fn step(&mut self, input: ComponentInput) -> ComponentOutput {
                let o = self.mem_i;
                let y =  {
                    let path::to::CalledComponentOutput { out } = <path::to::CalledComponentState as grust::core::Component>::step(&mut self.called_component_state, path::to::CalledComponentInput {});
                    (out)
                };
                self.mem_i = o + 1i64;
                ComponentOutput { out }
            }
        };
        let step = step.prepare_tokens(false, false);
        let f: syn::ItemFn = parse_quote!(#step);
        assert_eq!(f, control)
    }

    #[test]
    fn should_create_rust_ast_structure_from_ir2_component_input() {
        let input = Input {
            comp_name: Loc::test_id("component"),
            elements: vec![(Loc::test_id("i"), Typ::int())],
        }
        .prepare_tokens(true, false)
        .to_token_stream();
        let control = parse_quote!(
            pub struct ComponentInput {
                pub i: i64,
            }
        );
        let inp: syn::ItemStruct = parse_quote!(#input);
        assert_eq!(inp, control)
    }
}
