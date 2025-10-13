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
    /// Parenthesized term: `(x+y)`.
    Paren {
        /// The parenthesized term.
        term: Box<Self>,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: Ident,
        /// True if its type needs logical model.
        views: bool,
    },
    /// A memory access: `self.i_mem`.
    MemoryAccess {
        /// The identifier to the memory.
        identifier: Ident,
        /// True if its type needs logical model.
        views: bool,
    },
    /// An input access: `input.i`.
    InputAccess {
        /// The identifier to the input.
        identifier: Ident,
        /// True if its type needs logical model.
        views: bool,
    },
    /// An output access: `result.o`.
    OutputAccess {
        /// The identifier to the output.
        identifier: Ident,
        /// True if its type needs logical model.
        views: bool,
    },
    /// An unary operation: `!x`.
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
        function: Ident,
        /// The arguments.
        arguments: Vec<Self>,
    },
    /// A component call: `self.called_comp.step(inputs)`.
    ComponentCall {
        /// Component's identifier in memory.
        memory_ident: Ident,
        /// The identifier to the component.
        comp_name: Ident,
        /// The name of the input structure of the called component.
        input_name: Ident,
        /// The filled input's fields.
        input_fields: Vec<(Ident, Self)>,
    },
}

mk_new! { impl Term =>
    Literal: literal {
        literal: Constant,
    }
    Paren: paren { term: Self = term.into() }
    Identifier: ident {
        identifier: impl Into<Ident> = identifier.into(),
        views: bool,
    }
    MemoryAccess: mem {
        identifier: impl Into<Ident> = identifier.into(),
        views: bool,
    }
    InputAccess: input {
        identifier: impl Into<Ident> = identifier.into(),
        views: bool,
    }
    OutputAccess: output {
        identifier: impl Into<Ident> = identifier.into(),
        views: bool,
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
        function: impl Into<Ident> = function.into(),
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
    /// - `prophecy` changes the way `Self::MemoryAccess` identifiers are printed:
    ///   - `(^self).last_<id>` if true,
    ///   - `self.last_<id>` otherwise.
    pub fn prepare_tokens(&self, prophecy: bool, function_like: bool) -> TermTokens {
        TermTokens {
            term: self,
            prophecy,
            function_like,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct TermTokens<'a> {
    term: &'a Term,
    prophecy: bool,
    function_like: bool,
}
impl<'a> TermTokens<'a> {
    /// Swaps the underlying term.
    fn set_term(&self, term: &'a Term) -> Self {
        Self {
            term,
            prophecy: self.prophecy,
            function_like: self.function_like,
        }
    }
}

impl ToTokens for TermTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.term {
            Term::Paren { term } => {
                let term = self.set_term(term);
                quote!( (#term) ).to_tokens(tokens)
            }
            Term::Unop { op, term } => {
                let term = self.set_term(term);
                quote!(#op #term).to_tokens(tokens)
            }
            Term::Binop { op, left, right } => {
                let lft = self.set_term(left);
                let rgt = self.set_term(right);
                quote!(#lft #op #rgt).to_tokens(tokens)
            }
            Term::Literal { literal } => literal.to_logic_tokens(tokens),
            Term::Identifier { identifier, views } => {
                identifier.to_tokens(tokens);
                if *views {
                    quote!(@).to_tokens(tokens)
                }
            }
            Term::MemoryAccess { identifier, views } => {
                if self.function_like {
                    noErrorDesc!("unexpected function-like memory access")
                } else {
                    let id = identifier.to_last_var();
                    if self.prophecy {
                        quote!((^self).).to_tokens(tokens)
                    } else {
                        quote!(self.).to_tokens(tokens)
                    }
                    id.to_tokens(tokens)
                }
                if *views {
                    quote!(@).to_tokens(tokens)
                }
            }
            Term::InputAccess { identifier, views } => {
                if self.function_like {
                    identifier.to_tokens(tokens)
                } else {
                    quote!(input.).to_tokens(tokens);
                    identifier.to_tokens(tokens)
                };
                if *views {
                    quote!(@).to_tokens(tokens)
                }
            }
            Term::OutputAccess { identifier, views } => {
                if self.function_like {
                    identifier.to_tokens(tokens)
                } else {
                    quote!(result.).to_tokens(tokens);
                    identifier.to_tokens(tokens)
                };
                if *views {
                    quote!(@).to_tokens(tokens)
                }
            }
            Term::Implication { left, right } => {
                self.set_term(left).to_tokens(tokens);
                quote!(==>).to_tokens(tokens);
                self.set_term(right).to_tokens(tokens);
            }
            Term::Forall { name, ty, term } => {
                let term = self.set_term(term);
                quote!(forall < #name : #ty > #term).to_tokens(tokens)
            }
            Term::Enumeration {
                enum_name,
                elem_name,
                element,
            } => {
                enum_name.to_tokens(tokens);
                syn::token::PathSep::default().to_tokens(tokens);
                elem_name.to_tokens(tokens);
                if let Some(term) = element {
                    let term = self.set_term(term);
                    quote!((#term)).to_tokens(tokens)
                }
            }
            Term::Ok { term } => {
                let term = self.set_term(term);
                quote!(Ok(#term)).to_tokens(tokens)
            }
            Term::Err => quote!(Err(())).to_tokens(tokens),
            Term::Some { term } => {
                let term = self.set_term(term);
                quote!(Some(#term)).to_tokens(tokens)
            }
            Term::None => quote!(None).to_tokens(tokens),
            Term::FunctionCall {
                function,
                arguments,
            } => {
                let args = arguments.iter().map(|term| self.set_term(term));
                quote!(logical::#function(#(#args),*)).to_tokens(tokens)
            }
            Term::ComponentCall { .. } => {
                panic!("`ir2::Contract::to_tokens` does not support component calls yet")
            }
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
    pub fn empty() -> Self {
        Self {
            requires: Vec::with_capacity(0),
            ensures: Vec::with_capacity(0),
            invariant: Vec::with_capacity(0),
        }
    }
}

pub struct ContractTokens<'a> {
    contract: &'a Contract,
    function_like: bool,
}
impl Contract {
    pub fn prepare_tokens(&self, function_like: bool) -> ContractTokens {
        ContractTokens {
            contract: self,
            function_like,
        }
    }
}

impl ToTokens for ContractTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for term in self.contract.requires.iter() {
            let term = term.prepare_tokens(false, self.function_like);
            quote!(#[requires(#term)]).to_tokens(tokens);
        }
        for term in self.contract.ensures.iter() {
            let term = term.prepare_tokens(false, self.function_like);
            quote!(#[ensures(#term)]).to_tokens(tokens)
        }
        for term in self.contract.invariant.iter() {
            let term_pre = term.prepare_tokens(false, self.function_like);
            let term_cur = term.prepare_tokens(true, self.function_like);
            quote!(#[requires(#term_pre)]).to_tokens(tokens);
            quote!(#[ensures(#term_cur)]).to_tokens(tokens);
        }
    }
}
