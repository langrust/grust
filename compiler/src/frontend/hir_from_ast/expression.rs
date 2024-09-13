prelude! {
    ast::expr::*,
    hir::Dependencies,
}

use super::{HIRFromAST, PatLocCtxt};

mod simple_expr {
    prelude! {
        ast::{
            expr::{
                Unop, Binop, IfThenElse, Application, Structure, Enumeration, Array, Tuple, Match,
                FieldAccess, TupleElementAccess, Map, Fold, Sort, Zip, TypedAbstraction, Arm,
            },
        },
        hir::expr,
        frontend::hir_from_ast::{HIRFromAST, PatLocCtxt},
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Unop<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Unop { op, expression } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Unop {
                op,
                expression: Box::new(expression.hir_from_ast(ctxt)?),
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Binop<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Binop {
                op,
                left_expression,
                right_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Binop {
                op,
                left_expression: Box::new(left_expression.hir_from_ast(ctxt)?),
                right_expression: Box::new(right_expression.hir_from_ast(ctxt)?),
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for IfThenElse<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let IfThenElse {
                expression,
                true_expression,
                false_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::IfThenElse {
                expression: Box::new(expression.hir_from_ast(ctxt)?),
                true_expression: Box::new(true_expression.hir_from_ast(ctxt)?),

                false_expression: Box::new(false_expression.hir_from_ast(ctxt)?),
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Application<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Application {
                function_expression,
                inputs,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Application {
                function_expression: Box::new(function_expression.hir_from_ast(ctxt)?),
                inputs: inputs
                    .into_iter()
                    .map(|input| input.hir_from_ast(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Structure<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
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
                    let expression = expression.hir_from_ast(ctxt)?;
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

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Enumeration<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>>
        where
            E: HIRFromAST<PatLocCtxt<'a>>,
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

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Array<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Array { elements } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Array {
                elements: elements
                    .into_iter()
                    .map(|expression| expression.hir_from_ast(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Tuple<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Tuple { elements } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Tuple {
                elements: elements
                    .into_iter()
                    .map(|expression| expression.hir_from_ast(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Match<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Match { expression, arms } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Match {
                expression: Box::new(expression.hir_from_ast(ctxt)?),
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
                            let pattern = pattern.hir_from_ast(&mut ctxt.remove_pat())?;
                            let guard = guard
                                .map(|expression| expression.hir_from_ast(ctxt))
                                .transpose()?;
                            let expression = expression.hir_from_ast(ctxt)?;
                            ctxt.syms.global();
                            Ok((pattern, guard, vec![], expression))
                        },
                    )
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for FieldAccess<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let FieldAccess { expression, field } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::FieldAccess {
                expression: Box::new(expression.hir_from_ast(ctxt)?),
                field,
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for TupleElementAccess<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let TupleElementAccess {
                expression,
                element_number,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::TupleElementAccess {
                expression: Box::new(expression.hir_from_ast(ctxt)?),
                element_number,
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Map<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Map {
                expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Map {
                expression: Box::new(expression.hir_from_ast(ctxt)?),
                function_expression: Box::new(function_expression.hir_from_ast(ctxt)?),
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Fold<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Fold {
                expression,
                initialization_expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Fold {
                expression: Box::new(expression.hir_from_ast(ctxt)?),
                initialization_expression: Box::new(initialization_expression.hir_from_ast(ctxt)?),
                function_expression: Box::new(function_expression.hir_from_ast(ctxt)?),
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Sort<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Sort {
                expression,
                function_expression,
            } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Sort {
                expression: Box::new(expression.hir_from_ast(ctxt)?),
                function_expression: Box::new(function_expression.hir_from_ast(ctxt)?),
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for Zip<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let Zip { arrays } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use
            Ok(expr::Kind::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|array| array.hir_from_ast(ctxt))
                    .collect::<TRes<Vec<_>>>()?,
            })
        }
    }

    impl<'a, E> HIRFromAST<PatLocCtxt<'a>> for TypedAbstraction<E>
    where
        E: HIRFromAST<PatLocCtxt<'a>>,
    {
        type HIR = expr::Kind<E::HIR>;

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<expr::Kind<E::HIR>> {
            let TypedAbstraction { inputs, expression } = self;
            // precondition: identifiers are stored in symbol table
            // postcondition: construct HIR expression kind and check identifiers good use

            ctxt.syms.local();
            let inputs = inputs
                .into_iter()
                .map(|(input_name, typing)| {
                    let typing = typing.hir_from_ast(&mut ctxt.remove_pat())?;
                    ctxt.syms.insert_identifier(
                        input_name,
                        Some(typing),
                        true,
                        ctxt.loc.clone(),
                        ctxt.errors,
                    )
                })
                .collect::<TRes<Vec<_>>>()?;
            let expression = expression.hir_from_ast(ctxt)?;
            ctxt.syms.global();

            Ok(expr::Kind::Abstraction {
                inputs,
                expression: Box::new(expression),
            })
        }
    }
}

impl<'a> HIRFromAST<PatLocCtxt<'a>> for Expr {
    type HIR = hir::Expr;

    // precondition: identifiers are stored in symbol table
    // postcondition: construct HIR expression and check identifiers good use
    fn hir_from_ast(self, ctxt: &mut PatLocCtxt<'a>) -> TRes<Self::HIR> {
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
            Self::Unop(expression) => expression.hir_from_ast(ctxt)?,
            Self::Binop(expression) => expression.hir_from_ast(ctxt)?,
            Self::IfThenElse(expression) => expression.hir_from_ast(ctxt)?,
            Self::Application(expression) => expression.hir_from_ast(ctxt)?,
            Self::TypedAbstraction(expression) => expression.hir_from_ast(ctxt)?,
            Self::Structure(expression) => expression.hir_from_ast(ctxt)?,
            Self::Tuple(expression) => expression.hir_from_ast(ctxt)?,
            Self::Enumeration(expression) => expression.hir_from_ast(ctxt)?,
            Self::Array(expression) => expression.hir_from_ast(ctxt)?,
            Self::Match(expression) => expression.hir_from_ast(ctxt)?,
            Self::FieldAccess(expression) => expression.hir_from_ast(ctxt)?,
            Self::TupleElementAccess(expression) => expression.hir_from_ast(ctxt)?,
            Self::Map(expression) => expression.hir_from_ast(ctxt)?,
            Self::Fold(expression) => expression.hir_from_ast(ctxt)?,
            Self::Sort(expression) => expression.hir_from_ast(ctxt)?,
            Self::Zip(expression) => expression.hir_from_ast(ctxt)?,
        };
        Ok(hir::Expr {
            kind,
            typing: None,
            location: ctxt.loc.clone(),
            dependencies: Dependencies::new(),
        })
    }
}
