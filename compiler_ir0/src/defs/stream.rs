prelude! {
    equation::EventPattern,
    expr::*,
}

/// Buffered signal.
#[derive(Debug, PartialEq, Clone)]
pub struct Last {
    /// Location.
    pub loc: Loc,
    /// Signal identifier.
    pub ident: Ident,
    /// The initialization constant.
    pub constant: Option<Box<Expr>>,
}
impl HasLoc for Last {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl Last =>
    new {
        loc: impl Into<Loc> = loc.into(),
        ident: Ident,
        constant: Option<Expr> = constant.map(Expr::into),
    }
}

/// Arm for when stream expression.
#[derive(Debug, PartialEq, Clone)]
pub struct EventArmWhen {
    /// The event pattern to catch.
    pub pattern: EventPattern,
    /// The optional guard.
    pub guard: Option<Box<Expr>>,
    /// The expression.
    pub expr: Expr,
}
mk_new! { impl EventArmWhen =>
    new {
        pattern: EventPattern,
        guard: Option<Box<Expr>>,
        expr: Expr,
    }
}

/// Init arm for when stream expression.
#[derive(Debug, PartialEq, Clone)]
pub struct InitArmWhen {
    pub init_token: keyword::init,
    pub arrow_token: Token![=>],
    /// The initial expression.
    pub expr: Expr,
}
mk_new! { impl InitArmWhen =>
    new {
        init_token: keyword::init,
        arrow_token: Token![=>],
        expr: Expr,
    }
}

/// Pattern matching for event expression.
#[derive(Debug, PartialEq, Clone)]
pub struct When {
    pub when_token: keyword::when,
    /// The optional init arm.
    pub init: Option<InitArmWhen>,
    /// The different event cases.
    pub arms: Vec<EventArmWhen>,
}
impl When {
    pub fn loc(&self) -> Loc {
        self.pattern.loc()
    }
}
mk_new! { impl When =>
    new {
        when_token: keyword::when,
        init: Option<InitArmWhen>,
        arms: Vec<EventArmWhen>,
    }
}

/// Emit event expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Emit {
    /// Location.
    pub loc: Loc,
    pub emit_token: keyword::emit,
    /// The expression to emit.
    pub expr: Box<Expr>,
}
impl HasLoc for Emit {
    fn loc(&self) -> Loc {
        self.loc
    }
}
mk_new! { impl Emit =>
    new {
        loc: impl Into<Loc> = loc.into(),
        emit_token: keyword::emit,
        expr: impl Into<Box<Expr >> = expr.into(),
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust stream expression kind AST.
pub enum Expr {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(Ident),
    /// Application expression.
    Application(Application<Self>),
    /// UnOp expression.
    UnOp(UnOp<Self>),
    /// BinOp expression.
    BinOp(BinOp<Self>),
    /// IfThenElse expression.
    IfThenElse(IfThenElse<Self>),
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
    /// Initialized buffer stream expression.
    Last(Last),
    /// Emit event.
    Emit(Emit),
}
mk_new! { impl Expr =>
    Constant: cst(arg: Constant = arg)
    Identifier: ident(arg : impl Into<Ident> = arg.into())
    Identifier: test_ident(arg: impl AsRef<str> = Ident::new(arg.as_ref(), Span::mixed_site()))
    Application: app(arg : Application<Self> = arg)
    UnOp: unop(arg: UnOp<Self> = arg)
    BinOp: binop(arg: BinOp<Self> = arg)
    IfThenElse: ite(arg: IfThenElse<Self> = arg)
    TypedAbstraction: type_abstraction(arg: TypedAbstraction<Self> = arg)
    Structure: structure(arg: Structure<Self> = arg)
    Tuple: tuple(arg: Tuple<Self> = arg)
    Enumeration: enumeration(arg: Enumeration<Self> = arg)
    Array: array(arg: Array<Self> = arg)
    Match: pat_match(arg: Match<Self> = arg)
    FieldAccess: field_access(arg: FieldAccess<Self> = arg)
    TupleElementAccess: tuple_access(arg: TupleElementAccess<Self> = arg)
    Map: map(arg: Map<Self> = arg)
    Fold: fold(arg: Fold<Self> = arg)
    Sort: sort(arg: Sort<Self> = arg)
    Zip: zip(arg: Zip<Self> = arg)
    Last: last(arg: Last = arg)
    Emit: emit(arg: Emit = arg)
}

impl HasLoc for Expr {
    fn loc(&self) -> Loc {
        use stream::Expr::*;
        match self {
            Constant(c) => c.loc(),
            Identifier(id) => id.span().into(),
            Application(app) => {
                let loc = app.fun.loc();
                app.inputs.iter().fold(loc, |acc, i| acc.join(i.loc()))
            }
            UnOp(op) => op.op_loc.join(op.expr.loc()),
            BinOp(op) => op.op_loc.join(op.lft.loc()).join(op.rgt.loc()),
            IfThenElse(ite) => ite.cnd.loc().join(ite.thn.loc()).join(ite.els.loc()),
            TypedAbstraction(abs) => abs.loc(),
            Structure(s) => s.loc(),
            Tuple(t) => t.loc(),
            Enumeration(e) => e.loc(),
            Array(a) => a.loc(),
            Match(m) => m.loc(),
            FieldAccess(fa) => fa.loc(),
            TupleElementAccess(ta) => ta.loc(),
            Map(m) => m.loc(),
            Fold(f) => f.loc(),
            Sort(s) => s.loc(),
            Zip(z) => z.loc(),
            Last(l) => l.loc(),
            Emit(e) => e.loc(),
        }
    }
}

impl Expr {
    pub fn check_is_constant(&self, table: &SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        match &self {
            // Constant by default
            stream::Expr::Constant { .. } | stream::Expr::Enumeration { .. } => Ok(()),
            // Not constant by default
            stream::Expr::TypedAbstraction { .. }
            | stream::Expr::Match { .. }
            | stream::Expr::Emit { .. }
            | stream::Expr::FieldAccess { .. }
            | stream::Expr::TupleElementAccess { .. }
            | stream::Expr::Map { .. }
            | stream::Expr::Fold { .. }
            | stream::Expr::Sort { .. }
            | stream::Expr::Zip { .. }
            | stream::Expr::Last { .. } => {
                let loc = self.loc();
                bad!(errors, @loc => ErrorKind::expected_constant())
            }
            // It depends
            stream::Expr::Identifier(ident) => {
                // check id exists
                let id = table
                    .get_identifier_id(ident, false, &mut vec![])
                    .or_else(|_| table.get_function_id(ident, false, errors))?;
                // check it is a function or and operator
                if table.is_function(id) {
                    Ok(())
                } else {
                    bad!(errors, @ident.span() => ErrorKind::expected_constant())
                }
            }
            stream::Expr::UnOp(UnOp { expr, .. }) => expr.check_is_constant(table, errors),
            stream::Expr::BinOp(BinOp { lft, rgt, .. }) => {
                lft.check_is_constant(table, errors)?;
                rgt.check_is_constant(table, errors)
            }
            stream::Expr::IfThenElse(IfThenElse { cnd, thn, els, .. }) => {
                cnd.check_is_constant(table, errors)?;
                thn.check_is_constant(table, errors)?;
                els.check_is_constant(table, errors)
            }
            stream::Expr::Application(Application { fun, inputs, .. }) => {
                fun.check_is_constant(table, errors)?;
                inputs
                    .iter()
                    .map(|expression| expression.check_is_constant(table, errors))
                    .collect::<TRes<_>>()
            }
            stream::Expr::Structure(Structure { fields, .. }) => fields
                .iter()
                .map(|(_, expression)| expression.check_is_constant(table, errors))
                .collect::<TRes<_>>(),
            stream::Expr::Array(Array { elements, .. })
            | stream::Expr::Tuple(Tuple { elements, .. }) => elements
                .iter()
                .map(|expression| expression.check_is_constant(table, errors))
                .collect::<TRes<_>>(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust reactive expression kind AST.
pub enum ReactExpr {
    /// Stream expression.
    Expr(Expr),
    /// Pattern matching event.
    When(When),
}
mk_new! { impl ReactExpr =>
    Expr: expr(arg: Expr = arg)
    When: when_match(arg: When = arg)
}
impl ReactExpr {
    pub fn loc(&self) -> Loc {
        match self {
            Self::Expr(e) => e.loc(),
            Self::When(w) => w.loc(),
        }
    }
    pub fn check_is_constant(&self, table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        match &self {
            stream::ReactExpr::Expr(expr) => expr.check_is_constant(table, errors),
            stream::ReactExpr::When(whn) => {
                bad!(errors, @whn.loc() => ErrorKind::expected_constant())
            }
        }
    }
}
