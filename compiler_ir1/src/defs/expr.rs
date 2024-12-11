//! [Expr] module.

prelude! {
    graph::Label,
}

#[derive(Debug, PartialEq, Clone)]
/// expression kind.
pub enum Kind<E> {
    /// Constant expression.
    Constant {
        /// The constant.
        constant: Constant,
    },
    /// Identifier expression.
    Identifier {
        /// Element identifier.
        id: usize,
    },
    /// UnOp expression.
    UnOp {
        /// The unary operator.
        op: UOp,
        /// The input expression.
        expr: Box<E>,
    },
    /// BinOp expression.
    BinOp {
        /// The unary operator.
        op: BOp,
        /// The left expression.
        lft: Box<E>,
        /// The right expression.
        rgt: Box<E>,
    },
    /// IfThenElse expression.
    IfThenElse {
        /// Condition.
        cnd: Box<E>,
        /// `then` branch.
        thn: Box<E>,
        /// `else` branch.
        els: Box<E>,
    },
    /// Application expression.
    Application {
        /// The expression applied.
        fun: Box<E>,
        /// The inputs to the expression.
        inputs: Vec<E>,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<usize>,
        /// The expression abstracted.
        expr: Box<E>,
    },
    /// Structure expression.
    Structure {
        /// The structure id.
        id: usize,
        /// The fields associated with their expressions.
        fields: Vec<(usize, E)>,
    },
    /// Enumeration expression.
    Enumeration {
        /// The enumeration id.
        enum_id: usize,
        /// The element id.
        elem_id: usize,
    },
    /// Array expression.
    Array {
        /// The elements inside the array.
        elements: Vec<E>,
    },
    /// Tuple expression.
    Tuple {
        /// The elements.
        elements: Vec<E>,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expr: Box<E>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<E>, Vec<Stmt<E>>, E)>,
    },
    /// Field access expression.
    FieldAccess {
        /// The structure expression.
        expr: Box<E>,
        /// The field to access.
        field: Ident, // can not be a usize because we don't know the structure type
    },
    /// Tuple element access expression.
    TupleElementAccess {
        /// The tuple expression.
        expr: Box<E>,
        /// The element to access.
        element_number: usize,
    },
    /// Array map operator expression.
    Map {
        /// The array expression.
        expr: Box<E>,
        /// The function expression.
        fun: Box<E>,
    },
    /// Array fold operator expression.
    Fold {
        /// The array expression.
        array: Box<E>,
        /// The initialization expression.
        init: Box<E>,
        /// The function expression.
        fun: Box<E>,
    },
    /// Array sort operator expression.
    Sort {
        /// The array expression.
        expr: Box<E>,
        /// The function expression.
        fun: Box<E>,
    },
    /// Arrays zip operator expression.
    Zip {
        /// The array expressions.
        arrays: Vec<E>,
    },
}

mk_new! { impl{E} Kind<E> =>
    Constant: constant { constant: Constant }
    Identifier: ident { id : usize }
    UnOp: unop {
        op: UOp,
        expr: E = expr.into(),
    }
    BinOp: binop {
        op: BOp,
        lft: E = lft.into(),
        rgt: E = rgt.into(),
    }
    IfThenElse: if_then_else {
        cnd: E = cnd.into(),
        thn: E = thn.into(),
        els: E = els.into(),
    }
    Application: app {
        fun: E = fun.into(),
        inputs: Vec<E>,
    }
    Abstraction: lambda {
        inputs: Vec<usize>,
        expr: E = expr.into(),
    }
    Structure: structure {
        id: usize,
        fields: Vec<(usize, E)>,
    }
    Enumeration: enumeration {
        enum_id: usize,
        elem_id: usize,
    }
    Array: array { elements: Vec<E> }
    Tuple: tuple { elements: Vec<E> }
    Match: match_expr {
        expr: E = expr.into(),
        arms: Vec<(ir1::Pattern, Option<E>, Vec<ir1::Stmt<E>>, E)>,
    }
    FieldAccess: field_access {
        expr: E = expr.into(),
        field: impl Into<Ident> = field.into(),
    }
    TupleElementAccess: tuple_access {
        expr: E = expr.into(),
        element_number: usize,
    }
    Map: map {
        expr: E = expr.into(),
        fun: E = fun.into(),
    }
    Fold: fold {
        array: E = array.into(),
        init: E = init.into(),
        fun: E = fun.into(),
    }
    Sort: sort{
        expr: E = expr.into(),
        fun: E = fun.into(),
    }
    Zip: zip { arrays: Vec<E> }
}

/// expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    /// Expression kind.
    pub kind: ir1::expr::Kind<Self>,
    /// Expression type.
    pub typing: Option<Typ>,
    /// Expression location.
    pub loc: Loc,
    /// Expression dependencies.
    pub dependencies: ir1::Dependencies,
}

impl Expr {
    /// Get expression's type.
    pub fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
    /// Get expression's mutable type.
    pub fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
    /// Get expression's dependencies.
    pub fn get_dependencies(&self) -> &Vec<(usize, Label)> {
        self.dependencies
            .get()
            .expect("there should be dependencies")
    }
}

/// Constructs expression.
///
/// Typing, location and dependencies are empty.
pub fn init(kind: Kind<Expr>) -> Expr {
    Expr {
        kind,
        typing: None,
        loc: Loc::builtin(),
        dependencies: ir1::Dependencies::new(),
    }
}

impl<E> Kind<E> {
    /// Propagate a predicate over the expression tree.
    pub fn propagate_predicate<F1, F2>(&self, expr_pred: F1, stmt_pred: F2) -> bool
    where
        F1: Fn(&E) -> bool,
        F2: Fn(&ir1::Stmt<E>) -> bool,
    {
        match self {
            Kind::Constant { .. }
            | Kind::Identifier { .. }
            | Kind::Abstraction { .. }
            | Kind::Enumeration { .. } => true,
            Kind::UnOp { expr, .. } => expr_pred(expr),
            Kind::BinOp { lft, rgt, .. } => expr_pred(lft) && expr_pred(rgt),
            Kind::IfThenElse { cnd, thn, els } => {
                expr_pred(cnd) && expr_pred(thn) && expr_pred(els)
            }
            Kind::Application { fun, inputs } => {
                expr_pred(fun) && inputs.iter().all(|expression| expr_pred(expression))
            }
            Kind::Structure { fields, .. } => {
                fields.iter().all(|(_, expression)| expr_pred(expression))
            }
            Kind::Array { elements } | Kind::Tuple { elements } => {
                elements.iter().all(|expression| expr_pred(expression))
            }
            Kind::Match { expr, arms } => {
                expr_pred(expr)
                    && arms.iter().all(|(_, option, body, expr)| {
                        body.iter().all(|statement| stmt_pred(statement))
                            && option.as_ref().map_or(true, |expr| expr_pred(expr))
                            && expr_pred(expr)
                    })
            }
            Kind::FieldAccess { expr, .. } => expr_pred(expr),
            Kind::TupleElementAccess { expr, .. } => expr_pred(expr),
            Kind::Map { expr, fun } => expr_pred(expr) && expr_pred(fun),
            Kind::Fold { array, init, fun } => {
                expr_pred(array) && expr_pred(init) && expr_pred(fun)
            }
            Kind::Sort { expr, fun } => expr_pred(expr) && expr_pred(fun),
            Kind::Zip { arrays } => arrays.iter().all(|expr| expr_pred(expr)),
        }
    }
}
