prelude! {
    equation::EventPattern,
    expr::*,
}

/// Buffered signal.
#[derive(Debug, PartialEq, Clone)]
pub struct Last {
    /// Signal identifier.
    pub ident: Ident,
    /// The initialization constant.
    pub constant: Option<Box<Expr>>,
}
mk_new! { impl Last =>
    new {
        ident: Ident,
        constant: Option<Expr> = constant.map(Expr::into),
    }
}

/// Pattern matching for event expression.
#[derive(Debug, PartialEq, Clone)]
pub struct When {
    /// The pattern receiving the value of the event.
    pub pattern: EventPattern,
    /// The optional guard.
    pub guard: Option<Box<Expr>>,
    pub then_token: keyword::then,
    /// Action triggered by event.
    pub expr: Box<Expr>,
}
mk_new! { impl When =>
    new {
        pattern: EventPattern,
        guard: Option<Expr> = guard.map(Expr::into),
        then_token: keyword::then,
        expr: impl Into<Box<Expr >> = expr.into(),
    }
}

/// Emit event expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Emit {
    pub emit_token: keyword::emit,
    /// The expression to emit.
    pub expr: Box<Expr>,
}
mk_new! { impl Emit =>
    new {
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
    Identifier(String),
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
    Identifier: ident(arg : impl Into<String> = arg.into())
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

impl Expr {
    pub fn check_is_constant(&self, table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
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
                let error = Error::ExpectConstant {
                    loc: Location::default(),
                };
                errors.push(error);
                Err(TerminationError)
            }
            // It depends
            stream::Expr::Identifier(id) => {
                // check id exists
                let id = table
                    .get_identifier_id(&id, false, Location::default(), &mut vec![])
                    .or_else(|_| table.get_function_id(&id, false, Location::default(), errors))?;
                // check it is a function or and operator
                if table.is_function(id) {
                    Ok(())
                } else {
                    let error = Error::ExpectConstant {
                        loc: Location::default(),
                    };
                    errors.push(error);
                    Err(TerminationError)
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
            stream::Expr::Application(Application { fun, inputs }) => {
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
            stream::Expr::Array(Array { elements }) | stream::Expr::Tuple(Tuple { elements }) => {
                elements
                    .iter()
                    .map(|expression| expression.check_is_constant(table, errors))
                    .collect::<TRes<_>>()
            }
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
    pub fn check_is_constant(&self, table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        match &self {
            stream::ReactExpr::Expr(expr) => expr.check_is_constant(table, errors),
            stream::ReactExpr::When { .. } => {
                let error = Error::ExpectConstant {
                    loc: Location::default(),
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }
}
