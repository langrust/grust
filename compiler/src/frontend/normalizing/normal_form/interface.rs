use crate::hir::flow_expression::{FlowExpression, FlowExpressionKind};
use crate::hir::flow_statement::{
    FlowDeclaration, FlowExport, FlowImport, FlowInstanciation, FlowStatement,
};
use crate::hir::identifier_creator::IdentifierCreator;
use crate::hir::pattern::{Pattern, PatternKind};
use crate::symbol_table::SymbolTable;

pub struct Interface<'a>(pub &'a Vec<FlowStatement>);
impl<'a> Interface<'a> {
    pub fn get_flows_names(self, symbol_table: &SymbolTable) -> Vec<String> {
        self.0
            .iter()
            .flat_map(|statement| match statement {
                FlowStatement::Declaration(FlowDeclaration { pattern, .. })
                | FlowStatement::Instanciation(FlowInstanciation { pattern, .. }) => pattern
                    .identifiers()
                    .into_iter()
                    .map(|id| symbol_table.get_name(id).clone())
                    .collect(),
                FlowStatement::Import(FlowImport { id, .. })
                | FlowStatement::Export(FlowExport { id, .. }) => {
                    vec![symbol_table.get_name(*id).clone()]
                }
            })
            .collect()
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
            | FlowStatement::Instanciation(FlowInstanciation {
                ref mut flow_expression,
                ..
            }) => flow_expression.into_flow_call(identifier_creator, symbol_table),
            _ => vec![],
        };
        new_statements.push(self);
        new_statements
    }
}

impl FlowExpression {
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
            FlowExpressionKind::Ident { .. } => vec![],
            FlowExpressionKind::Sample {
                flow_expression, ..
            } => flow_expression.into_flow_call(identifier_creator, symbol_table),
            FlowExpressionKind::Scan {
                flow_expression, ..
            } => flow_expression.into_flow_call(identifier_creator, symbol_table),
            FlowExpressionKind::Timeout {
                flow_expression, ..
            } => flow_expression.into_flow_call(identifier_creator, symbol_table),
            FlowExpressionKind::Throtle {
                flow_expression, ..
            } => flow_expression.into_flow_call(identifier_creator, symbol_table),
            FlowExpressionKind::OnChange { flow_expression } => {
                flow_expression.into_flow_call(identifier_creator, symbol_table)
            }
            FlowExpressionKind::ComponentCall { inputs, .. } => inputs
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
            FlowExpressionKind::Ident { .. } => vec![],
            _ => {
                let mut statements = self.normal_form(identifier_creator, symbol_table);

                // create fresh identifier for the new statement
                let fresh_name = identifier_creator.new_identifier(
                    String::from(""),
                    String::from("x"),
                    String::from(""),
                );
                let typing = self.get_type().unwrap();
                let fresh_id = symbol_table.insert_fresh_flow(fresh_name, typing.clone());

                // create statement for the expression
                let new_statement = FlowStatement::Declaration(FlowDeclaration {
                    let_token: Default::default(),
                    pattern: Pattern {
                        kind: PatternKind::Identifier { id: fresh_id },
                        typing: Some(typing.clone()),
                        location: self.location.clone(),
                    },
                    eq_token: Default::default(),
                    flow_expression: self.clone(),
                    semi_token: Default::default(),
                });
                statements.push(new_statement);

                // change current expression be an identifier to the statement of the expression
                self.kind = FlowExpressionKind::Ident { id: fresh_id };
                // self.dependencies = Dependencies::from(vec![(fresh_id, Label::Weight(0))]);

                // return new additional statements
                statements
            }
        }
    }
}
