//! [StateMachine] module.

prelude! {
    ir0::contract::Term,
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
    /// A node.
    CalledNode {
        /// Identifier of the memory storage.
        memory_ident: Ident,
        /// Name of the node called.
        node_name: Ident,
        /// Component's path.
        path_opt: Option<syn::Path>,
    },
}

mk_new! { impl{T} StateElm<T> =>
    Buffer : buffer {
        ident : impl Into<Ident> = ident.into(),
        data : T,
    }
    CalledNode : called_node {
        memory_ident : impl Into<Ident> = memory_ident.into(),
        node_name : impl Into<Ident> = node_name.into(),
        path_opt: Option<syn::Path>,
    }
}

pub type StateElmInfo = StateElm<Typ>;

pub type StateElmInit = StateElm<Expr>;

impl ToTokens for StateElmInit {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            StateElmInit::Buffer { ident, data } => quote!(#ident : #data).to_tokens(tokens),
            StateElmInit::CalledNode {
                memory_ident,
                node_name,
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
                let called_state_ty = node_name.to_state_ty();
                quote!(#called_state_ty as grust::core::Component>::init()).to_tokens(tokens)
            }
        }
    }
}

/// An input element structure.
#[derive(Debug, PartialEq)]
pub struct InputElm {
    /// The name of the input.
    pub identifier: Ident,
    /// The type of the input.
    pub typ: Typ,
}

mk_new! { impl InputElm =>
    new {
        identifier : impl Into<Ident> = identifier.into(),
        typ : Typ,
    }
}

/// A node input structure.
#[derive(Debug, PartialEq)]
pub struct Input {
    /// The node's name.
    pub node_name: Ident,
    /// The input's elements.
    pub elements: Vec<InputElm>,
}

mk_new! { impl Input =>
    new {
        node_name : impl Into<Ident> = node_name.into(),
        elements : Vec<InputElm>,
    }
}

pub struct InputTokens<'a> {
    i: &'a Input,
    public: bool,
}
impl Input {
    pub fn prepare_tokens(&self, public: bool) -> InputTokens<'_> {
        InputTokens { i: self, public }
    }
}

impl<'a> ToTokens for InputTokens<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let pub_token = if self.public {
            quote! {pub}
        } else {
            quote! {}
        };
        let fields = self
            .i
            .elements
            .iter()
            .map(|InputElm { identifier, typ }| quote!(#pub_token #identifier : #typ));
        let input_ty = self.i.node_name.to_input_ty();
        quote!(
            #pub_token struct #input_ty {
                #(#fields,)*
            }
        )
        .to_tokens(tokens)
    }
}

/// A init function.
#[derive(Debug, PartialEq)]
pub struct Init {
    /// The node's name.
    pub node_name: Ident,
    /// The initialization of the node's state.
    pub state_init: Vec<StateElmInit>,
    /// The invariant initialization to prove.
    pub invariant_initialization: Vec<Term>,
}

mk_new! { impl Init =>
    new {
        node_name : impl Into<Ident> = node_name.into(),
        state_init : Vec<StateElmInit>,
        invariant_initialization : Vec<Term>,
    }
}

impl ToTokens for Init {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let state_ty = self.node_name.to_state_ty();
        let fields = self.state_init.iter();
        let id = quote_spanned!(self.node_name.span() => init);
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
    /// The node's name.
    pub node_name: Ident,
    /// The output type.
    pub output_type: Typ,
    /// The body of the step function.
    pub body: para::Stmts,
    /// The update of the node's state.
    pub state_elements_step: Vec<StateElmStep>,
    /// Logs.
    pub logs: Vec<Stmt>,
    /// The output expression.
    pub output: Expr,
    /// The contract to prove.
    pub contract: Contract,
}

mk_new! { impl Step =>
    new {
        node_name: impl Into<Ident> = node_name.into(),
        output_type: Typ,
        body: para::Stmts,
        state_elements_step: Vec<StateElmStep>,
        logs: impl Iterator<Item= Stmt> = logs.collect(),
        output: Expr,
        contract: Contract,
    }
}

pub struct StepTokens<'a> {
    step: &'a Step,
    with_contracts: bool,
}
impl Step {
    pub fn prepare_tokens(&self, with_contracts: bool) -> StepTokens {
        StepTokens {
            step: self,
            with_contracts,
        }
    }
}

impl ToTokens for StepTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.with_contracts {
            self.step.contract.prepare_tokens(false).to_tokens(tokens);
        }

        let input_ty = self.step.node_name.to_input_ty();
        let output_ty = &self.step.output_type;
        let id = quote_spanned!(self.step.node_name.span() => step);

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
            self.step.output.to_tokens(&mut tokens);

            tokens
        };

        quote! {
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

/// A node state structure.
#[derive(Debug, PartialEq)]
pub struct State {
    /// The node's name.
    pub node_name: Ident,
    /// The state's elements.
    pub elements: Vec<StateElmInfo>,
    /// The init function.
    pub init: Init,
    /// The step function.
    pub step: Step,
}

mk_new! { impl State => new {
    node_name : impl Into<Ident> = node_name.into(),
    elements : Vec<StateElmInfo>,
    init : Init,
    step : Step,
} }

pub struct StateTokens<'a> {
    state: &'a State,
    with_contracts: bool,
    align: bool,
    public: bool,
}
impl State {
    pub fn prepare_tokens(&self, with_contracts: bool, align: bool, public: bool) -> StateTokens {
        StateTokens {
            state: self,
            with_contracts,
            align,
            public,
        }
    }
}

impl StateTokens<'_> {
    fn to_struct_and_impl_tokens(&self) -> (TokenStream2, TokenStream2) {
        let fields = self.state.elements.iter().map(|element| match element {
            StateElm::Buffer { ident, data: typ } => quote!(#ident : #typ),
            StateElm::CalledNode {
                memory_ident,
                node_name,
                path_opt,
            } => {
                let name = node_name.to_state_ty();

                if let Some(mut path) = path_opt.clone() {
                    path.segments.pop();
                    path.segments.push(name.into());
                    quote!(#memory_ident : #path)
                } else {
                    quote!(#memory_ident : #name)
                }
            }
        });

        let input_ty = self.state.node_name.to_input_ty();
        let output_ty = &self.state.step.output_type;
        let state_ty = self.state.node_name.to_state_ty();
        let align_conf = if self.align {
            quote! { #[repr(align(64))]}
        } else {
            quote! {}
        };
        let pub_token = if self.public {
            quote! {pub}
        } else {
            quote! {}
        };

        let structure = quote! {
            #align_conf
            #pub_token struct #state_ty { #(#fields),* }
        };

        let init = &self.state.init;
        let step = self.state.step.prepare_tokens(self.with_contracts);
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
    /// The node's name.
    pub name: Ident,
    /// The input structure.
    pub input: Input,
    /// The state structure.
    pub state: State,
}

mk_new! { impl StateMachine => new {
    name : impl Into<Ident> = name.into(),
    input : Input,
    state : State,
} }

pub struct StateMachineTokens<'a> {
    sm: &'a StateMachine,
    with_contracts: bool,
    align: bool,
    public: bool,
}
impl StateMachine {
    pub fn prepare_tokens(
        &self,
        with_contracts: bool,
        align: bool,
        public: bool,
    ) -> StateMachineTokens {
        StateMachineTokens {
            sm: self,
            with_contracts,
            align,
            public,
        }
    }
}

impl ToTokens for StateMachineTokens<'_> {
    /// Transform [ir2] state_machine into items.
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let input_structure = &self.sm.input;
        input_structure
            .prepare_tokens(self.public)
            .to_tokens(tokens);

        let (state_structure, state_implementation) = self
            .sm
            .state
            .prepare_tokens(self.with_contracts, self.align, self.public)
            .to_struct_and_impl_tokens();
        state_structure.to_tokens(tokens);
        state_implementation.to_tokens(tokens);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_node_init() {
        let init = Init::new(
            Loc::test_id("Node"),
            vec![
                StateElmInit::buffer(
                    Loc::test_id("mem_i"),
                    Expr::lit(Constant::int(parse_quote!(0i64))),
                ),
                StateElmInit::called_node(
                    Loc::test_id("called_node_state"),
                    Loc::test_id("CalledNode"),
                    None,
                ),
            ],
            vec![],
        );

        let control = parse_quote! {
            fn init() -> NodeState {
                NodeState {
                    mem_i: 0i64,
                    called_node_state: <CalledNodeState as grust::core::Component>::init()
                }
            }
        };
        let f: syn::ItemFn = parse_quote!(#init);
        assert_eq!(f, control)
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_ext_node_init() {
        let init = Init::new(
            Loc::test_id("Node"),
            vec![
                StateElmInit::buffer(
                    Loc::test_id("mem_i"),
                    Expr::lit(Constant::int(parse_quote!(0i64))),
                ),
                StateElmInit::called_node(
                    Loc::test_id("called_node_state"),
                    Loc::test_id("CalledNode"),
                    Some(parse_quote!(path::to::called_node)),
                ),
            ],
            vec![],
        );

        let control = parse_quote! {
            fn init() -> NodeState {
                NodeState {
                    mem_i: 0i64,
                    called_node_state: <path::to::CalledNodeState as grust::core::Component>::init()
                }
            }
        };
        let f: syn::ItemFn = parse_quote!(#init);
        assert_eq!(f, control)
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_node_step() {
        let step = Step {
            contract: Default::default(),
            node_name: Loc::test_id("Node"),
            output_type: Typ::int(),
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
                    Expr::node_call(
                        Loc::test_id("called_node_state"),
                        Loc::test_id("called_node"),
                        Loc::test_id("CalledNodeInput"),
                        vec![],
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
                    Loc::test_id("called_node_state"),
                    Expr::test_ident("new_called_node_state"),
                ),
            ],
            logs: vec![],
            output: Expr::binop(BOp::Add, Expr::test_ident("o"), Expr::test_ident("y")),
        };

        let control = parse_quote! {
            fn step(&mut self, input: NodeInput) -> i64 {
                let o = self.mem_i;
                let y =  <CalledNodeState as grust::core::Component>::step(&mut self.called_node_state, CalledNodeInput {});
                self.mem_i = o + 1i64;
                self.called_node_state = new_called_node_state;
                o + y
            }
        };
        let step = step.prepare_tokens(false);
        let f: syn::ItemFn = parse_quote!(#step);
        assert_eq!(f, control)
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_ext_node_step() {
        let step = Step {
            contract: Default::default(),
            node_name: Loc::test_id("Node"),
            output_type: Typ::int(),
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
                    Expr::node_call(
                        Loc::test_id("called_node_state"),
                        Loc::test_id("called_node"),
                        Loc::test_id("CalledNodeInput"),
                        vec![],
                        Some(parse_quote!(path::to::called_node)),
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
            output: Expr::binop(BOp::Add, Expr::test_ident("o"), Expr::test_ident("y")),
        };

        let control = parse_quote! {
            fn step(&mut self, input: NodeInput) -> i64 {
                let o = self.mem_i;
                let y =  <path::to::CalledNodeState as grust::core::Component>::step(&mut self.called_node_state, path::to::CalledNodeInput {});
                self.mem_i = o + 1i64;
                o + y
            }
        };
        let step = step.prepare_tokens(false);
        let f: syn::ItemFn = parse_quote!(#step);
        assert_eq!(f, control)
    }

    #[test]
    fn should_create_rust_ast_structure_from_ir2_node_input() {
        let input = Input {
            node_name: Loc::test_id("Node"),
            elements: vec![InputElm {
                identifier: Loc::test_id("i"),
                typ: Typ::int(),
            }],
        }
        .prepare_tokens(true)
        .to_token_stream();
        let control = parse_quote!(
            pub struct NodeInput {
                pub i: i64,
            }
        );
        let inp: syn::ItemStruct = parse_quote!(#input);
        assert_eq!(inp, control)
    }
}
