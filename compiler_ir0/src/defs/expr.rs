prelude! {}

/// UnOp expression.
#[derive(Debug, PartialEq, Clone)]
pub struct UnOp<E> {
    /// The unary operator.
    pub op: UOp,
    /// The input expression.
    pub expr: Box<E>,
}

mk_new! { impl{E} UnOp<E> =>
    new {
        op : UOp,
        expr: impl Into<Box<E>> = expr.into(),
    }

}

/// Binop expression.
///
/// TODO: precedence
#[derive(Debug, PartialEq, Clone)]
pub struct Binop<E> {
    /// The unary operator.
    pub op: BOp,
    /// The left expression.
    pub lft: Box<E>,
    /// The right expression.
    pub rgt: Box<E>,
}

mk_new! { impl{E} Binop<E> =>
    new {
        op : BOp,
        lft: impl Into<Box<E>> = lft.into(),
        rgt: impl Into<Box<E>> = rgt.into(),
    }

}

/// IfThenElse expression.
#[derive(Debug, PartialEq, Clone)]
pub struct IfThenElse<E> {
    /// Condition.
    pub cnd: Box<E>,
    /// `then` branch.
    pub thn: Box<E>,
    /// `else` branch.
    pub els: Box<E>,
}

mk_new! { impl{E} IfThenElse<E> =>
    new {
        cnd: impl Into<Box<E>> = cnd.into(),
        thn: impl Into<Box<E>> = thn.into(),
        els: impl Into<Box<E>> = els.into()
    }
}

/// Application expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Application<E> {
    /// The expression applied.
    pub fun: Box<E>,
    /// The inputs to the expression.
    pub inputs: Vec<E>,
}

mk_new! { impl{E} Application<E> =>
    new {
        fun: impl Into<Box<E>> = fun.into(),
        inputs: Vec<E>,
    }
}

/// Abstraction expression with inputs types.
#[derive(Debug, PartialEq, Clone)]
pub struct TypedAbstraction<E> {
    /// The inputs to the abstraction.
    pub inputs: Vec<(String, Typ)>,
    /// The expression abstracted.
    pub expr: Box<E>,
}

mk_new! { impl{E} TypedAbstraction<E> =>
    new {
        inputs: Vec<(String, Typ)>,
        expr: impl Into<Box<E>> = expr.into(),
    }
}

/// Structure expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Structure<E> {
    /// The structure name.
    pub name: String,
    /// The fields associated with their expressions.
    pub fields: Vec<(String, E)>,
}

mk_new! { impl{E} Structure<E> =>
    new {
        name: impl Into<String> = name.into(),
        fields: Vec<(String, E)>,
    }
}

/// Tuple expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple<E> {
    /// The elements.
    pub elements: Vec<E>,
}

mk_new! { impl{E} Tuple<E> =>
    new { elements: Vec<E> }
}

/// Enumeration expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration<E> {
    /// The enumeration name.
    pub enum_name: String,
    /// The enumeration element.
    pub elem_name: String,
    /// Marker for the unused type param.
    pub mark: std::marker::PhantomData<E>,
}

impl<E> Enumeration<E> {
    pub fn new(enum_name: impl Into<String>, elem_name: impl Into<String>) -> Self {
        Self {
            enum_name: enum_name.into(),
            elem_name: elem_name.into(),
            mark: std::marker::PhantomData,
        }
    }
}

/// Array expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Array<E> {
    /// The elements inside the array.
    pub elements: Vec<E>,
}

mk_new! { impl{E} Array<E> =>
    new { elements: Vec<E> }
}

/// Structure pattern that matches the structure and its fields.
#[derive(Debug, PartialEq, Clone)]
pub struct PatStructure {
    /// The structure name.
    pub name: String,
    /// The structure fields with the corresponding patterns to match.
    pub fields: Vec<(String, Option<Pattern>)>,
    /// The rest of the fields
    pub rest: Option<Token![..]>,
}
mk_new! { impl PatStructure =>
    new {
        name: impl Into<String> = name.into(),
        fields: Vec<(String, Option<Pattern>)>,
        rest: Option<Token![..]>,
    }
}

/// Enumeration pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct PatEnumeration {
    /// The enumeration type name.
    pub enum_name: String,
    /// The element name.
    pub elem_name: String,
}
mk_new! { impl PatEnumeration =>
    new {
        enum_name: impl Into<String> = enum_name.into(),
        elem_name: impl Into<String> = elem_name.into(),
    }
}

/// Tuple pattern that matches tuples.
#[derive(Debug, PartialEq, Clone)]
pub struct PatTuple {
    /// The elements of the tuple.
    pub elements: Vec<Pattern>,
}
mk_new! { impl PatTuple =>
    new { elements: Vec<Pattern> }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust matching pattern AST.
pub enum Pattern {
    /// Constant pattern.
    Constant(Constant),
    /// Identifier pattern.
    Identifier(String),
    /// Structure pattern that matches the structure and its fields.
    Structure(PatStructure),
    /// Enumeration pattern.
    Enumeration(PatEnumeration),
    /// Tuple pattern that matches tuples.
    Tuple(PatTuple),
    /// Default pattern.
    Default,
}
impl Pattern {
    mk_new! {
        Constant: constant(c: Constant = c)
        Constant: cst(c: Constant = c)
        Identifier: ident(s: impl Into<String> = s.into())
        Structure: structure(s: PatStructure = s)
        Enumeration: enumeration(e: PatEnumeration = e)
        Tuple: tuple(t: PatTuple = t)
        Default: default()
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
    /// The expression to match.
    pub expr: Box<E>,
    /// The different matching cases.
    pub arms: Vec<Arm<E>>,
}

mk_new! { impl{E} Match<E> =>
    new {
        expr: impl Into<Box<E>> = expr.into(),
        arms: Vec<Arm<E>>,
    }
}

/// Field access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<E> {
    /// The structure expression.
    pub expr: Box<E>,
    /// The field to access.
    pub field: String,
}

mk_new! { impl{E} FieldAccess<E> =>
    new {
        expr: impl Into<Box<E>> = expr.into(),
        field: impl Into<String> = field.into(),
    }
}

/// Tuple element access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleElementAccess<E> {
    /// The tuple expression.
    pub expr: Box<E>,
    /// The element to access.
    pub element_number: usize,
}

mk_new! { impl{E} TupleElementAccess<E> =>
    new {
        expr: impl Into<Box<E>> = expr.into(),
        element_number: usize,
    }
}

/// Array map operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Map<E> {
    /// The array expression.
    pub expr: Box<E>,
    /// The function expression.
    pub fun: Box<E>,
}

mk_new! { impl{E} Map<E> =>
    new {
        expr: impl Into<Box<E>> = expr.into(),
        fun: impl Into<Box<E>> = fun.into(),
    }
}

/// Array fold operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Fold<E> {
    /// The array expression.
    pub array: Box<E>,
    /// The initialization expression.
    pub init: Box<E>,
    /// The function expression.
    pub fun: Box<E>,
}

mk_new! { impl{E} Fold<E> =>
    new {
        array: impl Into<Box<E>> = array.into(),
        init: impl Into<Box<E>> = init.into(),
        fun: impl Into<Box<E>> = fun.into(),
    }
}

/// Array sort operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Sort<E> {
    /// The array expression.
    pub expr: Box<E>,
    /// The function expression.
    pub fun: Box<E>,
}

mk_new! { impl{E} Sort<E> =>
    new {
        expr: impl Into<Box<E>> = expr.into(),
        fun: impl Into<Box<E>> = fun.into(),
    }
}

/// Arrays zip operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Zip<E> {
    /// The array expressions.
    pub arrays: Vec<E>,
}

mk_new! { impl{E} Zip<E> =>
    new { arrays: Vec<E> }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust expression AST.
pub enum Expr {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(String),
    /// UnOp expression.
    UnOp(UnOp<Self>),
    /// Binop expression.
    Binop(Binop<Self>),
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
    /// Array map operator expression.
    Map(Map<Self>),
    /// Array fold operator expression.
    Fold(Fold<Self>),
    /// Array sort operator expression.
    Sort(Sort<Self>),
    /// Arrays zip operator expression.
    Zip(Zip<Self>),
}

mk_new! { impl Expr =>
    Constant: constant (val: Constant = val)
    Constant: cst (val: Constant = val)
    Identifier: ident (val: impl Into<String> = val.into())
    UnOp: unop (val: UnOp<Self> = val)
    Binop: binop (val: Binop<Self> = val)
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
    Map: map (val: Map<Self> = val)
    Fold: fold (val: Fold<Self> = val)
    Sort: sort (val: Sort<Self> = val)
    Zip: zip (val: Zip<Self> = val)
}
