//! HIR [Interface](crate::hir::interface::Interface) module.

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
        flow_expression: Box<Expr>,
        /// Sampling period in milliseconds.
        period_ms: u64,
    },
    /// GReact `scan` operator.
    Scan {
        /// Input expression.
        flow_expression: Box<Expr>,
        /// Scaning period in milliseconds.
        period_ms: u64,
    },
    /// GReact `timeout` operator.
    Timeout {
        /// Input expression.
        flow_expression: Box<Expr>,
        /// Dealine in milliseconds.
        deadline: u64,
    },
    /// GReact `throttle` operator.
    Throttle {
        /// Input expression.
        flow_expression: Box<Expr>,
        /// Variation that will update the signal.
        delta: Constant,
    },
    /// GReact `on_change` operator.
    OnChange {
        /// Input expression.
        flow_expression: Box<Expr>,
    },
    /// GReact `merge` operator.
    Merge {
        /// Input expressions.
        flow_expression_1: Box<Expr>,
        flow_expression_2: Box<Expr>,
    },
    /// Component call.
    ComponentCall {
        /// Identifier to the component to call.
        component_id: usize,
        /// Input expressions.
        inputs: Vec<(usize, Expr)>,
    },
}

#[derive(Debug, PartialEq, Clone)]
/// Flow expression HIR.
pub struct Expr {
    /// Flow expression's kind.
    pub kind: Kind,
    /// Flow expression type.
    pub typing: Option<Typ>,
    /// Flow expression location.
    pub location: Location,
}
impl Expr {
    pub fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    pub fn get_dependencies(&self) -> Vec<usize> {
        match &self.kind {
            Kind::Ident { id } => vec![*id],
            Kind::Sample {
                flow_expression, ..
            }
            | Kind::Scan {
                flow_expression, ..
            }
            | Kind::Timeout {
                flow_expression, ..
            }
            | Kind::Throttle {
                flow_expression, ..
            }
            | Kind::OnChange { flow_expression } => flow_expression.get_dependencies(),
            Kind::Merge {
                flow_expression_1,
                flow_expression_2,
            } => {
                let mut dependencies = flow_expression_1.get_dependencies();
                dependencies.extend(flow_expression_2.get_dependencies());
                dependencies
            }
            Kind::ComponentCall { inputs, .. } => inputs
                .iter()
                .flat_map(|(_, flow_expression)| flow_expression.get_dependencies())
                .collect(),
        }
    }

    pub fn is_normal(&self) -> bool {
        match &self.kind {
            flow::Kind::Ident { .. } => true,
            flow::Kind::Sample {
                flow_expression, ..
            }
            | flow::Kind::Scan {
                flow_expression, ..
            }
            | flow::Kind::Timeout {
                flow_expression, ..
            }
            | flow::Kind::Throttle {
                flow_expression, ..
            }
            | flow::Kind::OnChange { flow_expression } => flow_expression.is_ident(),
            flow::Kind::Merge {
                flow_expression_1,
                flow_expression_2,
            } => flow_expression_1.is_ident() && flow_expression_2.is_ident(),
            flow::Kind::ComponentCall { inputs, .. } => inputs
                .iter()
                .all(|(_, flow_expression)| flow_expression.is_ident()),
        }
    }

    fn is_ident(&self) -> bool {
        if let flow::Kind::Ident { .. } = &self.kind {
            true
        } else {
            false
        }
    }
    /// Change HIR flow expression into a normal form.
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
        symbol_table: &mut SymbolTable,
    ) -> Vec<interface::FlowStatement> {
        match &mut self.kind {
            flow::Kind::Ident { .. } => vec![],
            flow::Kind::Sample {
                flow_expression, ..
            } => flow_expression.into_flow_call(identifier_creator, symbol_table),
            flow::Kind::Scan {
                flow_expression, ..
            } => flow_expression.into_flow_call(identifier_creator, symbol_table),
            flow::Kind::Timeout {
                flow_expression, ..
            } => flow_expression.into_flow_call(identifier_creator, symbol_table),
            flow::Kind::Throttle {
                flow_expression, ..
            } => flow_expression.into_flow_call(identifier_creator, symbol_table),
            flow::Kind::OnChange { flow_expression } => {
                flow_expression.into_flow_call(identifier_creator, symbol_table)
            }
            flow::Kind::Merge {
                flow_expression_1,
                flow_expression_2,
            } => {
                let mut stmts = flow_expression_1.into_flow_call(identifier_creator, symbol_table);
                stmts.extend(flow_expression_2.into_flow_call(identifier_creator, symbol_table));
                stmts
            }
            flow::Kind::ComponentCall { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|(_, flow_expression)| {
                    flow_expression.into_flow_call(identifier_creator, symbol_table)
                })
                .collect(),
        }
    }

    fn into_flow_call(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<interface::FlowStatement> {
        match self.kind {
            flow::Kind::Ident { .. } => vec![],
            _ => {
                let mut statements = self.normal_form(identifier_creator, symbol_table);

                // create fresh identifier for the new statement
                let fresh_name = identifier_creator.fresh_identifier("", "x");
                let typing = self.get_type().unwrap();
                let kind = match typing {
                    Typ::Signal { .. } => ast::interface::FlowKind::Signal(Default::default()),
                    Typ::Event { .. } => ast::interface::FlowKind::Event(Default::default()),
                    Typ::Tuple { .. } => panic!("tuple of flows can not be converted into flow"),
                    _ => unreachable!(),
                };
                let fresh_id = symbol_table.insert_fresh_flow(fresh_name, kind, typing.clone());

                // create statement for the expression
                let new_statement =
                    interface::FlowStatement::Declaration(interface::FlowDeclaration {
                        let_token: Default::default(),
                        pattern: hir::stmt::Pattern {
                            kind: hir::stmt::Kind::Identifier { id: fresh_id },
                            typing: Some(typing.clone()),
                            location: self.location.clone(),
                        },
                        eq_token: Default::default(),
                        flow_expression: self.clone(),
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
