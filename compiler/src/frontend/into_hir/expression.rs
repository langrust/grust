prelude! {
    ast::expr::*,
    hir::Dependencies,
}

mod simple_expr {
    prelude! {
        ast::{
            expr::{
                Unop, Binop, IfThenElse, Application, Structure, Enumeration, Array, Tuple, Match,
                FieldAccess, TupleElementAccess, Map, Fold, Sort, Zip, TypedAbstraction, Arm,
            },
        },
        hir::expr,
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Unop<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Unop { op, expression } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Unop {
                op,
                expression: Box::new(expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Binop<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Binop {
                op,
                left_expression,
                right_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Binop {
                op,
                left_expression: Box::new(left_expression.into_hir(ctxt)?),
                right_expression: Box::new(right_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for IfThenElse<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let IfThenElse {
                expression,
                true_expression,
                false_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::IfThenElse {
                expression: Box::new(expression.into_hir(ctxt)?),
                true_expression: Box::new(true_expression.into_hir(ctxt)?),

                false_expression: Box::new(false_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Application<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Application {
                function_expression,
                inputs,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Application {
                function_expression: Box::new(function_expression.into_hir(ctxt)?),
                inputs: inputs
                    .into_iter()
                    .map(|input| input.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Structure<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Structure { name, fields } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            let id = ctxt
                .syms
                .get_struct_id(&name, false, ctxt.loc.clone(), ctxt.errors)?;
            let mut field_ids = ctxt
                .syms
                .get_struct_fields(id)
                .clone()
                .into_iter()
                .map(|id| (ctxt.syms.get_name(id).clone(), id))
                .collect::<HashMap<_, _>>();

            let fields = fields
                .into_iter()
                .map(|(field_name, expression)| {
                    let id = field_ids.remove(&field_name).map_or_else(
                        || {
                            let error = Error::UnknownField {
                                structure_name: name.clone(),
                                field_name: field_name.clone(),
                                location: ctxt.loc.clone(),
                            };
                            ctxt.errors.push(error);
                            Err(TerminationError)
                        },
                        |id| Ok(id),
                    )?;
                    let expression = expression.into_hir(ctxt)?;
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
                        location: ctxt.loc.clone(),
                    };
                    ctxt.errors.push(error);
                    Err::<(), _>(TerminationError)
                })
                .collect::<TRes<Vec<_>>>()?;

            Ok(expr::Kind::Structure { id, fields })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Enumeration<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>>
        where
            E: IntoHir<hir::ctx::PatLoc<'a>>,
        {
            let Enumeration {
                enum_name,
                elem_name,
                ..
            } = self;

            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            let enum_id =
                ctxt.syms
                    .get_enum_id(&enum_name, false, ctxt.loc.clone(), ctxt.errors)?;
            let elem_id = ctxt.syms.get_enum_elem_id(
                &elem_name,
                &enum_name,
                false,
                ctxt.loc.clone(),
                ctxt.errors,
            )?;
            // TODO check elem is in enum
            Ok(expr::Kind::Enumeration { enum_id, elem_id })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Array<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Array { elements } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Array {
                elements: elements
                    .into_iter()
                    .map(|expression| expression.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Tuple<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Tuple { elements } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Tuple {
                elements: elements
                    .into_iter()
                    .map(|expression| expression.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Match<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Match { expression, arms } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Match {
                expression: Box::new(expression.into_hir(ctxt)?),
                arms: arms
                    .into_iter()
                    .map(
                        |Arm {
                             pattern,
                             guard,
                             expression,
                         }| {
                            ctxt.syms.local();
                            pattern.store(ctxt.syms, ctxt.errors)?;
                            let pattern = pattern.into_hir(&mut ctxt.remove_pat())?;
                            let guard = guard
                                .map(|expression| expression.into_hir(ctxt))
                                .transpose()?;
                            let expression = expression.into_hir(ctxt)?;
                            ctxt.syms.global();
                            Ok((pattern, guard, vec![], expression))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for FieldAccess<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let FieldAccess { expression, field } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::FieldAccess {
                expression: Box::new(expression.into_hir(ctxt)?),
                field,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for TupleElementAccess<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let TupleElementAccess {
                expression,
                element_number,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::TupleElementAccess {
                expression: Box::new(expression.into_hir(ctxt)?),
                element_number,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Map<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Map {
                expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Map {
                expression: Box::new(expression.into_hir(ctxt)?),
                function_expression: Box::new(function_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Fold<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Fold {
                expression,
                initialization_expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Fold {
                expression: Box::new(expression.into_hir(ctxt)?),
                initialization_expression: Box::new(initialization_expression.into_hir(ctxt)?),
                function_expression: Box::new(function_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Sort<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Sort {
                expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Sort {
                expression: Box::new(expression.into_hir(ctxt)?),
                function_expression: Box::new(function_expression.into_hir(ctxt)?),
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for Zip<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let Zip { arrays } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|array| array.into_hir(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> IntoHir<hir::ctx::PatLoc<'a>> for TypedAbstraction<E>
    where
        E: IntoHir<hir::ctx::PatLoc<'a>>,
    {
        type Hir = expr::Kind<E::Hir>;

        /// Transforms AST into HIR and check identifiers good use.
        fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<expr::Kind<E::Hir>> {
            let TypedAbstraction { inputs, expression } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use

            ctxt.syms.local();
            let inputs = inputs
                .into_iter()
                .map(|(input_name, typing)| {
                    let typing = typing.into_hir(&mut ctxt.remove_pat())?;
                    ctxt.syms.insert_identifier(
                        input_name,
                        Some(typing),
                        true,
                        ctxt.loc.clone(),
                        ctxt.errors,
                    )
                })
                .collect::<TRes<Vec<_>>>()?;
            let expression = expression.into_hir(ctxt)?;
            ctxt.syms.global();

            Ok(expr::Kind::Abstraction {
                inputs,
                expression: Box::new(expression),
            })
        }
    }
}

impl<'a> IntoHir<hir::ctx::PatLoc<'a>> for Expr {
    type Hir = hir::Expr;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR expression and check identifiers good use
    fn into_hir(self, ctxt: &mut hir::ctx::PatLoc<'a>) -> TRes<Self::Hir> {
        let kind = match self {
            Self::Constant(constant) => hir::expr::Kind::Constant { constant },
            Self::Identifier(id) => {
                let id = ctxt
                    .syms
                    .get_identifier_id(&id, false, ctxt.loc.clone(), &mut vec![])
                    .or_else(|_| {
                        ctxt.syms
                            .get_function_id(&id, false, ctxt.loc.clone(), ctxt.errors)
                    })?;
                hir::expr::Kind::Identifier { id }
            }
            Self::Unop(expression) => expression.into_hir(ctxt)?,
            Self::Binop(expression) => expression.into_hir(ctxt)?,
            Self::IfThenElse(expression) => expression.into_hir(ctxt)?,
            Self::Application(expression) => expression.into_hir(ctxt)?,
            Self::TypedAbstraction(expression) => expression.into_hir(ctxt)?,
            Self::Structure(expression) => expression.into_hir(ctxt)?,
            Self::Tuple(expression) => expression.into_hir(ctxt)?,
            Self::Enumeration(expression) => expression.into_hir(ctxt)?,
            Self::Array(expression) => expression.into_hir(ctxt)?,
            Self::Match(expression) => expression.into_hir(ctxt)?,
            Self::FieldAccess(expression) => expression.into_hir(ctxt)?,
            Self::TupleElementAccess(expression) => expression.into_hir(ctxt)?,
            Self::Map(expression) => expression.into_hir(ctxt)?,
            Self::Fold(expression) => expression.into_hir(ctxt)?,
            Self::Sort(expression) => expression.into_hir(ctxt)?,
            Self::Zip(expression) => expression.into_hir(ctxt)?,
        };
        Ok(hir::Expr {
            kind,
            typing: None,
            location: ctxt.loc.clone(),
            dependencies: Dependencies::new(),
        })
    }
}
