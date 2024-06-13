prelude! {
    ast::interface::FlowKind,
    hir::{
        flow, IdentifierCreator, Pattern,
        interface::{FlowDeclaration, FlowInstantiation, FlowStatement, Interface},
    },
}

impl Interface {
    pub fn normal_form(&mut self, symbol_table: &mut SymbolTable) {
        let mut identifier_creator = IdentifierCreator::from(self.get_flows_names(symbol_table));
        let statements = std::mem::take(&mut self.statements);
        self.statements = statements
            .into_iter()
            .flat_map(|flow_statement| {
                flow_statement.normal_form(&mut identifier_creator, symbol_table)
            })
            .collect();
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
            _ => vec![],
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
                let fresh_name = identifier_creator.fresh_identifier("flow_expression");
                let typing = self.get_type().unwrap();
                let kind = match typing {
                    Typ::Signal(_) => FlowKind::Signal(Default::default()),
                    Typ::Event(_) => FlowKind::Event(Default::default()),
                    Typ::Tuple(_) => panic!("tuple of flows can not be converted into flow"),
                    _ => unreachable!(),
                };
                let fresh_id = symbol_table.insert_fresh_flow(fresh_name, kind, typing.clone());

                // create statement for the expression
                let new_statement = FlowStatement::Declaration(FlowDeclaration {
                    let_token: Default::default(),
                    pattern: Pattern {
                        kind: hir::pattern::Kind::Identifier { id: fresh_id },
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
