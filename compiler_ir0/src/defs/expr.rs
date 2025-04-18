prelude! {}

/// UnOp expression.
#[derive(Debug, PartialEq, Clone)]
pub struct UnOp<E> {
    /// The unary operator.
    pub op: UOp,
    /// Operator's location.
    pub op_loc: Loc,
    /// The input expression.
    pub expr: Box<E>,
}
impl<E: HasLoc> HasLoc for UnOp<E> {
    fn loc(&self) -> Loc {
        self.op_loc.join(self.expr.loc())
    }
}
mk_new! { impl{E} UnOp<E> =>
    new {
        op : UOp,
        op_loc: Loc,
        expr: impl Into<Box<E>> = expr.into(),
    }

}

/// Binary operator.
///
/// TODO: precedence
#[derive(Debug, PartialEq, Clone)]
pub struct BinOp<E> {
    /// The unary operator.
    pub op: BOp,
    /// Operator's location.
    pub op_loc: Loc,
    /// The left expression.
    pub lft: Box<E>,
    /// The right expression.
    pub rgt: Box<E>,
}
impl<E: HasLoc> HasLoc for BinOp<E> {
    fn loc(&self) -> Loc {
        self.lft.loc().join(self.rgt.loc())
    }
}
mk_new! { impl{E} BinOp<E> =>
    new {
        op : BOp,
        op_loc: Loc,
        lft: impl Into<Box<E>> = lft.into(),
        rgt: impl Into<Box<E>> = rgt.into(),
    }

}

/// IfThenElse expression.
#[derive(Debug, PartialEq, Clone)]
pub struct IfThenElse<E> {
    /// Location.
    pub loc: Loc,
    /// Condition.
    pub cnd: Box<E>,
    /// `then` branch.
    pub thn: Box<E>,
    /// `else` branch.
    pub els: Box<E>,
}
impl<E> HasLoc for IfThenElse<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} IfThenElse<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        cnd: impl Into<Box<E>> = cnd.into(),
        thn: impl Into<Box<E>> = thn.into(),
        els: impl Into<Box<E>> = els.into(),
    }
}

/// Application expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Application<E> {
    /// Location.
    pub loc: Loc,
    /// The expression applied.
    pub fun: Box<E>,
    /// The inputs to the expression.
    pub inputs: Vec<E>,
}
impl<E> HasLoc for Application<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} Application<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        fun: impl Into<Box<E>> = fun.into(),
        inputs: Vec<E>,
    }
}

/// Abstraction expression with inputs types.
#[derive(Debug, PartialEq, Clone)]
pub struct TypedAbstraction<E> {
    pub loc: Loc,
    /// The inputs to the abstraction.
    pub inputs: Vec<(Ident, Typ)>,
    /// The expression abstracted.
    pub expr: Box<E>,
}
impl<E> HasLoc for TypedAbstraction<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}

mk_new! { impl{E} TypedAbstraction<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        inputs: Vec<(Ident, Typ)>,
        expr: impl Into<Box<E>> = expr.into(),
    }
}

/// Structure expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Structure<E> {
    /// Location.
    pub loc: Loc,
    /// The structure name.
    pub name: Ident,
    /// The fields associated with their expressions.
    pub fields: Vec<(Ident, E)>,
}
impl<E> HasLoc for Structure<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}

mk_new! { impl{E} Structure<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        name: impl Into<Ident> = name.into(),
        fields: Vec<(Ident, E)>,
    }
}

/// Tuple expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple<E> {
    /// Location.
    pub loc: Loc,
    /// The elements.
    pub elements: Vec<E>,
}
impl<E> HasLoc for Tuple<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}

mk_new! { impl{E} Tuple<E> => new {
    loc: impl Into<Loc> = loc.into(),
    elements: Vec<E>,
} }

/// Enumeration expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration<E> {
    /// Location.
    pub loc: Loc,
    /// The enumeration name.
    pub enum_name: Ident,
    /// The enumeration element.
    pub elem_name: Ident,
    /// Marker for the unused type param.
    pub mark: std::marker::PhantomData<E>,
}
impl<E> HasLoc for Enumeration<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}

impl<E> Enumeration<E> {
    pub fn new(
        loc: impl Into<Loc>,
        enum_name: impl Into<Ident>,
        elem_name: impl Into<Ident>,
    ) -> Self {
        Self {
            loc: loc.into(),
            enum_name: enum_name.into(),
            elem_name: elem_name.into(),
            mark: std::marker::PhantomData,
        }
    }
}

/// Array expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Array<E> {
    /// Location.
    pub loc: Loc,
    /// The elements inside the array.
    pub elements: Vec<E>,
}
impl<E> HasLoc for Array<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} Array<E> => new {
    loc: impl Into<Loc> = loc.into(),
    elements: Vec<E>,
} }

/// Structure pattern that matches the structure and its fields.
#[derive(Debug, PartialEq, Clone)]
pub struct PatStructure {
    pub braces: syn::token::Brace,
    /// The structure name.
    pub name: Ident,
    /// The structure fields with the corresponding patterns to match.
    pub fields: Vec<(Ident, Option<Pattern>)>,
    /// The rest of the fields
    pub rest: Option<Token![..]>,
}
impl HasLoc for PatStructure {
    fn loc(&self) -> Loc {
        self.name.loc().join(self.braces.span.join())
    }
}
mk_new! { impl PatStructure =>
    new {
        braces: syn::token::Brace,
        name: impl Into<Ident> = name.into(),
        fields: Vec<(Ident, Option<Pattern>)>,
        rest: Option<Token![..]>,
    }
}

/// Enumeration pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct PatEnumeration {
    /// The enumeration type name.
    pub enum_name: Ident,
    /// The element name.
    pub elem_name: Ident,
}
impl HasLoc for PatEnumeration {
    fn loc(&self) -> Loc {
        self.enum_name.loc().join(self.elem_name.loc())
    }
}
mk_new! { impl PatEnumeration =>
    new {
        enum_name: impl Into<Ident> = enum_name.into(),
        elem_name: impl Into<Ident> = elem_name.into(),
    }
}

/// Tuple pattern that matches tuples.
#[derive(Debug, PartialEq, Clone)]
pub struct PatTuple {
    pub parens: syn::token::Paren,
    /// The elements of the tuple.
    pub elements: Vec<Pattern>,
}
impl HasLoc for PatTuple {
    fn loc(&self) -> Loc {
        self.parens.span.join().into()
    }
}
mk_new! { impl PatTuple => new {
    parens: syn::token::Paren,
    elements: Vec<Pattern>,
} }

#[derive(Debug, PartialEq, Clone)]
/// GRust matching pattern AST.
pub enum Pattern {
    /// Constant pattern.
    Constant(Constant),
    /// Identifier pattern.
    Identifier(Ident),
    /// Structure pattern that matches the structure and its fields.
    Structure(PatStructure),
    /// Enumeration pattern.
    Enumeration(PatEnumeration),
    /// Tuple pattern that matches tuples.
    Tuple(PatTuple),
    /// Default pattern.
    Default(Loc),
}
impl HasLoc for Pattern {
    fn loc(&self) -> Loc {
        match self {
            Self::Constant(c) => c.loc(),
            Self::Identifier(i) => i.loc(),
            Self::Structure(s) => s.loc(),
            Self::Enumeration(e) => e.loc(),
            Self::Tuple(t) => t.loc(),
            Self::Default(loc) => *loc,
        }
    }
}
impl Pattern {
    mk_new! {
        Constant: constant(c: Constant = c)
        Constant: cst(c: Constant = c)
        Identifier: ident(
            id: impl Into<Ident> = id.into(),
        )
        Identifier: test_ident(
            s: impl AsRef<str> = Ident::new(s.as_ref(), Loc::test_dummy().into()),
        )
        Structure: structure(s: PatStructure = s)
        Enumeration: enumeration(e: PatEnumeration = e)
        Tuple: tuple(t: PatTuple = t)
        Default: default(
            loc: impl Into<Loc> = loc.into(),
        )
    }
}

/// Arm for matching expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Arm<E> {
    /// The pattern to match.
    pub pattern: Pattern,
    /// The optional guard.
    pub guard: Option<E>,
    /// The expression.
    pub expr: E,
}

mk_new! { impl{E} Arm<E> =>
    new_with_guard {
        pattern: Pattern,
        expr: E,
        guard: Option<E>,
    }
}
impl<E> Arm<E> {
    pub fn new(pattern: Pattern, expr: E) -> Self {
        Self {
            pattern,
            expr,
            guard: None,
        }
    }
}

/// Pattern matching expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Match<E> {
    /// Location.
    pub loc: Loc,
    /// The expression to match.
    pub expr: Box<E>,
    /// The different matching cases.
    pub arms: Vec<Arm<E>>,
}
impl<E> HasLoc for Match<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} Match<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        expr: impl Into<Box<E>> = expr.into(),
        arms: Vec<Arm<E>>,
    }
}

/// Field access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<E> {
    /// Location.
    pub loc: Loc,
    /// The structure expression.
    pub expr: Box<E>,
    /// The field to access.
    pub field: Ident,
}
impl<E> HasLoc for FieldAccess<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} FieldAccess<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        expr: impl Into<Box<E>> = expr.into(),
        field: impl Into<Ident> = field.into(),
    }
}

/// Tuple element access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleElementAccess<E> {
    /// Location.
    pub loc: Loc,
    /// The tuple expression.
    pub expr: Box<E>,
    /// The element to access.
    pub element_number: usize,
}
impl<E> HasLoc for TupleElementAccess<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} TupleElementAccess<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        expr: impl Into<Box<E>> = expr.into(),
        element_number: usize,
    }
}

/// Array access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct ArrayAccess<E> {
    /// Location.
    pub loc: Loc,
    /// The structure expression.
    pub expr: Box<E>,
    /// The index to access.
    pub index: syn::LitInt,
}
impl<E> HasLoc for ArrayAccess<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} ArrayAccess<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        expr: impl Into<Box<E>> = expr.into(),
        index: impl Into<syn::LitInt> = index.into(),
    }
}

/// Array map operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Map<E> {
    /// Location.
    pub loc: Loc,
    /// The array expression.
    pub expr: Box<E>,
    /// The function expression.
    pub fun: Box<E>,
}
impl<E> HasLoc for Map<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} Map<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        expr: impl Into<Box<E>> = expr.into(),
        fun: impl Into<Box<E>> = fun.into(),
    }
}

/// Array fold operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Fold<E> {
    /// Location.
    pub loc: Loc,
    /// The array expression.
    pub array: Box<E>,
    /// The initialization expression.
    pub init: Box<E>,
    /// The function expression.
    pub fun: Box<E>,
}
impl<E> HasLoc for Fold<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} Fold<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        array: impl Into<Box<E>> = array.into(),
        init: impl Into<Box<E>> = init.into(),
        fun: impl Into<Box<E>> = fun.into(),
    }
}

/// Array sort operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Sort<E> {
    /// Location.
    pub loc: Loc,
    /// The array expression.
    pub expr: Box<E>,
    /// The function expression.
    pub fun: Box<E>,
}
impl<E> HasLoc for Sort<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} Sort<E> =>
    new {
        loc: impl Into<Loc> = loc.into(),
        expr: impl Into<Box<E>> = expr.into(),
        fun: impl Into<Box<E>> = fun.into(),
    }
}

/// Arrays zip operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Zip<E> {
    /// Location.
    pub loc: Loc,
    /// The array expressions.
    pub arrays: Vec<E>,
}
impl<E> HasLoc for Zip<E> {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl{E} Zip<E> => new {
    loc: impl Into<Loc> = loc.into(),
    arrays: Vec<E>,
} }

#[derive(Debug, PartialEq, Clone)]
/// GRust expression AST.
pub enum Expr {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(Ident),
    /// UnOp expression.
    UnOp(UnOp<Self>),
    /// BinOp expression.
    BinOp(BinOp<Self>),
    /// IfThenElse expression.
    IfThenElse(IfThenElse<Self>),
    /// Application expression.
    Application(Application<Self>),
    /// Abstraction expression with inputs types.
    TypedAbstraction(TypedAbstraction<Self>),
    /// Structure expression.
    Structure(Structure<Self>),
    /// Tuple expression.
    Tuple(Tuple<Self>),
    /// Enumeration expression.
    Enumeration(Enumeration<Self>),
    /// Array expression.
    Array(Array<Self>),
    /// Pattern matching expression.
    Match(Match<Self>),
    /// Field access expression.
    FieldAccess(FieldAccess<Self>),
    /// Tuple element access expression.
    TupleElementAccess(TupleElementAccess<Self>),
    /// Array access expression.
    ArrayAccess(ArrayAccess<Self>),
    /// Array map operator expression.
    Map(Map<Self>),
    /// Array fold operator expression.
    Fold(Fold<Self>),
    /// Array sort operator expression.
    Sort(Sort<Self>),
    /// Arrays zip operator expression.
    Zip(Zip<Self>),
}

impl HasLoc for Expr {
    fn loc(&self) -> Loc {
        use Expr::*;
        match self {
            Constant(c) => c.loc(),
            Identifier(id) => id.span().into(),
            UnOp(op) => op.loc(),
            BinOp(op) => op.loc(),
            IfThenElse(ite) => ite.loc(),
            Application(app) => app.loc(),
            TypedAbstraction(ta) => ta.loc(),
            Structure(s) => s.loc(),
            Tuple(t) => t.loc(),
            Enumeration(e) => e.loc(),
            Array(a) => a.loc(),
            Match(m) => m.loc(),
            FieldAccess(fa) => fa.loc(),
            TupleElementAccess(ta) => ta.loc(),
            ArrayAccess(aa) => aa.loc(),
            Map(m) => m.loc(),
            Fold(f) => f.loc(),
            Sort(s) => s.loc(),
            Zip(z) => z.loc(),
        }
    }
}

mk_new! { impl Expr =>
    Constant: constant (val: Constant = val)
    Constant: cst (val: Constant = val)
    Identifier: ident (val: impl Into<Ident> = val.into())
    Identifier: test_ident (
        val: impl AsRef<str> = Ident::new(val.as_ref(), Loc::test_dummy().into()),
    )
    UnOp: unop (val: UnOp<Self> = val)
    BinOp: binop (val: BinOp<Self> = val)
    IfThenElse: ite (val: IfThenElse<Self> = val)
    Application: app (val: Application<Self> = val)
    TypedAbstraction: typed_abstraction (val: TypedAbstraction<Self> = val)
    Structure: structure (val: Structure<Self> = val)
    Tuple: tuple (val: Tuple<Self> = val)
    Enumeration: enumeration (val: Enumeration<Self> = val)
    Array: array (val: Array<Self> = val)
    Match: pat_match (val: Match<Self> = val)
    FieldAccess: field_access (val: FieldAccess<Self> = val)
    TupleElementAccess: tuple_access (val: TupleElementAccess<Self> = val)
    ArrayAccess: array_access (val: ArrayAccess<Self> = val)
    Map: map (val: Map<Self> = val)
    Fold: fold (val: Fold<Self> = val)
    Sort: sort (val: Sort<Self> = val)
    Zip: zip (val: Zip<Self> = val)
}
