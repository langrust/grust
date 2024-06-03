prelude! {}

/// HIR Contract construction from AST Contract
pub mod contract;
/// HIR Equation construction from AST Equation
pub mod equation;
/// HIR Expression construction from AST Expression
pub mod expression;
/// HIR File construction from AST File
pub mod file;
/// HIR Function construction from AST Function
pub mod function;
/// HIR Interface construction from AST Interface.
pub mod interface;
/// HIR Pattern construction from AST Pattern
pub mod pattern;
/// HIR Statement construction from AST Statement
pub mod statement;
/// HIR StreamExpression construction from AST StreamExpression
pub mod stream_expression;
/// HIR Typedef construction from AST Typedef.
pub mod typedef;

/// AST transformation into HIR.
pub trait HIRFromAST {
    /// Corresponding HIR construct.
    type HIR;

    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR>;
}

impl HIRFromAST for ast::Component {
    type HIR = hir::Node;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let ast::Component {
            ident,
            contract,
            equations,
            ..
        } = self;
        let name = ident.to_string();
        let location = Location::default();
        let id = symbol_table.get_node_id(&name, false, location.clone(), errors)?;

        // create local context with all signals
        symbol_table.local();
        symbol_table.restore_context(id);
        symbol_table.enter_in_node(id);

        let statements = equations
            .into_iter()
            .map(|equation| equation.hir_from_ast(symbol_table, errors))
            .collect::<TRes<Vec<_>>>()?;
        let contract = contract.hir_from_ast(symbol_table, errors)?;

        symbol_table.leave_node();
        symbol_table.global();

        Ok(hir::Node {
            id,
            statements,
            contract,
            location,
            graph: graph::DiGraphMap::new(),
            memory: hir::Memory::new(),
        })
    }
}
