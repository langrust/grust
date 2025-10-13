prelude! {
    equation::EventPattern,
    expr::*,
}

/// Buffered identifier.
#[derive(Debug, PartialEq, Clone)]
pub struct Last {
    /// Location.
    pub loc: Loc,
    /// Identifier.
    pub ident: Ident,
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
pub struct WhenExpr {
    pub when_token: keyword::when,
    /// The optional init arm.
    pub init: Option<InitArmWhen>,
    /// The different event cases.
    pub arms: Vec<EventArmWhen>,
}
impl WhenExpr {
    pub fn loc(&self) -> Loc {
        self.when_token.span.into()
    }
}
mk_new! { impl WhenExpr =>
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
    /// Lambda expression with inputs types.
    Lambda(Lambda<Self>),
    /// Structure expression.
    Structure(Structure<Self>),
    /// Tuple expression.
    Tuple(Tuple<Self>),
    /// Enumeration expression.
    Enumeration(Enumeration<Self>),
    /// Array expression.
    Array(Array<Self>),
    /// Pattern matching expression.
    MatchExpr(MatchExpr<Self>),
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
    Lambda: type_lambda(arg: Lambda<Self> = arg)
    Structure: structure(arg: Structure<Self> = arg)
    Tuple: tuple(arg: Tuple<Self> = arg)
    Enumeration: enumeration(arg: Enumeration<Self> = arg)
    Array: array(arg: Array<Self> = arg)
    MatchExpr: match_expr(arg: MatchExpr<Self> = arg)
    FieldAccess: field_access(arg: FieldAccess<Self> = arg)
    TupleElementAccess: tuple_access(arg: TupleElementAccess<Self> = arg)
    ArrayAccess: array_access(arg: ArrayAccess<Self> = arg)
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
            Lambda(abs) => abs.loc(),
            Structure(s) => s.loc(),
            Tuple(t) => t.loc(),
            Enumeration(e) => e.loc(),
            Array(a) => a.loc(),
            MatchExpr(m) => m.loc(),
            FieldAccess(fa) => fa.loc(),
            TupleElementAccess(ta) => ta.loc(),
            ArrayAccess(aa) => aa.loc(),
            Map(m) => m.loc(),
            Fold(f) => f.loc(),
            Sort(s) => s.loc(),
            Zip(z) => z.loc(),
            Last(l) => l.loc(),
            Emit(e) => e.loc(),
        }
    }
}

impl TryFrom<ir0::Expr> for Expr {
    type Error = Error;

    fn try_from(value: ir0::Expr) -> Res<Self> {
        match value {
            ir0::Expr::Constant(constant) => Ok(Self::cst(constant)),
            ir0::Expr::Identifier(ident) => Ok(Self::ident(ident)),
            ir0::Expr::UnOp(un_op) => {
                let expr: Expr = (*un_op.expr).try_into()?;
                Ok(Self::unop(UnOp::new(un_op.op, un_op.op_loc, expr)))
            }
            ir0::Expr::BinOp(bin_op) => {
                let lft: Expr = (*bin_op.lft).try_into()?;
                let rgt: Expr = (*bin_op.rgt).try_into()?;
                Ok(Self::binop(BinOp::new(bin_op.op, bin_op.op_loc, lft, rgt)))
            }
            ir0::Expr::IfThenElse(if_then_else) => {
                let cnd: Expr = (*if_then_else.cnd).try_into()?;
                let thn: Expr = (*if_then_else.thn).try_into()?;
                let els: Expr = (*if_then_else.els).try_into()?;
                Ok(Self::ite(IfThenElse::new(if_then_else.loc, cnd, thn, els)))
            }
            ir0::Expr::Application(application) => {
                let fun: Expr = (*application.fun).try_into()?;
                let inputs = application
                    .inputs
                    .into_iter()
                    .map(|expr| -> Res<_> { expr.try_into() })
                    .collect::<Res<_>>()?;
                Ok(Self::app(Application::new(application.loc, fun, inputs)))
            }
            ir0::Expr::Lambda(lambda) => Ok(Self::type_lambda(Lambda::new(
                lambda.loc,
                lambda.inputs,
                *lambda.expr,
            ))),
            ir0::Expr::Structure(structure) => {
                let fields = structure
                    .fields
                    .into_iter()
                    .map(|(ident, expr)| -> Res<_> { Ok((ident, expr.try_into()?)) })
                    .collect::<Res<_>>()?;
                Ok(Self::structure(Structure::new(
                    structure.loc,
                    structure.name,
                    fields,
                )))
            }
            ir0::Expr::Tuple(tuple) => {
                let elements = tuple
                    .elements
                    .into_iter()
                    .map(|expr| -> Res<_> { expr.try_into() })
                    .collect::<Res<_>>()?;
                Ok(Self::tuple(Tuple::new(tuple.loc, elements)))
            }
            ir0::Expr::Enumeration(enumeration) => Ok(Self::enumeration(Enumeration::new(
                enumeration.loc,
                enumeration.enum_name,
                enumeration.elem_name,
            ))),
            ir0::Expr::Array(array) => {
                let elements = array
                    .elements
                    .into_iter()
                    .map(|expr| -> Res<_> { expr.try_into() })
                    .collect::<Res<_>>()?;
                Ok(Self::array(Array::new(array.loc, elements)))
            }
            ir0::Expr::MatchExpr(match_expr) => {
                let expr: Expr = (*match_expr.expr).try_into()?;
                let arms = match_expr
                    .arms
                    .into_iter()
                    .map(|arm| -> Res<_> {
                        let Arm {
                            pattern,
                            guard,
                            expr,
                        } = arm;
                        Ok(Arm::new_with_guard(
                            pattern,
                            expr.try_into()?,
                            guard.map(|expr| expr.try_into()).transpose()?,
                        ))
                    })
                    .collect::<Res<_>>()?;
                Ok(Self::match_expr(MatchExpr::new(match_expr.loc, expr, arms)))
            }
            ir0::Expr::FieldAccess(field_access) => {
                let expr: Expr = (*field_access.expr).try_into()?;
                Ok(Self::field_access(FieldAccess::new(
                    field_access.loc,
                    expr,
                    field_access.field,
                )))
            }
            ir0::Expr::TupleElementAccess(tuple_element_access) => {
                let expr: Expr = (*tuple_element_access.expr).try_into()?;
                Ok(Self::tuple_access(TupleElementAccess::new(
                    tuple_element_access.loc,
                    expr,
                    tuple_element_access.element_number,
                )))
            }
            ir0::Expr::ArrayAccess(array_access) => {
                let expr: Expr = (*array_access.expr).try_into()?;
                Ok(Self::array_access(ArrayAccess::new(
                    array_access.loc,
                    expr,
                    array_access.index,
                )))
            }
            ir0::Expr::Map(map) => {
                let expr: Expr = (*map.expr).try_into()?;
                let fun: Expr = (*map.fun).try_into()?;
                Ok(Self::map(Map::new(map.loc, expr, fun)))
            }
            ir0::Expr::Fold(fold) => {
                let array: Expr = (*fold.array).try_into()?;
                let init: Expr = (*fold.init).try_into()?;
                let fun: Expr = (*fold.fun).try_into()?;
                Ok(Self::fold(Fold::new(fold.loc, array, init, fun)))
            }
            ir0::Expr::Sort(sort) => {
                let expr: Expr = (*sort.expr).try_into()?;
                let fun: Expr = (*sort.fun).try_into()?;
                Ok(Self::sort(Sort::new(sort.loc, expr, fun)))
            }
            ir0::Expr::Zip(zip) => {
                let arrays = zip
                    .arrays
                    .into_iter()
                    .map(|expr| -> Res<_> { expr.try_into() })
                    .collect::<Res<_>>()?;
                Ok(Self::zip(Zip::new(zip.loc, arrays)))
            }
        }
    }
}

impl Expr {
    pub fn check_is_constant(&self, table: &Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        match self {
            // Constant by default
            stream::Expr::Constant { .. } | stream::Expr::Enumeration { .. } => Ok(()),
            // Not constant by default
            stream::Expr::Lambda { .. }
            | stream::Expr::MatchExpr { .. }
            | stream::Expr::Emit { .. }
            | stream::Expr::FieldAccess { .. }
            | stream::Expr::TupleElementAccess { .. }
            | stream::Expr::ArrayAccess { .. }
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
                let id = table.get_ident(ident, false, true, errors)?;
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
                    .collect_res()
            }
            stream::Expr::Structure(Structure { fields, .. }) => fields
                .iter()
                .map(|(_, expression)| expression.check_is_constant(table, errors))
                .collect_res(),
            stream::Expr::Array(Array { elements, .. })
            | stream::Expr::Tuple(Tuple { elements, .. }) => elements
                .iter()
                .map(|expression| expression.check_is_constant(table, errors))
                .collect_res(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust reactive expression kind AST.
pub enum ReactExpr {
    /// Stream expression.
    Expr(Expr),
    /// Pattern matching event.
    WhenExpr(WhenExpr),
}
mk_new! { impl ReactExpr =>
    Expr: expr(arg: Expr = arg)
    WhenExpr: when_expr(arg: WhenExpr = arg)
}
impl ReactExpr {
    pub fn loc(&self) -> Loc {
        match self {
            Self::Expr(e) => e.loc(),
            Self::WhenExpr(w) => w.loc(),
        }
    }
    pub fn check_is_constant(&self, table: &mut Ctx, errors: &mut Vec<Error>) -> TRes<()> {
        match &self {
            stream::ReactExpr::Expr(expr) => expr.check_is_constant(table, errors),
            stream::ReactExpr::WhenExpr(whn) => {
                bad!(errors, @whn.loc() => ErrorKind::expected_constant())
            }
        }
    }
}
