prelude! {
    frontend::typing_analysis::TypeAnalysis,
}

mod abstraction;
mod application;
mod array;
mod binop;
mod constant;
mod enumeration;
mod field_access;
mod fold;
mod identifier;
mod if_then_else;
mod map;
mod r#match;
mod sort;
mod structure;
mod tuple;
mod tuple_element_access;
mod unop;
mod when;
mod zip;

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Tries to type the given construct.
    pub fn typing(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            hir::expr::Kind::Constant { .. } => self.typing_constant(),
            hir::expr::Kind::Identifier { .. } => self.typing_identifier(symbol_table),
            hir::expr::Kind::Unop { .. } => self.typing_unop(location, symbol_table, errors),
            hir::expr::Kind::Binop { .. } => self.typing_binop(location, symbol_table, errors),
            hir::expr::Kind::IfThenElse { .. } => {
                self.typing_if_then_else(location, symbol_table, errors)
            }
            hir::expr::Kind::Application { .. } => {
                self.typing_application(location, symbol_table, errors)
            }
            hir::expr::Kind::Abstraction { .. } => self.typing_abstraction(symbol_table, errors),
            hir::expr::Kind::Structure { .. } => {
                self.typing_structure(location, symbol_table, errors)
            }
            hir::expr::Kind::Array { .. } => self.typing_array(location, symbol_table, errors),
            hir::expr::Kind::Tuple { .. } => self.typing_tuple(symbol_table, errors),
            hir::expr::Kind::When { .. } => self.typing_when(location, symbol_table, errors),
            hir::expr::Kind::Match { .. } => self.typing_match(location, symbol_table, errors),
            hir::expr::Kind::FieldAccess { .. } => {
                self.typing_field_access(location, symbol_table, errors)
            }
            hir::expr::Kind::Map { .. } => self.typing_map(location, symbol_table, errors),
            hir::expr::Kind::Fold { .. } => self.typing_fold(location, symbol_table, errors),
            hir::expr::Kind::Sort { .. } => self.typing_sort(location, symbol_table, errors),
            hir::expr::Kind::Zip { .. } => self.typing_zip(location, symbol_table, errors),
            hir::expr::Kind::TupleElementAccess { .. } => {
                self.typing_tuple_element_access(location, symbol_table, errors)
            }
            hir::expr::Kind::Enumeration { .. } => self.typing_enumeration(symbol_table),
        }
    }
}

impl TypeAnalysis for hir::Expr {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        self.typing = Some(self.kind.typing(&self.location, symbol_table, errors)?);
        Ok(())
    }
    fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
    fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
}
