prelude! {
    ast::interface::FlowKind,
    hir::{
        flow, IdentifierCreator,
        interface::{
            FlowDeclaration, FlowInstantiation, FlowStatement, Interface, Service,
        },
    },
}

impl Interface {
    pub fn normal_form(&mut self, symbol_table: &mut SymbolTable) {
        self.services
            .iter_mut()
            .for_each(|service| service.normal_form(symbol_table))
    }
}

impl Service {
    pub fn normal_form(&mut self, symbol_table: &mut SymbolTable) {
        symbol_table.local();
        let mut identifier_creator = IdentifierCreator::from(self.get_flows_names(symbol_table));
        let statements = std::mem::take(&mut self.statements);
        debug_assert!(self.statements.is_empty());
        statements.into_values().for_each(|flow_statement| {
            let statements = flow_statement.normal_form(&mut identifier_creator, symbol_table);
            for statement in statements {
                let _unique = self
                    .statements
                    .insert(symbol_table.get_fresh_id(), statement);
                debug_assert!(_unique.is_none())
            }
        });
        symbol_table.global()
    }
}

impl FlowStatement {
    /// Change HIR flow statement into a normal form.
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
        mut self,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
    ) -> Vec<FlowStatement> {
        let mut new_statements = match &mut self {
            FlowStatement::Declaration(FlowDeclaration {
                ref mut flow_expression,
                ..
            })
            | FlowStatement::Instantiation(FlowInstantiation {
                ref mut flow_expression,
                ..
            }) => flow_expression.normal_form(identifier_creator, symbol_table),
        };
        new_statements.push(self);
        new_statements
    }
}

impl flow::Expr {
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
    ) -> Vec<FlowStatement> {
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
    ) -> Vec<FlowStatement> {
        match self.kind {
            flow::Kind::Ident { .. } => vec![],
            _ => {
                let mut statements = self.normal_form(identifier_creator, symbol_table);

                // create fresh identifier for the new statement
                let fresh_name = identifier_creator.fresh_identifier("", "x");
                let typing = self.get_type().unwrap();
                let kind = match typing {
                    Typ::Signal { .. } => FlowKind::Signal(Default::default()),
                    Typ::Event { .. } => FlowKind::Event(Default::default()),
                    Typ::Tuple { .. } => panic!("tuple of flows can not be converted into flow"),
                    _ => unreachable!(),
                };
                let fresh_id = symbol_table.insert_fresh_flow(fresh_name, kind, typing.clone());

                // create statement for the expression
                let new_statement = FlowStatement::Declaration(FlowDeclaration {
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
