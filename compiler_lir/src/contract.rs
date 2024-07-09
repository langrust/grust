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
