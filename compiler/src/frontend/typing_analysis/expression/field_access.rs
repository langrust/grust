prelude! {
    frontend::TypeAnalysis,
    ast::symbol::SymbolKind,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the field access expression.
    pub fn typing_field_access(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            hir::expr::Kind::FieldAccess {
                ref mut expression,
                ref field,
            } => {
                expression.typing(symbol_table, errors)?;

                match expression.get_type().unwrap() {
                    Typ::Structure { name, id } => {
                        let symbol = symbol_table
                            .get_symbol(*id)
                            .expect("there should be a symbole");
                        match symbol.kind() {
                            SymbolKind::Structure { fields } => {
                                let option_field_type = fields
                                    .iter()
                                    .filter(|id| {
                                        let field_name = symbol_table.get_name(**id);
                                        field == field_name
                                    })
                                    .map(|id| symbol_table.get_type(*id).clone())
                                    .next();
                                if let Some(field_type) = option_field_type {
                                    Ok(field_type)
                                } else {
                                    let error = Error::UnknownField {
                                        structure_name: name.clone(),
                                        field_name: field.clone(),
                                        location: location.clone(),
                                    };
                                    errors.push(error);
                                    Err(TerminationError)
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    given_type => {
                        let error = Error::ExpectStructure {
                            given_type: given_type.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
