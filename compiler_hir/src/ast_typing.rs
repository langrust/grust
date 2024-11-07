prelude! {}

/// Performs type analysis.
pub trait Typing {
    /// Tries to type the given construct.
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;

    /// Get type from construct.
    fn get_type(&self) -> Option<&Typ> {
        None
    }

    /// Get mutable type from construct.
    fn get_type_mut(&mut self) -> Option<&mut Typ> {
        None
    }
}

impl Typing for File {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        for c in self.components.iter_mut() {
            c.typing(symbols, errors)?;
        }
        for f in self.functions.iter_mut() {
            f.typing(symbols, errors)?;
        }
        for s in self.interface.services.iter_mut() {
            for stmt in s.statements.values_mut() {
                stmt.typing(symbols, errors)?
            }
        }
        Ok(())
    }
}

impl Typing for Function {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        for stmt in self.statements.iter_mut() {
            stmt.typing(symbols, errors)?;
        }
        self.returned.typing(symbols, errors)?;
        let expected_type = symbols.get_function_output_type(self.id);
        self.returned
            .get_type()
            .unwrap()
            .eq_check(expected_type, self.loc.clone(), errors)
    }
}

impl Typing for Component {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        if let Component::Definition(comp_def) = self {
            comp_def.typing(symbols, errors)
        } else {
            Ok(())
        }
    }
}

impl Typing for ComponentDefinition {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        for stmt in self.statements.iter_mut() {
            stmt.typing(symbols, errors)?;
        }
        self.contract.typing(symbols, errors)
    }
}

impl Typing for Contract {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        for term in self.requires.iter_mut() {
            term.typing(symbols, errors)?
        }
        for term in self.ensures.iter_mut() {
            term.typing(symbols, errors)?
        }
        for term in self.invariant.iter_mut() {
            term.typing(symbols, errors)?
        }
        Ok(())
    }
}

impl Typing for contract::Term {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let ty = match &mut self.kind {
            contract::Kind::Constant { constant } => constant.get_type(),
            contract::Kind::Identifier { id } => symbols.get_type(*id).clone(),
            contract::Kind::Enumeration { enum_id, .. } => Typ::Enumeration {
                name: Ident::new(symbols.get_name(*enum_id), Span::call_site()),
                id: *enum_id,
            },
            contract::Kind::Unary { op, term } => {
                term.typing(symbols, errors)?;
                let ty = term.typing.as_ref().unwrap().clone();
                let mut unop_type = op.get_type();
                unop_type.apply(vec![ty], self.loc.clone(), errors)?
            }
            contract::Kind::Binary { op, left, right } => {
                left.typing(symbols, errors)?;
                let left_type = left.typing.as_ref().unwrap().clone();
                right.typing(symbols, errors)?;
                let right_type = right.typing.as_ref().unwrap().clone();
                let mut binop_type = op.get_type();
                binop_type.apply(vec![left_type, right_type], self.loc.clone(), errors)?
            }
            contract::Kind::ForAll { term, .. } => {
                term.typing(symbols, errors)?;
                let ty = term.typing.as_ref().unwrap();
                ty.eq_check(&Typ::bool(), self.loc.clone(), errors)?;
                Typ::bool()
            }
            contract::Kind::Implication { left, right } => {
                left.typing(symbols, errors)?;
                let ty = left.typing.as_ref().unwrap();
                ty.eq_check(&Typ::bool(), self.loc.clone(), errors)?;
                right.typing(symbols, errors)?;
                let ty = right.typing.as_ref().unwrap();
                ty.eq_check(&Typ::bool(), self.loc.clone(), errors)?;
                ty.clone()
            }
            contract::Kind::PresentEvent { event_id, pattern } => {
                let typing = symbols.get_type(*event_id).clone();
                match &typing {
                    Typ::SMEvent { ty, .. } => {
                        symbols.set_type(*pattern, *ty.clone());
                    }
                    _ => unreachable!(),
                };
                typing
            }
        };
        self.typing = Some(ty);
        Ok(())
    }
}

impl Typing for interface::FlowStatement {
    // pre-condition: identifiers associated with statement is already typed
    // post-condition: expression associated with statement is typed and checked
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        use interface::*;
        match self {
            FlowStatement::Declaration(FlowDeclaration { pattern, expr, .. }) => {
                let expected_type = pattern.typing.as_ref().unwrap();
                expr.typing(symbols, errors)?;
                let expression_type = expr.get_type().unwrap();
                expression_type.eq_check(expected_type, Location::default(), errors)
            }
            FlowStatement::Instantiation(FlowInstantiation { pattern, expr, .. }) => {
                pattern.typing(symbols, errors)?;
                let expected_type = pattern.typing.as_ref().unwrap();
                expr.typing(symbols, errors)?;
                let expression_type = expr.get_type().unwrap();
                expression_type.eq_check(expected_type, Location::default(), errors)
            }
        }
    }
}

impl Typing for flow::Expr {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let loc = Location::default();

        match &mut self.kind {
            flow::Kind::Ident { id } => {
                let typing = symbols.get_type(*id);
                self.typing = Some(typing.clone());
                Ok(())
            }
            flow::Kind::Sample { expr, .. } => {
                expr.typing(symbols, errors)?;
                // get expression type
                let typing = expr.get_type().unwrap();
                match typing {
                    Typ::Event { ty: typing, .. } => {
                        // set typing
                        self.typing = Some(Typ::signal((**typing).clone()));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectEvent {
                            given_type: given_type.clone(),
                            loc: loc,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            flow::Kind::Scan { expr, .. } => {
                expr.typing(symbols, errors)?;
                // get expression type
                let typing = expr.get_type().unwrap();
                match typing {
                    Typ::Signal { ty: typing, .. } => {
                        // set typing
                        self.typing = Some(Typ::event((**typing).clone()));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectSignal {
                            given_type: given_type.clone(),
                            loc: loc,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            flow::Kind::Timeout { expr, .. } => {
                expr.typing(symbols, errors)?;
                // get expression type
                match expr.get_type().unwrap() {
                    Typ::Event { .. } => (),
                    given_type => {
                        let error = Error::ExpectEvent {
                            given_type: given_type.clone(),
                            loc: loc,
                        };
                        errors.push(error);
                        return Err(TerminationError);
                    }
                }
                // set typing
                self.typing = Some(Typ::event(Typ::unit()));
                Ok(())
            }
            flow::Kind::Throttle { expr, delta } => {
                expr.typing(symbols, errors)?;
                // get expression type
                let typing = expr.get_type().unwrap();
                match typing {
                    Typ::Signal { ty: typing, .. } => {
                        let delta_ty = delta.get_type();
                        typing.eq_check(&delta_ty, loc, errors)?;
                        // set typing
                        self.typing = Some(Typ::signal((**typing).clone()));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectSignal {
                            given_type: given_type.clone(),
                            loc: loc,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            flow::Kind::OnChange { expr } => {
                expr.typing(symbols, errors)?;
                // get expression type
                let typing = expr.get_type().unwrap();
                match typing {
                    Typ::Signal { ty: typing, .. } => {
                        // set typing
                        self.typing = Some(Typ::event((**typing).clone()));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectSignal {
                            given_type: given_type.clone(),
                            loc: loc,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            flow::Kind::Merge { expr_1, expr_2, .. } => {
                expr_1.typing(symbols, errors)?;
                expr_2.typing(symbols, errors)?;
                // get expression type
                match expr_1.get_type().unwrap() {
                    Typ::Event { ty: typing_1, .. } => {
                        match expr_2.get_type().unwrap() {
                            Typ::Event { ty: typing_2, .. } => {
                                typing_2.eq_check(typing_1.as_ref(), loc, errors)?;
                                // set typing
                                self.typing = Some(Typ::event((**typing_1).clone()));
                                Ok(())
                            }
                            given_type => {
                                let error = Error::ExpectEvent {
                                    given_type: given_type.clone(),
                                    loc: loc,
                                };
                                errors.push(error);
                                Err(TerminationError)
                            }
                        }
                    }
                    given_type => {
                        let error = Error::ExpectEvent {
                            given_type: given_type.clone(),
                            loc: loc,
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            flow::Kind::ComponentCall {
                ref component_id,
                ref mut inputs,
                ..
            } => {
                // type all inputs and check their types
                inputs
                    .iter_mut()
                    .map(|(id, input)| {
                        input.typing(symbols, errors)?;
                        let input_type = input.get_type().unwrap().convert();
                        let expected_type = symbols.get_type(*id);
                        input_type.eq_check(expected_type, self.loc.clone(), errors)
                    })
                    .collect::<TRes<()>>()?;

                // get the outputs types of the called component
                let mut outputs_types = symbols
                    .get_node_outputs(*component_id)
                    .iter()
                    .map(|(_, output_id)| {
                        let output_type = symbols.get_type(*output_id);
                        output_type.rev_convert()
                    })
                    .collect::<Vec<_>>();

                // construct node application type
                let node_application_type = if outputs_types.len() == 1 {
                    outputs_types.pop().unwrap()
                } else {
                    Typ::tuple(outputs_types)
                };

                self.typing = Some(node_application_type);
                Ok(())
            }
        }
    }

    fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
}

impl Typing for stream::Expr {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        match self.kind {
            stream::Kind::FollowedBy {
                id,
                ref mut constant,
            } => {
                // type expressions
                constant.typing(symbols, errors)?;

                // check it is equal to constant type
                let id_type = symbols.get_type(id);
                let constant_type = constant.get_type().unwrap();
                id_type.eq_check(constant_type, self.loc.clone(), errors)?;

                // check the scope is not 'very_local'
                if let Scope::VeryLocal = symbols.get_scope(id) {
                    return Err(TerminationError); // todo generate error
                }

                self.typing = Some(constant_type.clone());
                Ok(())
            }

            stream::Kind::NodeApplication {
                called_node_id,
                ref mut inputs,
                ..
            } => {
                // type all inputs and check their types
                inputs
                    .iter_mut()
                    .map(|(id, input)| {
                        input.typing(symbols, errors)?;

                        let input_type = input.typing.as_ref().unwrap();
                        let expected_type = symbols.get_type(*id);
                        input_type.eq_check(expected_type, self.loc.clone(), errors)
                    })
                    .collect::<TRes<()>>()?;

                // get the called signal type
                let node_application_type = {
                    let mut outputs_types = symbols
                        .get_node_outputs(called_node_id)
                        .iter()
                        .map(|(_, output_signal)| symbols.get_type(*output_signal).clone())
                        .collect::<Vec<_>>();
                    if outputs_types.len() == 1 {
                        outputs_types.pop().unwrap()
                    } else {
                        Typ::tuple(outputs_types)
                    }
                };

                self.typing = Some(node_application_type);
                Ok(())
            }

            stream::Kind::Expression { ref mut expr } => {
                self.typing = Some(expr.typing(&self.loc, symbols, errors)?);
                Ok(())
            }

            stream::Kind::SomeEvent { ref mut expr } => {
                expr.typing(symbols, errors)?;
                let expr_type = expr.get_type().unwrap().clone();
                self.typing = Some(Typ::sm_event(expr_type));
                Ok(())
            }

            stream::Kind::NoneEvent => {
                self.typing = Some(Typ::sm_event(Typ::Any));
                Ok(())
            }
            stream::Kind::RisingEdge { ref mut expr } => {
                expr.typing(symbols, errors)?;
                // check expr is a boolean
                let expr_type = expr.get_type().unwrap().clone();
                let expected = Typ::bool();
                expr_type.eq_check(&expected, self.loc.clone(), errors)?;
                // set the type
                self.typing = Some(expected);
                Ok(())
            }
        }
    }

    fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
}

impl<E: Typing> Typing for Stmt<E> {
    // pre-condition: identifiers associated with statement is already typed
    // post-condition: expression associated with statement is typed and checked
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let Stmt { pattern, expr, loc } = self;

        pattern.typing(symbols, errors)?;
        let expected_type = pattern.typing.as_ref().unwrap();

        expr.typing(symbols, errors)?;
        let expr_type = expr.get_type().unwrap();

        expr_type.eq_check(expected_type, loc.clone(), errors)?;

        Ok(())
    }
}

impl Pattern {
    /// Tries to type the given construct.
    pub fn typing(
        &mut self,
        expected_type: &Typ,
        symbols: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
        use pattern::Kind;
        match self.kind {
            Kind::Constant { ref constant } => {
                let pattern_type = constant.get_type();
                pattern_type.eq_check(&expected_type, self.loc.clone(), errors)?;
                self.typing = Some(pattern_type);
                Ok(())
            }
            Kind::Identifier { id } => {
                symbols.set_type(id, expected_type.clone());
                self.typing = Some(expected_type.clone());
                Ok(())
            }
            Kind::Structure {
                ref id,
                ref mut fields,
            } => {
                fields
                    .iter_mut()
                    .map(|(id, optional_pattern)| {
                        let expected_type = symbols.get_type(*id).clone();
                        if let Some(pattern) = optional_pattern {
                            pattern.typing(&expected_type, symbols, errors)?;
                            // check pattern type
                            let pattern_type = pattern.get_type().unwrap();
                            pattern_type.eq_check(&expected_type, self.loc.clone(), errors)
                        } else {
                            Ok(())
                        }
                    })
                    .collect::<Vec<TRes<()>>>()
                    .into_iter()
                    .collect::<TRes<()>>()?;
                self.typing = Some(Typ::structure_str(symbols.get_name(*id), *id));
                Ok(())
            }
            Kind::Enumeration { ref enum_id, .. } => {
                self.typing = Some(Typ::enumeration_str(symbols.get_name(*enum_id), *enum_id));
                Ok(())
            }
            Kind::Tuple { ref mut elements } => match expected_type {
                Typ::Tuple {
                    elements: types, ..
                } => {
                    if elements.len() != types.len() {
                        let error = Error::IncompatibleTuple {
                            loc: self.loc.clone(),
                        };
                        errors.push(error);
                        return Err(TerminationError);
                    }
                    elements
                        .iter_mut()
                        .zip(types)
                        .map(|(pattern, expected_type)| {
                            pattern.typing(expected_type, symbols, errors)
                        })
                        .collect::<Vec<TRes<()>>>()
                        .into_iter()
                        .collect::<TRes<()>>()?;
                    let types = elements
                        .iter()
                        .map(|pattern| pattern.get_type().unwrap().clone())
                        .collect();
                    self.typing = Some(Typ::tuple(types));
                    Ok(())
                }
                _ => {
                    let error = Error::ExpectTuplePattern {
                        loc: self.loc.clone(),
                    };
                    errors.push(error);
                    Err(TerminationError)
                }
            },
            Kind::Some { ref mut pattern } => match expected_type {
                Typ::SMEvent { ty, .. } => {
                    pattern.typing(ty, symbols, errors)?;
                    let pattern_type = pattern.get_type().unwrap().clone();
                    self.typing = Some(Typ::sm_event(pattern_type));
                    Ok(())
                }
                _ => {
                    let error = Error::ExpectOptionPattern {
                        loc: self.loc.clone(),
                    };
                    errors.push(error);
                    Err(TerminationError)
                }
            },
            Kind::None => {
                self.typing = Some(Typ::sm_event(Typ::Any));
                Ok(())
            }
            Kind::Default => {
                self.typing = Some(Typ::any());
                Ok(())
            }
            Kind::PresentEvent {
                event_id,
                ref mut pattern,
            } => {
                let typing = symbols.get_type(event_id).clone();
                expected_type.eq_check(&typing, self.loc.clone(), errors)?;

                match &typing {
                    Typ::SMEvent { ty, .. } => pattern.typing(&ty, symbols, errors)?,
                    _ => unreachable!(),
                };

                self.typing = Some(typing);
                Ok(())
            }
            Kind::NoEvent { event_id } => {
                let typing = symbols.get_type(event_id).clone();
                expected_type.eq_check(&typing, self.loc.clone(), errors)?;
                self.typing = Some(typing);
                Ok(())
            }
        }
    }
}

impl stmt::Pattern {
    /// Tries to construct the type of the given construct.
    pub fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        match self.kind {
            stmt::Kind::Identifier { id } => {
                let typing = symbols.get_type(id);
                self.typing = Some(typing.clone());
                Ok(())
            }
            stmt::Kind::Typed { id, ref typing } => {
                let expected_type = symbols.get_type(id);
                typing.eq_check(expected_type, Location::default(), errors)?;
                // symbols.set_type(id, typing.clone());
                self.typing = Some(typing.clone());
                Ok(())
            }
            stmt::Kind::Tuple { ref mut elements } => {
                let types = elements
                    .iter_mut()
                    .map(|pattern| {
                        pattern.typing(symbols, errors)?;
                        Ok(pattern.typing.as_ref().unwrap().clone())
                    })
                    .collect::<Vec<TRes<_>>>()
                    .into_iter()
                    .collect::<TRes<Vec<_>>>()?;

                self.typing = Some(Typ::tuple(types));
                Ok(())
            }
        }
    }
}

impl Typing for Expr {
    fn typing(&mut self, symbols: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        self.typing = Some(self.kind.typing(&self.loc, symbols, errors)?);
        Ok(())
    }
    fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }
    fn get_type_mut(&mut self) -> Option<&mut Typ> {
        self.typing.as_mut()
    }
}

impl<E: Typing> expr::Kind<E> {
    /// Tries to type the given construct.
    fn typing(
        &mut self,
        loc: &Location,
        symbols: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        let mut typing = ExprTyping::new(loc, symbols, errors);
        match self {
            expr::Kind::Constant { constant } => Ok(constant.get_type()),
            expr::Kind::Identifier { id } => {
                let typing = symbols.get_type(*id);
                Ok(typing.clone())
            }
            expr::Kind::UnOp { op, expr } => typing.unop(op, expr.as_mut()),
            expr::Kind::BinOp { op, lft, rgt } => typing.binop(op, lft.as_mut(), rgt.as_mut()),
            expr::Kind::IfThenElse { cnd, thn, els } => {
                typing.if_then_else(cnd.as_mut(), thn.as_mut(), els.as_mut())
            }
            expr::Kind::Application { fun: f, inputs } => typing.application(f.as_mut(), inputs),
            expr::Kind::Abstraction { inputs, expr } => typing.abstraction(inputs, expr.as_mut()),
            expr::Kind::Structure { id, fields } => typing.structure(*id, fields),
            expr::Kind::Array { elements } => typing.array(elements),
            expr::Kind::Tuple { elements } => typing.tuple(elements),
            expr::Kind::Match { expr, arms } => typing.matching(expr.as_mut(), arms),
            expr::Kind::FieldAccess { expr, field } => typing.field_access(expr.as_mut(), field),
            expr::Kind::Map { expr, fun } => typing.map(expr.as_mut(), fun.as_mut()),
            expr::Kind::Fold { array, init, fun } => {
                typing.fold(array.as_mut(), init.as_mut(), fun.as_mut())
            }
            expr::Kind::Sort { expr, fun } => typing.sort(expr.as_mut(), fun.as_mut()),
            expr::Kind::Zip { arrays } => typing.zip(arrays),
            expr::Kind::TupleElementAccess {
                expr,
                element_number,
            } => typing.tuple_element_access(expr.as_mut(), *element_number),
            expr::Kind::Enumeration { enum_id, .. } => typing.enumeration(*enum_id),
        }
    }
}

struct ExprTyping<'a, E: Typing> {
    loc: &'a Location,
    table: &'a mut SymbolTable,
    errors: &'a mut Vec<Error>,
    _phantom: std::marker::PhantomData<E>,
}
impl<'a, E: Typing> ExprTyping<'a, E> {
    fn new(loc: &'a Location, table: &'a mut SymbolTable, errors: &'a mut Vec<Error>) -> Self {
        Self {
            loc,
            table,
            errors,
            _phantom: std::marker::PhantomData,
        }
    }

    fn loc(&self) -> Location {
        self.loc.clone()
    }

    fn abstraction(&mut self, inputs: &Vec<usize>, expr: &mut E) -> TRes<Typ> {
        // type the abstracted expression with the local context
        expr.typing(self.table, self.errors)?;

        // compute abstraction type
        let input_types = inputs
            .iter()
            .map(|id| self.table.get_type(*id).clone())
            .collect::<Vec<_>>();
        let abstraction_type = Typ::function(input_types, expr.get_type().unwrap().clone());

        Ok(abstraction_type)
    }

    fn application(&mut self, f: &mut E, inputs: &mut Vec<E>) -> TRes<Typ> {
        // type all inputs
        for input in inputs.iter_mut() {
            input.typing(self.table, self.errors)?;
        }

        let input_types = inputs
            .iter()
            .map(|input| input.get_type().unwrap().clone())
            .collect::<Vec<_>>();

        // type the function expression
        f.typing(self.table, self.errors)?;

        // compute the application type
        let application_type =
            f.get_type_mut()
                .unwrap()
                .apply(input_types, self.loc(), self.errors)?;

        Ok(application_type)
    }

    fn array(&mut self, elms: &mut Vec<E>) -> TRes<Typ> {
        if elms.len() == 0 {
            let error = Error::ExpectInput {
                loc: self.loc.clone(),
            };
            self.errors.push(error);
            return Err(TerminationError);
        }

        elms.iter_mut()
            .map(|element| element.typing(self.table, self.errors))
            .collect::<TRes<()>>()?;

        let first_type = elms[0].get_type().unwrap(); // todo: manage zero element error
        elms.iter()
            .map(|element| {
                let element_type = element.get_type().unwrap();
                element_type.eq_check(first_type, self.loc(), self.errors)
            })
            .collect::<TRes<()>>()?;

        let array_type = Typ::array(first_type.clone(), elms.len());

        Ok(array_type)
    }

    fn binop(&mut self, op: &BOp, lft: &mut E, rgt: &mut E) -> TRes<Typ> {
        // get expressions type
        lft.typing(self.table, self.errors)?;
        let lft_type = lft.get_type().unwrap().clone();
        rgt.typing(self.table, self.errors)?;
        let rgt_type = rgt.get_type().unwrap().clone();

        // get binop type
        let mut binop_type = op.get_type();

        binop_type.apply(vec![lft_type, rgt_type], self.loc(), self.errors)
    }

    fn enumeration(&mut self, enum_id: usize) -> TRes<Typ> {
        Ok(Typ::Enumeration {
            name: Ident::new(self.table.get_name(enum_id), Span::call_site()),
            id: enum_id,
        })
    }

    fn field_access(&mut self, expr: &mut E, field: &str) -> TRes<Typ> {
        expr.typing(self.table, self.errors)?;

        match expr.get_type().unwrap() {
            Typ::Structure { name, id } => {
                let symbol = self
                    .table
                    .get_symbol(*id)
                    .expect("there should be a symbol");
                match symbol.kind() {
                    ast::symbol::SymbolKind::Structure { fields } => {
                        let option_field_type = fields
                            .iter()
                            .filter(|id| {
                                let field_name = self.table.get_name(**id);
                                field == field_name
                            })
                            .map(|id| self.table.get_type(*id).clone())
                            .next();
                        if let Some(field_type) = option_field_type {
                            Ok(field_type)
                        } else {
                            let error = Error::UnknownField {
                                structure_name: name.to_string(),
                                field_name: field.to_string(),
                                loc: self.loc(),
                            };
                            self.errors.push(error);
                            Err(TerminationError)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            given_type => {
                let error = Error::ExpectStructure {
                    given_type: given_type.clone(),
                    loc: self.loc(),
                };
                self.errors.push(error);
                Err(TerminationError)
            }
        }
    }

    fn fold(&mut self, expr: &mut E, init: &mut E, fun: &mut E) -> TRes<Typ> {
        // type the expression
        expr.typing(self.table, self.errors)?;

        // verify it is an array
        match expr.get_type().unwrap() {
            Typ::Array {
                ty: element_type, ..
            } => {
                // type the initialization expression
                init.typing(self.table, self.errors)?;
                let initialization_type = init.get_type().unwrap();

                // type the function expression
                fun.typing(self.table, self.errors)?;
                let function_type = fun.get_type_mut().unwrap();

                // apply the function type to the type of the initialization and array's elements
                let new_type = function_type.apply(
                    vec![initialization_type.clone(), *element_type.clone()],
                    self.loc(),
                    self.errors,
                )?;

                // check the new type is equal to the initialization type
                new_type.eq_check(initialization_type, self.loc(), self.errors)?;

                Ok(new_type)
            }
            given_type => {
                let error = Error::ExpectArray {
                    given_type: given_type.clone(),
                    loc: self.loc(),
                };
                self.errors.push(error);
                Err(TerminationError)
            }
        }
    }

    fn if_then_else(&mut self, cnd: &mut E, thn: &mut E, els: &mut E) -> TRes<Typ> {
        // get expressions type
        cnd.typing(self.table, self.errors)?;
        let cnd_type = cnd.get_type().unwrap().clone();
        thn.typing(self.table, self.errors)?;
        let thn_type = thn.get_type().unwrap().clone();
        els.typing(self.table, self.errors)?;
        let els_type = els.get_type().unwrap().clone();

        // get if_then_else type
        let mut if_then_else_type = OtherOp::IfThenElse.get_type();

        if_then_else_type.apply(vec![cnd_type, thn_type, els_type], self.loc(), self.errors)
    }

    fn map(&mut self, expr: &mut E, fun: &mut E) -> TRes<Typ> {
        // type the expression
        expr.typing(self.table, self.errors)?;

        // verify it is an array
        match expr.get_type().unwrap() {
            Typ::Array {
                ty: element_type,
                size,
                bracket_token,
                semi_token,
            } => {
                // type the function expression
                fun.typing(self.table, self.errors)?;
                let function_type = fun.get_type_mut().unwrap();

                // apply the function type to the type of array's elements
                let new_element_type =
                    function_type.apply(vec![*element_type.clone()], self.loc(), self.errors)?;

                Ok(Typ::Array {
                    ty: new_element_type.into(),
                    size: size.clone(),
                    bracket_token: *bracket_token,
                    semi_token: *semi_token,
                })
            }
            given_type => {
                let error = Error::ExpectArray {
                    given_type: given_type.clone(),
                    loc: self.loc(),
                };
                self.errors.push(error);
                Err(TerminationError)
            }
        }
    }

    fn matching(
        &mut self,
        expr: &mut E,
        arms: &mut Vec<(Pattern, Option<E>, Vec<Stmt<E>>, E)>,
    ) -> TRes<Typ> {
        expr.typing(self.table, self.errors)?;

        let expr_type = expr.get_type().unwrap();

        arms.iter_mut()
            .map(
                |(pattern, optional_test_expression, body, arm_expression)| {
                    // check it matches pattern type
                    pattern.typing(expr_type, self.table, self.errors)?;

                    optional_test_expression
                        .as_mut()
                        .map_or(Ok(()), |expression| {
                            expression.typing(self.table, self.errors)?;
                            expression.get_type().unwrap().eq_check(
                                &Typ::bool(),
                                self.loc(),
                                self.errors,
                            )
                        })?;

                    // set types for every pattern
                    body.iter_mut()
                        .map(|statement| statement.pattern.typing(self.table, self.errors))
                        .collect::<TRes<()>>()?;

                    // type all equations
                    body.iter_mut()
                        .map(|statement| statement.typing(self.table, self.errors))
                        .collect::<TRes<()>>()?;

                    arm_expression.typing(self.table, self.errors)
                },
            )
            .collect::<TRes<()>>()?;

        let first_type = arms[0].3.get_type().unwrap();
        arms.iter()
            .map(|(_, _, _, arm_expression)| {
                let arm_expression_type = arm_expression.get_type().unwrap();
                arm_expression_type.eq_check(first_type, self.loc(), self.errors)
            })
            .collect::<TRes<()>>()?;

        // todo: patterns should be exhaustive
        Ok(first_type.clone())
    }

    fn sort(&mut self, expr: &mut E, fun: &mut E) -> TRes<Typ> {
        // type the expression
        expr.typing(self.table, self.errors)?;

        // verify it is an array
        match expr.get_type().unwrap() {
            Typ::Array {
                ty: element_type,
                size,
                bracket_token,
                semi_token,
            } => {
                // type the function expression
                fun.typing(self.table, self.errors)?;
                let function_type = fun.get_type_mut().unwrap();

                // check it is a sorting function: (element_type, element_type) -> int
                function_type.eq_check(
                    &Typ::function(
                        vec![*element_type.clone(), *element_type.clone()],
                        Typ::int(),
                    ),
                    self.loc(),
                    self.errors,
                )?;

                Ok(Typ::Array {
                    ty: element_type.clone(),
                    size: size.clone(),
                    bracket_token: *bracket_token,
                    semi_token: *semi_token,
                })
            }
            given_type => {
                let error = Error::ExpectArray {
                    given_type: given_type.clone(),
                    loc: self.loc(),
                };
                self.errors.push(error);
                Err(TerminationError)
            }
        }
    }

    fn structure(&mut self, id: usize, fields: &mut Vec<(usize, E)>) -> TRes<Typ> {
        // type each field and check their type
        fields
            .iter_mut()
            .map(|(id, expression)| {
                expression.typing(self.table, self.errors)?;
                let expression_type = expression.get_type().unwrap();
                let expected_type = self.table.get_type(*id);
                expression_type.eq_check(expected_type, self.loc(), self.errors)
            })
            .collect::<TRes<()>>()?;

        Ok(Typ::Structure {
            name: Ident::new(self.table.get_name(id), Span::call_site()),
            id,
        })
    }

    fn tuple_element_access(&mut self, expr: &mut E, elm: usize) -> TRes<Typ> {
        expr.typing(self.table, self.errors)?;

        match expr.get_type().unwrap() {
            Typ::Tuple { elements, .. } => {
                let option_element_type = elements.iter().nth(elm);
                if let Some(element_type) = option_element_type {
                    Ok(element_type.clone())
                } else {
                    let error = Error::IndexOutOfBounds { loc: self.loc() };
                    self.errors.push(error);
                    Err(TerminationError)
                }
            }
            given_type => {
                let error = Error::ExpectTuple {
                    given_type: given_type.clone(),
                    loc: self.loc(),
                };
                self.errors.push(error);
                Err(TerminationError)
            }
        }
    }

    fn tuple(&mut self, elms: &mut Vec<E>) -> TRes<Typ> {
        // debug_assert!(elms.len() >= 1);
        // the `when` item might create empty tuples in case there are only rising edges

        let elms_types = elms
            .iter_mut()
            .map(|element| {
                element.typing(self.table, self.errors)?;
                Ok(element.get_type().expect("should be typed").clone())
            })
            .collect::<TRes<Vec<Typ>>>()?;

        let tuple_type = Typ::tuple(elms_types);

        Ok(tuple_type)
    }

    fn unop(&mut self, op: &UOp, expr: &mut E) -> TRes<Typ> {
        // get expression type
        expr.typing(self.table, self.errors)?;
        let expr_type = expr.get_type().unwrap().clone();

        // get unop type
        let mut unop_type = op.get_type();

        unop_type.apply(vec![expr_type], self.loc(), self.errors)
    }

    fn zip(&mut self, arrays: &mut Vec<E>) -> TRes<Typ> {
        if arrays.len() == 0 {
            let error = Error::ExpectInput { loc: self.loc() };
            self.errors.push(error);
            return Err(TerminationError);
        }

        arrays
            .iter_mut()
            .map(|array| array.typing(self.table, self.errors))
            .collect::<TRes<()>>()?;

        let length = match arrays[0].get_type().unwrap() {
            Typ::Array { size: n, .. } => Ok(n),
            ty => {
                let error = Error::ExpectArray {
                    given_type: ty.clone(),
                    loc: self.loc(),
                };
                self.errors.push(error);
                Err(TerminationError)
            }
        }?;
        let tuple_types = arrays
            .iter()
            .map(|array| match array.get_type().unwrap() {
                Typ::Array { ty, size: n, .. } if n == length => Ok(*ty.clone()),
                Typ::Array { size: n, .. } => {
                    let error = Error::IncompatibleLength {
                        given_length: n.base10_parse().unwrap(),
                        expected_length: length.base10_parse().unwrap(),
                        loc: self.loc(),
                    };
                    self.errors.push(error);
                    Err(TerminationError)
                }
                ty => {
                    let error = Error::ExpectArray {
                        given_type: ty.clone(),
                        loc: self.loc(),
                    };
                    self.errors.push(error);
                    Err(TerminationError)
                }
            })
            .collect::<TRes<Vec<Typ>>>()?;

        let array_type = if tuple_types.len() > 1 {
            Typ::array(Typ::tuple(tuple_types), length.base10_parse().unwrap())
        } else {
            Typ::array(
                tuple_types.get(0).unwrap().clone(),
                length.base10_parse().unwrap(),
            )
        };

        Ok(array_type)
    }
}
