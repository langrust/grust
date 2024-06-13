prelude! {
    ast::{
        expr::{
            Unop, Binop, IfThenElse, Application, Structure, Enumeration, Array, Tuple, Match,
            FieldAccess, TupleElementAccess, Map, Fold, Sort, Zip, TypedAbstraction, Arm,
        },
        interface::{ ComponentCall, OnChange, Sample, Scan, Throttle, Timeout },
    },
    hir::expr,
}

pub trait SimpleHirExt<T> {
    fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<T>;
}

pub trait HirExt<Inner> {
    fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<Inner::HIR>>
    where
        Inner: HIRFromAST;
}

impl<E> HirExt<E> for Unop<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Unop { op, expression } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Unop {
            op,
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> HirExt<E> for Binop<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Binop {
            op,
            left_expression,
            right_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Binop {
            op,
            left_expression: Box::new(left_expression.hir_from_ast(symbol_table, errors)?),
            right_expression: Box::new(right_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> HirExt<E> for IfThenElse<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let IfThenElse {
            expression,
            true_expression,
            false_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::IfThenElse {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            true_expression: Box::new(true_expression.hir_from_ast(symbol_table, errors)?),

            false_expression: Box::new(false_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> HirExt<E> for Application<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Application {
            function_expression,
            inputs,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Application {
            function_expression: Box::new(function_expression.hir_from_ast(symbol_table, errors)?),
            inputs: inputs
                .into_iter()
                .map(|input| input.hir_from_ast(symbol_table, errors))
                .collect::<TRes<Vec<_>>>()?,
        })
    }
}

impl<E> HirExt<E> for Structure<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Structure { name, fields } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        let id = symbol_table.get_struct_id(&name, false, location.clone(), errors)?;
        let mut field_ids = symbol_table
            .get_struct_fields(id)
            .clone()
            .into_iter()
            .map(|id| (symbol_table.get_name(id).clone(), id))
            .collect::<HashMap<_, _>>();

        let fields = fields
            .into_iter()
            .map(|(field_name, expression)| {
                let id = field_ids.remove(&field_name).map_or_else(
                    || {
                        let error = Error::UnknownField {
                            structure_name: name.clone(),
                            field_name: field_name.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    },
                    |id| Ok(id),
                )?;
                let expression = expression.hir_from_ast(symbol_table, errors)?;
                Ok((id, expression))
            })
            .collect::<TRes<Vec<_>>>()?;

        // check if there are no missing fields
        field_ids
            .keys()
            .map(|field_name| {
                let error = Error::MissingField {
                    structure_name: name.clone(),
                    field_name: field_name.clone(),
                    location: location.clone(),
                };
                errors.push(error);
                Err::<(), _>(TerminationError)
            })
            .collect::<TRes<Vec<_>>>()?;

        Ok(expr::Kind::Structure { id, fields })
    }
}

impl<E> HirExt<E> for Enumeration<E> {
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>>
    where
        E: HIRFromAST,
    {
        let Enumeration {
            enum_name,
            elem_name,
            ..
        } = self;

        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        let enum_id = symbol_table.get_enum_id(&enum_name, false, location.clone(), errors)?;
        let elem_id = symbol_table.get_enum_elem_id(
            &elem_name,
            &enum_name,
            false,
            location.clone(),
            errors,
        )?;
        // TODO check elem is in enum
        Ok(expr::Kind::Enumeration { enum_id, elem_id })
    }
}

impl<E> HirExt<E> for Array<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Array { elements } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Array {
            elements: elements
                .into_iter()
                .map(|expression| expression.hir_from_ast(symbol_table, errors))
                .collect::<TRes<Vec<_>>>()?,
        })
    }
}

impl<E> HirExt<E> for Tuple<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Tuple { elements } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Tuple {
            elements: elements
                .into_iter()
                .map(|expression| expression.hir_from_ast(symbol_table, errors))
                .collect::<TRes<Vec<_>>>()?,
        })
    }
}

impl<E> HirExt<E> for Match<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Match { expression, arms } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Match {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            arms: arms
                .into_iter()
                .map(
                    |Arm {
                         pattern,
                         guard,
                         expression,
                     }| {
                        symbol_table.local();
                        pattern.store(true, symbol_table, errors)?;
                        let pattern = pattern.hir_from_ast(symbol_table, errors)?;
                        let guard = guard
                            .map(|expression| expression.hir_from_ast(symbol_table, errors))
                            .transpose()?;
                        let expression = expression.hir_from_ast(symbol_table, errors)?;
                        symbol_table.global();
                        Ok((pattern, guard, vec![], expression))
                    },
                )
                .collect::<TRes<Vec<_>>>()?,
        })
    }
}

impl<E> HirExt<E> for FieldAccess<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let FieldAccess { expression, field } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::FieldAccess {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            field,
        })
    }
}

impl<E> HirExt<E> for TupleElementAccess<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let TupleElementAccess {
            expression,
            element_number,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::TupleElementAccess {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            element_number,
        })
    }
}

impl<E> HirExt<E> for Map<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Map {
            expression,
            function_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Map {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            function_expression: Box::new(function_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> HirExt<E> for Fold<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Fold {
            expression,
            initialization_expression,
            function_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Fold {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            initialization_expression: Box::new(
                initialization_expression.hir_from_ast(symbol_table, errors)?,
            ),
            function_expression: Box::new(function_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> HirExt<E> for Sort<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Sort {
            expression,
            function_expression,
        } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Sort {
            expression: Box::new(expression.hir_from_ast(symbol_table, errors)?),
            function_expression: Box::new(function_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl<E> HirExt<E> for Zip<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let Zip { arrays } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use
        Ok(expr::Kind::Zip {
            arrays: arrays
                .into_iter()
                .map(|array| array.hir_from_ast(symbol_table, errors))
                .collect::<TRes<Vec<_>>>()?,
        })
    }
}

impl<E> HirExt<E> for TypedAbstraction<E>
where
    E: HIRFromAST,
{
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<expr::Kind<E::HIR>> {
        let TypedAbstraction { inputs, expression } = self;
        // precondition: identifiers are stored in symbol table
        // postcondition: construct HIR expression kind and check identifiers good use

        symbol_table.local();
        let inputs = inputs
            .into_iter()
            .map(|(input_name, typing)| {
                let typing = typing.hir_from_ast(&location, symbol_table, errors)?;
                symbol_table.insert_identifier(
                    input_name,
                    Some(typing),
                    true,
                    location.clone(),
                    errors,
                )
            })
            .collect::<TRes<Vec<_>>>()?;
        let expression = expression.hir_from_ast(symbol_table, errors)?;
        symbol_table.global();

        Ok(expr::Kind::Abstraction {
            inputs,
            expression: Box::new(expression),
        })
    }
}

impl SimpleHirExt<hir::flow::Kind> for ast::interface::Sample {
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<hir::flow::Kind> {
        let Sample {
            flow_expression,
            period_ms,
            ..
        } = self;
        Ok(hir::flow::Kind::Sample {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
            period_ms: period_ms.base10_parse().unwrap(),
        })
    }
}

impl SimpleHirExt<hir::flow::Kind> for Scan {
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<hir::flow::Kind> {
        let Scan {
            flow_expression,
            period_ms,
            ..
        } = self;
        Ok(hir::flow::Kind::Scan {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
            period_ms: period_ms.base10_parse().unwrap(),
        })
    }
}

impl SimpleHirExt<hir::flow::Kind> for Timeout {
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<hir::flow::Kind> {
        let Timeout {
            flow_expression,
            deadline,
            ..
        } = self;
        Ok(hir::flow::Kind::Timeout {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
            deadline: deadline.base10_parse().unwrap(),
        })
    }
}

impl SimpleHirExt<hir::flow::Kind> for Throttle {
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<hir::flow::Kind> {
        let Throttle {
            flow_expression,
            delta,
            ..
        } = self;
        Ok(hir::flow::Kind::Throttle {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
            delta,
        })
    }
}

impl SimpleHirExt<hir::flow::Kind> for OnChange {
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        _location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<hir::flow::Kind> {
        let OnChange {
            flow_expression, ..
        } = self;
        Ok(hir::flow::Kind::OnChange {
            flow_expression: Box::new(flow_expression.hir_from_ast(symbol_table, errors)?),
        })
    }
}

impl SimpleHirExt<hir::flow::Kind> for ComponentCall {
    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<hir::flow::Kind> {
        let ComponentCall {
            ident_component,
            inputs,
            ..
        } = self;

        let name = ident_component.to_string();

        // get called component id
        let component_id = symbol_table.get_node_id(&name, false, location.clone(), errors)?;

        let component_inputs = symbol_table.get_node_inputs(component_id).clone();

        // check inputs and node_inputs have the same length
        if inputs.len() != component_inputs.len() {
            let error = Error::IncompatibleInputsNumber {
                given_inputs_number: inputs.len(),
                expected_inputs_number: component_inputs.len(),
                location: location.clone(),
            };
            errors.push(error);
            return Err(TerminationError);
        }

        // transform inputs and map then to the identifiers of the component inputs
        let inputs = inputs
            .into_iter()
            .zip(component_inputs)
            .map(|(input, id)| Ok((id, input.hir_from_ast(symbol_table, errors)?)))
            .collect::<TRes<Vec<_>>>()?;

        Ok(hir::flow::Kind::ComponentCall {
            component_id,
            inputs,
        })
    }
}

mod pattern {
    prelude! {
        ast::pattern::{Enumeration, PatSome, Structure, Tuple, Typed},
    }

    impl SimpleHirExt<hir::pattern::Kind> for Typed {
        fn hir_from_ast(
            self,
            _location: &Location,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<hir::pattern::Kind> {
            let Typed {
                pattern, typing, ..
            } = self;
            let location = Location::default();

            let pattern = Box::new(pattern.hir_from_ast(symbol_table, errors)?);
            let typing = typing.hir_from_ast(&location, symbol_table, errors)?;
            Ok(hir::pattern::Kind::Typed { pattern, typing })
        }
    }

    impl SimpleHirExt<hir::pattern::Kind> for Structure {
        fn hir_from_ast(
            self,
            _location: &Location,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<hir::pattern::Kind> {
            let Structure { name, fields, rest } = self;
            let location = Location::default();

            let id = symbol_table.get_struct_id(&name, false, location.clone(), errors)?;
            let mut field_ids = symbol_table
                .get_struct_fields(id)
                .clone()
                .into_iter()
                .map(|id| (symbol_table.get_name(id).clone(), id))
                .collect::<HashMap<_, _>>();

            let fields = fields
                .into_iter()
                .map(|(field_name, optional_pattern)| {
                    let id = field_ids.remove(&field_name).map_or_else(
                        || {
                            let error = Error::UnknownField {
                                structure_name: name.clone(),
                                field_name: field_name.clone(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        },
                        |id| Ok(id),
                    )?;
                    let pattern = optional_pattern
                        .map(|pattern| pattern.hir_from_ast(symbol_table, errors))
                        .transpose()?;
                    Ok((id, pattern))
                })
                .collect::<TRes<Vec<_>>>()?;

            if rest.is_none() {
                // check if there are no missing fields
                field_ids
                    .keys()
                    .map(|field_name| {
                        let error = Error::MissingField {
                            structure_name: name.clone(),
                            field_name: field_name.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        TRes::<()>::Err(TerminationError)
                    })
                    .collect::<TRes<Vec<_>>>()?;
            }

            Ok(hir::pattern::Kind::Structure { id, fields })
        }
    }

    impl SimpleHirExt<hir::pattern::Kind> for Enumeration {
        fn hir_from_ast(
            self,
            _location: &Location,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<hir::pattern::Kind> {
            let Enumeration {
                enum_name,
                elem_name,
            } = self;
            let location = Location::default();

            let enum_id = symbol_table.get_enum_id(&enum_name, false, location.clone(), errors)?;
            let elem_id = symbol_table.get_enum_elem_id(
                &elem_name,
                &enum_name,
                false,
                location.clone(),
                errors,
            )?;
            Ok(hir::pattern::Kind::Enumeration { enum_id, elem_id })
        }
    }

    impl SimpleHirExt<hir::pattern::Kind> for Tuple {
        fn hir_from_ast(
            self,
            _location: &Location,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<hir::pattern::Kind> {
            let Tuple { elements } = self;
            Ok(hir::pattern::Kind::Tuple {
                elements: elements
                    .into_iter()
                    .map(|pattern| pattern.hir_from_ast(symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl SimpleHirExt<hir::pattern::Kind> for PatSome {
        // #TODO: why is this dead code?
        fn hir_from_ast(
            self,
            _location: &Location,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<hir::pattern::Kind> {
            let PatSome { pattern } = self;
            Ok(hir::pattern::Kind::Some {
                pattern: Box::new(pattern.hir_from_ast(symbol_table, errors)?),
            })
        }
    }
}
