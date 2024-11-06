//! LIR [Contract] module.

prelude! {
    operator::{BinaryOperator, UnaryOperator}
}

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
        identifier: String,
    },
    /// A memory access: `self.i_mem`.
    MemoryAccess {
        /// The identifier to the memory.
        identifier: String,
    },
    /// An input access: `self.i_mem`.
    InputAccess {
        /// The identifier to the input.
        identifier: String,
    },
    /// An unitary operation: `!x`.
    Unop {
        /// The operator.
        op: UnaryOperator,
        /// The expression.
        term: Box<Self>,
    },
    /// A binary operation: `x + y`.
    Binop {
        /// The operator.
        op: BinaryOperator,
        /// The left expression.
        left: Box<Self>,
        /// The right expression.
        right: Box<Self>,
    },
    /// Identifier term: x
    Forall {
        /// The identifier's name.
        name: String,
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
        enum_name: String,
        /// The element name.
        elem_name: String,
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
}

mk_new! { impl Term =>
    Literal: literal {
        literal: Constant,
    }
    Identifier: ident {
        identifier: impl Into<String> = identifier.into(),
    }
    MemoryAccess: mem {
        identifier: impl Into<String> = identifier.into(),
    }
    InputAccess: input {
        identifier: impl Into<String> = identifier.into(),
    }
    Unop: unop {
        op: UnaryOperator,
        term: Self = term.into(),
    }
    Binop: binop {
        op: BinaryOperator,
        left: Self = left.into(),
        right: Self = right.into(),
    }
    Forall: forall {
        name: impl Into<String> = name.into(),
        ty: Typ,
        term: Term = term.into(),
    }
    Implication: implication {
        left: Term = left.into(),
        right: Term = right.into(),
    }
    Enumeration: enumeration {
        enum_name: impl Into<String> = enum_name.into(),
        elem_name: impl Into<String> = elem_name.into(),
        element: Option<Term> = element.map(Term::into),
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
                let ts_op = op.to_syn();
                quote!(#ts_op #ts_term)
            }
            Self::Binop { op, left, right } => {
                let ts_left = left.to_token_stream(prophecy, function_like);
                let ts_right = right.to_token_stream(prophecy, function_like);
                let ts_op = op.to_syn();
                quote!(#ts_left #ts_op #ts_right)
            }
            Self::Literal { literal } => {
                let expr = literal.to_syn();
                quote!(#expr)
            }
            Self::Identifier { identifier } => {
                let id = Ident::new(&identifier, Span::call_site());
                quote!(#id)
            }
            Self::MemoryAccess { identifier } => {
                if function_like {
                    unreachable!()
                } else {
                    let id = format_ident!("last_{}", identifier);
                    if prophecy {
                        quote!((^self).#id)
                    } else {
                        quote!(self.#id)
                    }
                }
            }
            Self::InputAccess { identifier } => {
                let id = Ident::new(&identifier, Span::call_site());
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
                let id = Ident::new(&name, Span::call_site());
                let ts_term = term.to_token_stream(prophecy, function_like);
                let ts_ty = ty.to_syn();
                quote!(forall<#id:#ts_ty> #ts_term)
            }
            Self::Enumeration {
                enum_name,
                elem_name,
                element,
            } => {
                let ty = Ident::new(&enum_name, Span::call_site());
                let cons = Ident::new(&elem_name, Span::call_site());
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
    pub fn to_syn(self, function_like: bool) -> Vec<syn::Attribute> {
        let Self {
            requires,
            ensures,
            invariant,
        } = self;
        let mut attributes =
            Vec::with_capacity(requires.len() + ensures.len() + 2 * invariant.len());
        for term in requires {
            let ts = term.to_token_stream(false, function_like);
            attributes.push(parse_quote!(#[requires(#ts)]))
        }
        for term in ensures {
            let ts = term.to_token_stream(false, function_like);
            attributes.push(parse_quote!(#[ensures(#ts)]))
        }
        for term in invariant {
            let ts_pre = term.clone().to_token_stream(false, function_like);
            let ts_cur = term.clone().to_token_stream(true, function_like);
            attributes.push(parse_quote!(#[requires(#ts_pre)]));
            attributes.push(parse_quote!(#[ensures(#ts_cur)]));
        }
        attributes
    }
}
