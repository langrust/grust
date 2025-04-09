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

impl Input {
    pub fn into_syn(self) -> syn::ItemStruct {
        let mut fields: Vec<syn::Field> = Vec::new();
        for InputElm { identifier, typ } in self.elements {
            let typ = typ.into_syn();
            let identifier = format_ident!("{identifier}");
            fields.push(parse_quote! { pub #identifier : #typ });
        }

        let name = self.node_name.to_input_ty();
        parse_quote! {
            pub struct #name {
                #(#fields,)*
            }
        }
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

impl Init {
    pub fn into_syn(self, crates: &mut BTreeSet<String>) -> syn::ImplItemFn {
        let state_ty = Ident::new(
            &to_camel_case(&format!("{}State", self.node_name)),
            Span::call_site(),
        );
        let signature = syn::Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: Ident::new("init", Span::call_site()),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Default::default(),
            variadic: None,
            output: syn::ReturnType::Type(Default::default(), parse_quote! { #state_ty }),
        };

        let fields = self
            .state_init
            .into_iter()
            .map(|element| -> syn::FieldValue {
                match element {
                    StateElmInit::Buffer { ident, data } => {
                        let id = format_ident!("{}", ident);
                        let expr: syn::Expr = data.into_syn(crates);
                        parse_quote! { #id : #expr }
                    }
                    StateElmInit::CalledNode {
                        memory_ident,
                        node_name,
                        path_opt,
                    } => {
                        let id = memory_ident;
                        let called_state_ty = node_name.to_state_ty();

                        if let Some(mut path) = path_opt {
                            path.segments.pop();
                            path.segments.push(called_state_ty.into());
                            parse_quote! { #id : <#path as grust::core::Component>::init() }
                        } else {
                            parse_quote! { #id : <#called_state_ty as grust::core::Component>::init() }
                        }
                    }
                }
            })
            .collect();

        let body = syn::Block {
            brace_token: Default::default(),
            stmts: vec![syn::Stmt::Expr(
                syn::Expr::Struct(syn::ExprStruct {
                    attrs: vec![],
                    path: parse_quote! { #state_ty },
                    brace_token: Default::default(),
                    dot2_token: None,
                    rest: None,
                    fields,
                    qself: None, // Add the qself field here
                }),
                None,
            )],
        };
        syn::ImplItemFn {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: signature,
            block: body,
        }
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

impl Step {
    /// Transform [ir2] step into RustAST implementation method.
    pub fn into_syn(self, ctx: &ir0::Ctx, crates: &mut BTreeSet<String>) -> syn::ImplItemFn {
        let attributes = if ctx.conf.greusot {
            self.contract.into_syn(false)
        } else {
            vec![]
        };

        let input_ty = self.node_name.to_input_ty();
        let ty = parse_quote! { #input_ty };

        let inputs = vec![
            parse_quote!(&mut self),
            syn::FnArg::Typed(syn::PatType {
                attrs: vec![],
                pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident: Ident::new("input", Span::call_site()),
                    subpat: None,
                })),
                colon_token: Default::default(),
                ty,
            }),
        ]
        .into_iter()
        .collect();

        let signature = syn::Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: Ident::new("step", Span::call_site()),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs,
            variadic: None,
            output: syn::ReturnType::Type(
                Default::default(),
                Box::new(self.output_type.into_syn()),
            ),
        };

        let statements = {
            let mut vec = Vec::with_capacity(self.state_elements_step.len() + 1);

            self.body.extend_syn(&mut vec, crates);
            for StateElmStep {
                identifier,
                expression,
            } in self.state_elements_step
            {
                let id = format_ident!("{}", identifier);
                let expr = expression.into_syn(crates);
                vec.push(parse_quote! { self.#id = #expr; })
            }
            // add logs
            vec.extend(self.logs.into_iter().map(|l| l.into_syn(crates)));
            // add output expression
            vec.push(syn::Stmt::Expr(self.output.into_syn(crates), None));

            vec
        };

        let body = syn::Block {
            stmts: statements,
            brace_token: Default::default(),
        };

        syn::ImplItemFn {
            attrs: attributes,
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: signature,
            block: body,
        }
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

impl State {
    /// Transform [ir2] state into RustAST structure and implementation.
    pub fn into_syn(
        self,
        ctx: &ir0::Ctx,
        crates: &mut BTreeSet<String>,
    ) -> (syn::ItemStruct, syn::ItemImpl) {
        let fields: Vec<syn::Field> = self
            .elements
            .into_iter()
            .map(|element| match element {
                StateElm::Buffer { ident, data: typ } => {
                    let ident = format_ident!("{ident}");
                    let ty = typ.into_syn();
                    parse_quote! { #ident : #ty }
                }
                StateElm::CalledNode {
                    memory_ident,
                    node_name,
                    path_opt,
                } => {
                    let name = node_name.to_state_ty();

                    if let Some(mut path) = path_opt {
                        path.segments.pop();
                        path.segments.push(name.into());
                        parse_quote! { #memory_ident : #path}
                    } else {
                        parse_quote! { #memory_ident : #name }
                    }
                }
            })
            .collect();

        let input_ty = self.node_name.to_input_ty();
        let output_ty = self.step.output_type.into_syn();
        let state_ty = self.node_name.to_state_ty();
        let structure = parse_quote!(
            pub struct #state_ty { #(#fields),* }
        );

        let init = self.init.into_syn(crates);
        let step = self.step.into_syn(ctx, crates);
        let implementation = parse_quote!(
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

impl StateMachine {
    /// Transform [ir2] state_machine into items.
    pub fn into_syn(self, ctx: &ir0::Ctx, crates: &mut BTreeSet<String>) -> Vec<syn::Item> {
        let mut items = vec![];

        let input_structure = self.input.into_syn();
        items.push(syn::Item::Struct(input_structure));

        let (state_structure, state_implementation) = self.state.into_syn(ctx, crates);
        items.push(syn::Item::Struct(state_structure));
        items.push(syn::Item::Impl(state_implementation));

        items
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
        assert_eq!(init.into_syn(&mut Default::default()), control)
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
        assert_eq!(init.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_node_step() {
        let init = Step {
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
        assert_eq!(
            init.into_syn(&ir0::Ctx::empty(), &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_ir2_ext_node_step() {
        let init = Step {
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
        assert_eq!(
            init.into_syn(&ir0::Ctx::empty(), &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_structure_from_ir2_node_input() {
        let input = Input {
            node_name: Loc::test_id("Node"),
            elements: vec![InputElm {
                identifier: Loc::test_id("i"),
                typ: Typ::int(),
            }],
        };
        let control = parse_quote!(
            pub struct NodeInput {
                pub i: i64,
            }
        );

        assert_eq!(input.into_syn(), control)
    }
}
