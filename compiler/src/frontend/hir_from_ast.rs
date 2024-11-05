prelude! {}

/// HIR Component construction from AST Component
pub mod component;
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
pub mod typ;
/// HIR Typedef construction from AST Typedef.
pub mod typedef;

/// AST transformation into HIR.
pub trait HIRFromAST<Ctxt> {
    /// Corresponding HIR construct.
    type HIR;

    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(self, ctxt: &mut Ctxt) -> TRes<Self::HIR>;
}

pub struct SimpleCtxt<'a> {
    pub syms: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
pub struct LocCtxt<'a> {
    pub loc: &'a Location,
    pub syms: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
pub struct PatLocCtxt<'a> {
    pub pat: Option<&'a ast::stmt::Pattern>,
    pub loc: &'a Location,
    pub syms: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
impl<'a> SimpleCtxt<'a> {
    pub fn new(syms: &'a mut SymbolTable, errors: &'a mut Vec<Error>) -> Self {
        Self { syms, errors }
    }
    pub fn add_loc<'b>(&'b mut self, loc: &'b Location) -> LocCtxt<'b> {
        LocCtxt::new(loc, self.syms, self.errors)
    }
    pub fn add_pat_loc<'b>(
        &'b mut self,
        pat: Option<&'b ast::stmt::Pattern>,
        loc: &'b Location,
    ) -> PatLocCtxt<'b> {
        PatLocCtxt::new(pat, loc, self.syms, self.errors)
    }
}
impl<'a> LocCtxt<'a> {
    pub fn new(loc: &'a Location, syms: &'a mut SymbolTable, errors: &'a mut Vec<Error>) -> Self {
        Self { loc, syms, errors }
    }
    pub fn add_pat<'b>(&'b mut self, pat: Option<&'b ast::stmt::Pattern>) -> PatLocCtxt<'b> {
        PatLocCtxt::new(pat, self.loc, self.syms, self.errors)
    }
}
impl<'a> PatLocCtxt<'a> {
    pub fn new(
        pat: Option<&'a ast::stmt::Pattern>,
        loc: &'a Location,
        syms: &'a mut SymbolTable,
        errors: &'a mut Vec<Error>,
    ) -> Self {
        Self {
            pat,
            loc,
            syms,
            errors,
        }
    }
    pub fn remove_pat<'b>(&'b mut self) -> LocCtxt<'b> {
        LocCtxt::new(self.loc, self.syms, self.errors)
    }
    pub fn set_pat(
        &mut self,
        pat: Option<&'a ast::stmt::Pattern>,
    ) -> Option<&'a ast::stmt::Pattern> {
        std::mem::replace(&mut self.pat, pat)
    }
}
