//! Contexts.

prelude! {}

pub struct Simple<'a> {
    pub syms: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
pub struct Loc<'a> {
    pub loc: &'a Location,
    pub syms: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
pub struct PatLoc<'a> {
    pub pat: Option<&'a ast::stmt::Pattern>,
    pub loc: &'a Location,
    pub syms: &'a mut SymbolTable,
    pub errors: &'a mut Vec<Error>,
}
impl<'a> Simple<'a> {
    pub fn new(syms: &'a mut SymbolTable, errors: &'a mut Vec<Error>) -> Self {
        Self { syms, errors }
    }
    pub fn add_loc<'b>(&'b mut self, loc: &'b Location) -> Loc<'b> {
        Loc::new(loc, self.syms, self.errors)
    }
    pub fn add_pat_loc<'b>(
        &'b mut self,
        pat: Option<&'b ast::stmt::Pattern>,
        loc: &'b Location,
    ) -> PatLoc<'b> {
        PatLoc::new(pat, loc, self.syms, self.errors)
    }
}
impl<'a> Loc<'a> {
    pub fn new(loc: &'a Location, syms: &'a mut SymbolTable, errors: &'a mut Vec<Error>) -> Self {
        Self { loc, syms, errors }
    }
    pub fn add_pat<'b>(&'b mut self, pat: Option<&'b ast::stmt::Pattern>) -> PatLoc<'b> {
        PatLoc::new(pat, self.loc, self.syms, self.errors)
    }
}
impl<'a> PatLoc<'a> {
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
    pub fn remove_pat<'b>(&'b mut self) -> Loc<'b> {
        Loc::new(self.loc, self.syms, self.errors)
    }
    pub fn set_pat(
        &mut self,
        pat: Option<&'a ast::stmt::Pattern>,
    ) -> Option<&'a ast::stmt::Pattern> {
        std::mem::replace(&mut self.pat, pat)
    }
}
