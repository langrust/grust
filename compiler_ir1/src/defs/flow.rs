//! [Interface] module.

prelude! {}

#[derive(Debug, PartialEq, Clone)]
/// Flow expression kinds.
pub enum Kind {
    /// Flow identifier call.
    Ident {
        /// The identifier of the flow to call.
        id: usize,
    },
    /// GReact `sample` operator.
    Sample {
        /// Input expression.
        expr: Box<Expr>,
        /// Sampling period in milliseconds.
        period_ms: u64,
    },
    /// GReact `scan` operator.
    Scan {
        /// Input expression.
        expr: Box<Expr>,
        /// Scanning period in milliseconds.
        period_ms: u64,
    },
    /// GReact `timeout` operator.
    Timeout {
        /// Input expression.
        expr: Box<Expr>,
        /// Deadline in milliseconds.
        deadline: u64,
    },
    /// GReact `throttle` operator.
    Throttle {
        /// Input expression.
        expr: Box<Expr>,
        /// Variation that will update the signal.
        delta: Constant,
    },
    /// GReact `on_change` operator.
    OnChange {
        /// Input expression.
        expr: Box<Expr>,
    },
    /// GReact `persist` operator.
    Persist {
        /// Input expression.
        expr: Box<Expr>,
    },
    /// GReact `merge` operator.
    Merge {
        /// Input expressions.
        expr_1: Box<Expr>,
        expr_2: Box<Expr>,
    },
    /// GReact `time` operator.
    Time { loc: Loc },
    /// Component call.
    ComponentCall {
        /// Identifier to the component to call.
        component_id: usize,
        /// Input expressions.
        inputs: Vec<(usize, Expr)>,
    },
    /// Function call.
    FunctionCall {
        /// Identifier to the function to call.
        function_id: usize,
        /// Input expressions.
        inputs: Vec<(usize, Expr)>,
    },
}

#[derive(Debug, PartialEq, Clone)]
/// Flow expression [ir1].
pub struct Expr {
    /// Flow expression's kind.
    pub kind: Kind,
    /// Flow expression type.
    pub typ: Option<Typ>,
    /// Flow expression location.
    pub loc: Loc,
}
impl HasLoc for Expr {
    fn loc(&self) -> Loc {
        self.loc
    }
}
impl Expr {
    pub fn get_type(&self) -> Option<&Typ> {
        self.typ.as_ref()
    }

    pub fn get_dependencies(&self) -> Vec<usize> {
        match &self.kind {
            Kind::Ident { id } => vec![*id],
            Kind::Sample { expr, .. }
            | Kind::Scan { expr, .. }
            | Kind::Timeout { expr, .. }
            | Kind::Throttle { expr, .. }
            | Kind::OnChange { expr }
            | Kind::Persist { expr } => expr.get_dependencies(),
            Kind::Merge { expr_1, expr_2 } => {
                let mut dependencies = expr_1.get_dependencies();
                dependencies.extend(expr_2.get_dependencies());
                dependencies
            }
            Kind::ComponentCall { inputs, .. } | Kind::FunctionCall { inputs, .. } => inputs
                .iter()
                .flat_map(|(_, expr)| expr.get_dependencies())
                .collect(),
            Kind::Time { .. } => vec![],
        }
    }

    pub fn is_normal(&self) -> bool {
        match &self.kind {
            flow::Kind::Ident { .. } | Kind::Time { .. } => true,
            flow::Kind::Sample { expr, .. }
            | flow::Kind::Scan { expr, .. }
            | flow::Kind::Timeout { expr, .. }
            | flow::Kind::Throttle { expr, .. }
            | flow::Kind::OnChange { expr }
            | flow::Kind::Persist { expr } => expr.is_ident(),
            flow::Kind::Merge { expr_1, expr_2 } => expr_1.is_ident() && expr_2.is_ident(),
            flow::Kind::ComponentCall { inputs, .. } | Kind::FunctionCall { inputs, .. } => {
                inputs.iter().all(|(_, expr)| expr.is_ident())
            }
        }
    }

    fn is_ident(&self) -> bool {
        if let flow::Kind::Ident { .. } = &self.kind {
            true
        } else {
            false
        }
    }
    /// Change [ir1] flow expression into a normal form.
    ///
    /// The normal form of an expression is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// The normal form of a flow expression is as follows:
    /// - flow expressions others than identifiers are root expression
    /// - then, arguments are only identifiers
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn normal_form(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        ctx: &mut Ctx,
    ) -> Vec<interface::FlowStatement> {
        match &mut self.kind {
            flow::Kind::Ident { .. } | Kind::Time { .. } => vec![],
            flow::Kind::Sample { expr, .. } => expr.into_flow_call(identifier_creator, ctx),
            flow::Kind::Scan { expr, .. } => expr.into_flow_call(identifier_creator, ctx),
            flow::Kind::Timeout { expr, .. } => expr.into_flow_call(identifier_creator, ctx),
            flow::Kind::Throttle { expr, .. } => expr.into_flow_call(identifier_creator, ctx),
            flow::Kind::OnChange { expr } => expr.into_flow_call(identifier_creator, ctx),
            flow::Kind::Persist { expr } => expr.into_flow_call(identifier_creator, ctx),
            flow::Kind::Merge { expr_1, expr_2 } => {
                let mut stmts = expr_1.into_flow_call(identifier_creator, ctx);
                stmts.extend(expr_2.into_flow_call(identifier_creator, ctx));
                stmts
            }
            flow::Kind::ComponentCall { inputs, .. } | Kind::FunctionCall { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|(_, expr)| expr.into_flow_call(identifier_creator, ctx))
                .collect(),
        }
    }

    fn into_flow_call(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        ctx: &mut Ctx,
    ) -> Vec<interface::FlowStatement> {
        match self.kind {
            flow::Kind::Ident { .. } => vec![],
            _ => {
                let mut statements = self.normal_form(identifier_creator, ctx);

                // create fresh identifier for the new statement
                let fresh_name = identifier_creator.fresh_identifier(self.loc(), "", "x");
                let typ = self.get_type().unwrap();
                let kind = match typ {
                    Typ::Signal { .. } => ir0::interface::FlowKind::Signal(Default::default()),
                    Typ::Event { .. } => ir0::interface::FlowKind::Event(Default::default()),
                    Typ::Tuple { .. } => panic!("tuple of flows can not be converted into flow"),
                    _ => noErrorDesc!(),
                };
                let fresh_id = ctx.insert_fresh_flow(fresh_name, kind, typ.clone());

                // create statement for the expression
                let new_statement =
                    interface::FlowStatement::Declaration(interface::FlowDeclaration {
                        let_token: Default::default(),
                        pattern: ir1::stmt::Pattern {
                            kind: ir1::stmt::Kind::Identifier { id: fresh_id },
                            typ: Some(typ.clone()),
                            loc: self.loc.clone(),
                        },
                        eq_token: Default::default(),
                        expr: self.clone(),
                        semi_token: Default::default(),
                    });
                statements.push(new_statement);

                // change current expression be an identifier to the statement of the expression
                self.kind = flow::Kind::Ident { id: fresh_id };
                // self.dependencies = Dependencies::from(vec![(fresh_id, Label::Weight(0))]);

                // return new additional statements
                statements
            }
        }
    }
}
