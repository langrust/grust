//! [Contract] module.

prelude! {}

#[derive(Debug, PartialEq, Clone)]
/// Term.
pub enum Term {
    /// A literal expression: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: Ident,
    },
    /// A memory access: `self.i_mem`.
    MemoryAccess {
        /// The identifier to the memory.
        identifier: Ident,
    },
    /// An input access: `self.i_mem`.
    InputAccess {
        /// The identifier to the input.
        identifier: Ident,
    },
    /// An unitary operation: `!x`.
    Unop {
        /// The operator.
        op: UOp,
        /// The expression.
        term: Box<Self>,
    },
    /// A binary operation: `x + y`.
    Binop {
        /// The operator.
        op: BOp,
        /// The left expression.
        left: Box<Self>,
        /// The right expression.
        right: Box<Self>,
    },
    /// Identifier term: x
    Forall {
        /// The identifier's name.
        name: Ident,
        /// The identifier's type.
        ty: Typ,
        /// The term
        term: Box<Term>,
    },
    /// Implication term: x => y
    Implication {
        /// Left term
        left: Box<Term>,
        /// Right term
        right: Box<Term>,
    },
    /// Enumeration term.
    Enumeration {
        /// The enumeration type name.
        enum_name: Ident,
        /// The element name.
        elem_name: Ident,
        /// The optional element of the enumeration.
        element: Option<Box<Term>>,
    },
    /// Ok term.
    Ok {
        /// The pattern matching the value.
        term: Box<Term>,
    },
    /// Err term.
    Err,
    /// Some term.
    Some {
        /// The pattern matching the value.
        term: Box<Term>,
    },
    /// None term.
    None,
    /// A function call: `foo(x, y)`.
    FunctionCall {
        /// The function called.
        function: Box<Self>,
        /// The arguments.
        arguments: Vec<Self>,
    },
    /// A component call: `self.called_node.step(inputs)`.
    ComponentCall {
        /// Component's identifier in memory.
        memory_ident: Ident,
        /// The identifier to the node.
        comp_name: Ident,
        /// The name of the input structure of the called node.
        input_name: Ident,
        /// The filled input's fields.
        input_fields: Vec<(Ident, Self)>,
    },
}

mk_new! { impl Term =>
    Literal: literal {
        literal: Constant,
    }
    Identifier: ident {
        identifier: impl Into<Ident> = identifier.into(),
    }
    MemoryAccess: mem {
        identifier: impl Into<Ident> = identifier.into(),
    }
    InputAccess: input {
        identifier: impl Into<Ident> = identifier.into(),
    }
    Unop: unop {
        op: UOp,
        term: Self = term.into(),
    }
    Binop: binop {
        op: BOp,
        left: Self = left.into(),
        right: Self = right.into(),
    }
    Forall: forall {
        name: impl Into<Ident> = name.into(),
        ty: Typ,
        term: Term = term.into(),
    }
    Implication: implication {
        left: Term = left.into(),
        right: Term = right.into(),
    }
    Enumeration: enumeration {
        enum_name: impl Into<Ident> = enum_name.into(),
        elem_name: impl Into<Ident> = elem_name.into(),
        element: Option<Term> = element.map(Term::into),
    }
    FunctionCall: fun_call {
        function: Self = function.into(),
        arguments: impl Into<Vec<Self>> = arguments.into(),
    }
    ComponentCall: comp_call {
        memory_ident: impl Into<Ident> = memory_ident.into() ,
        comp_name: impl Into<Ident> = comp_name.into(),
        input_name: impl Into<Ident> = input_name.into(),
        input_fields: impl Into<Vec<(Ident, Self)>> = input_fields.into(),
    }
    Ok: ok { term: Term = term.into() }
    Err: err {}
    Some: some { term: Term = term.into() }
    None: none {}
}

impl Term {
    /// Tokens stream for a term.
    ///
    /// - `prophecy`: changes the way `Self::MemoryAccess` identifiers are printed.
    ///   - `(^self).last_<id>` if true,
    ///   - `self.last_<id>` otherwise.
    pub fn to_token_stream(self, prophecy: bool, function_like: bool) -> TokenStream2 {
        use quote::quote;
        match self {
            Self::Unop { op, term } => {
                let ts_term = term.to_token_stream(prophecy, function_like);
                let ts_op = op.into_syn();
                quote!(#ts_op #ts_term)
            }
            Self::Binop { op, left, right } => {
                let ts_left = left.to_token_stream(prophecy, function_like);
                let ts_right = right.to_token_stream(prophecy, function_like);
                let ts_op = op.into_syn();
                quote!(#ts_left #ts_op #ts_right)
            }
            Self::Literal { literal } => {
                let expr = literal.into_syn();
                quote!(#expr)
            }
            Self::Identifier { identifier } => {
                let id = identifier;
                quote!(#id)
            }
            Self::MemoryAccess { identifier } => {
                if function_like {
                    unreachable!()
                } else {
                    let id = identifier.to_last_var();
                    if prophecy {
                        quote!((^self).#id)
                    } else {
                        quote!(self.#id)
                    }
                }
            }
            Self::InputAccess { identifier } => {
                let id = identifier;
                if function_like {
                    quote!(#id)
                } else {
                    quote!(input.#id)
                }
            }
            Self::Implication { left, right } => {
                let ts_left = left.to_token_stream(prophecy, function_like);
                let ts_right = right.to_token_stream(prophecy, function_like);
                quote!(#ts_left ==> #ts_right)
            }
            Self::Forall { name, ty, term } => {
                let id = name;
                let ts_term = term.to_token_stream(prophecy, function_like);
                let ts_ty = ty.into_syn();
                quote!(forall<#id:#ts_ty> #ts_term)
            }
            Self::Enumeration {
                enum_name,
                elem_name,
                element,
            } => {
                let ty = enum_name;
                let cons = elem_name;
                if let Some(term) = element {
                    let inner = term.to_token_stream(prophecy, function_like);
                    parse_quote! { #ty::#cons(#inner) }
                } else {
                    parse_quote! { #ty::#cons }
                }
            }
            Self::Ok { term } => {
                let ts_term = term.to_token_stream(prophecy, function_like);
                parse_quote! { Ok(#ts_term) }
            }
            Self::Err => parse_quote! { Err(()) },
            Self::Some { term } => {
                let ts_term = term.to_token_stream(prophecy, function_like);
                parse_quote! { Some(#ts_term) }
            }
            Self::None => parse_quote! { None },
            Self::FunctionCall { .. } | Self::ComponentCall { .. } => todo!("not supported yet"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
/// Contract to prove using Creusot.
pub struct Contract {
    /// Requirements clauses to suppose
    pub requires: Vec<Term>,
    /// Ensures clauses to prove
    pub ensures: Vec<Term>,
    /// Invariant clauses to prove
    pub invariant: Vec<Term>,
}

mk_new! { impl Contract => new {
    requires: Vec<Term>,
    ensures: Vec<Term>,
    invariant: Vec<Term>,
} }

impl Contract {
    pub fn into_syn(self, function_like: bool) -> Vec<syn::Attribute> {
        let mut attributes =
            Vec::with_capacity(self.requires.len() + self.ensures.len() + 2 * self.invariant.len());
        for term in self.requires {
            let ts = term.to_token_stream(false, function_like);
            attributes.push(parse_quote!(#[requires(#ts)]))
        }
        for term in self.ensures {
            let ts = term.to_token_stream(false, function_like);
            attributes.push(parse_quote!(#[ensures(#ts)]))
        }
        for term in self.invariant {
            let ts_pre = term.clone().to_token_stream(false, function_like);
            let ts_cur = term.clone().to_token_stream(true, function_like);
            attributes.push(parse_quote!(#[requires(#ts_pre)]));
            attributes.push(parse_quote!(#[ensures(#ts_cur)]));
        }
        attributes
    }
}
