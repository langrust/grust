prelude! {}

pub mod hir_ext;

pub trait ComponentExt {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

mod component {
    prelude! {
        ast::{Component, Colon}
    }

    impl super::ComponentExt for Component {
        /// Store node's signals in symbol table.
        fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
            symbol_table.local();

            let name = self.ident.to_string();
            let period = self
                .period
                .as_ref()
                .map(|(_, literal, _)| literal.base10_parse().unwrap());
            let location = Location::default();

            // store input signals and get their ids
            let inputs = self
                .args
                .iter()
                .map(
                    |Colon {
                         left: ident,
                         right: typing,
                         ..
                     }| {
                        let name = ident.to_string();
                        let typing =
                            typing
                                .clone()
                                .hir_from_ast(&location, symbol_table, errors)?;
                        let id = symbol_table.insert_signal(
                            name,
                            Scope::Input,
                            Some(typing),
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(id)
                    },
                )
                .collect::<TRes<Vec<_>>>()?;

            // store input events as element of an "event enumeration"
            let enum_name = to_camel_case(&format!("{name}Event"));
            let element_ids = self
                .args
                .iter()
                .filter(|Colon { right: typing, .. }| typing.is_event())
                .map(
                    |Colon {
                         left: ident,
                         right: typing,
                         ..
                     }| {
                        let name = ident.to_string();
                        let typing =
                            typing
                                .clone()
                                .hir_from_ast(&location, symbol_table, errors)?;
                        let id = symbol_table.insert_event_element(
                            name,
                            enum_name.clone(),
                            typing,
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(id)
                    },
                )
                .collect::<TRes<Vec<_>>>()?;

            let event_enum = if !element_ids.is_empty() {
                // create identifier for event
                let event_name = format!("{name}_event");
                let event_id = symbol_table.insert_event(
                    event_name.clone(),
                    true,
                    location.clone(),
                    errors,
                )?;

                // create enumeration of events
                let event_enum_id = symbol_table.insert_event_enum(
                    enum_name,
                    event_id,
                    element_ids,
                    true,
                    location.clone(),
                    errors,
                )?;

                Some(event_enum_id)
            } else {
                None
            };

            // store outputs and get their ids
            let outputs = self
                .outs
                .iter()
                .map(
                    |Colon {
                         left: ident,
                         right: typing,
                         ..
                     }| {
                        let name = ident.to_string();
                        let typing =
                            typing
                                .clone()
                                .hir_from_ast(&location, symbol_table, errors)?;
                        let id = symbol_table.insert_signal(
                            name.clone(),
                            Scope::Output,
                            Some(typing),
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok((name, id))
                    },
                )
                .collect::<TRes<Vec<_>>>()?;

            // store locals and get their ids
            let locals = {
                let mut map = HashMap::with_capacity(25);
                for equation in self.equations.iter() {
                    if let Some(res) = equation.store_local_declarations(symbol_table, errors) {
                        for (key, n) in res? {
                            let prev = map.insert(key, n);
                            if prev.is_some() {
                                panic!("fatal: name-clashes detected in local declaration handler");
                            }
                        }
                    }
                }
                map.shrink_to_fit();
                map
            };

            symbol_table.global();

            let _ = symbol_table.insert_node(
                name, false, inputs, event_enum, outputs, locals, period, location, errors,
            )?;

            Ok(())
        }
    }
}

pub trait EquationExt {
    fn get_pattern(&self) -> ast::Pattern;

    fn store_local_declarations(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Option<TRes<Vec<(String, usize)>>>;

    fn store_signals(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Vec<(String, usize)>>;
}

mod equation {
    prelude! {
        ast::{
            equation::*,
            pattern::Tuple,
        },
    }

    impl super::EquationExt for Equation {
        fn get_pattern(&self) -> ast::Pattern {
            match self {
                Equation::LocalDef(declaration) => declaration.typed_pattern.clone(),
                Equation::OutputDef(instantiation) => instantiation.pattern.clone(),
                Equation::Match(Match { arms, .. }) => {
                    let Arm { equations, .. } = arms.first().unwrap();
                    let mut elements = equations
                        .iter()
                        .flat_map(|equation| equation.get_pattern().get_simple_patterns())
                        .collect::<Vec<_>>();
                    if elements.len() == 1 {
                        elements.pop().unwrap()
                    } else {
                        ast::Pattern::Tuple(Tuple { elements })
                    }
                }
                Equation::MatchWhen(MatchWhen { arms, .. }) => match arms.first().unwrap() {
                    ArmWhen::EventArmWhen(EventArmWhen { equations, .. })
                    | ArmWhen::TimeoutArmWhen(TimeoutArmWhen { equations, .. })
                    | ArmWhen::Default(DefaultArmWhen { equations, .. }) => {
                        let mut elements = equations
                            .iter()
                            .flat_map(|equation| equation.get_pattern().get_simple_patterns())
                            .collect::<Vec<_>>();
                        if elements.len() == 1 {
                            elements.pop().unwrap()
                        } else {
                            ast::Pattern::Tuple(Tuple { elements })
                        }
                    }
                },
            }
        }

        fn store_local_declarations(
            &self,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> Option<TRes<Vec<(String, usize)>>> {
            match self {
                Equation::LocalDef(declaration) => {
                    Some(declaration.typed_pattern.store(true, symbol_table, errors))
                }
                Equation::OutputDef(_) => None,
                Equation::Match(Match { arms, .. }) => {
                    arms.first().map(|Arm { equations, .. }| {
                        let local_declarations = {
                            let mut vec = Vec::with_capacity(25);
                            for eq in equations.iter() {
                                if let Some(res) = eq.store_local_declarations(symbol_table, errors)
                                {
                                    for pair in res? {
                                        vec.push(pair);
                                    }
                                }
                            }
                            vec.shrink_to_fit();
                            vec
                        };
                        Ok(local_declarations)
                    })
                }
                Equation::MatchWhen(MatchWhen { arms, .. }) => arms.first().map(|arm| match arm {
                    ArmWhen::EventArmWhen(EventArmWhen { equations, .. })
                    | ArmWhen::TimeoutArmWhen(TimeoutArmWhen { equations, .. })
                    | ArmWhen::Default(DefaultArmWhen { equations, .. }) => {
                        let local_declarations = {
                            let mut vec = Vec::with_capacity(25);
                            for eq in equations {
                                if let Some(res) = eq.store_local_declarations(symbol_table, errors)
                                {
                                    for pair in res? {
                                        vec.push(pair);
                                    }
                                }
                            }
                            vec.shrink_to_fit();
                            vec
                        };
                        Ok(local_declarations)
                    }
                }),
            }
        }

        fn store_signals(
            &self,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(String, usize)>> {
            match self {
                Equation::LocalDef(declaration) => {
                    declaration.typed_pattern.store(true, symbol_table, errors)
                }
                Equation::OutputDef(instantiation) => {
                    instantiation.pattern.store(false, symbol_table, errors)
                }
                Equation::Match(Match { arms, .. }) => {
                    arms.first().map_or(Ok(vec![]), |Arm { equations, .. }| {
                        Ok(equations
                            .iter()
                            .map(|equation| equation.store_signals(symbol_table, errors))
                            .collect::<TRes<Vec<_>>>()?
                            .into_iter()
                            .flatten()
                            .collect())
                    })
                }
                Equation::MatchWhen(MatchWhen { arms, .. }) => {
                    arms.first().map_or(Ok(vec![]), |arm| match arm {
                        ArmWhen::EventArmWhen(EventArmWhen { equations, .. })
                        | ArmWhen::TimeoutArmWhen(TimeoutArmWhen { equations, .. })
                        | ArmWhen::Default(DefaultArmWhen { equations, .. }) => Ok(equations
                            .iter()
                            .map(|equation| equation.store_signals(symbol_table, errors))
                            .collect::<TRes<Vec<_>>>()?
                            .into_iter()
                            .flatten()
                            .collect()),
                    })
                }
            }
        }
    }
}

pub trait AstExt {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

impl AstExt for Ast {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        self.items
            .iter()
            .map(|item| match item {
                crate::ast::Item::Component(component) => component.store(symbol_table, errors),
                crate::ast::Item::Function(function) => function.store(symbol_table, errors),
                crate::ast::Item::Typedef(typedef) => typedef.store(symbol_table, errors),
                crate::ast::Item::FlowStatement(_) => Ok(()),
            })
            .collect::<TRes<Vec<_>>>()?;
        Ok(())
    }
}

pub trait FunctionExt {
    /// Store function's identifiers in symbol table.
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

impl FunctionExt for ast::Function {
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        symbol_table.local();

        let location = Location::default();
        let inputs = self
            .args
            .iter()
            .map(
                |ast::Colon {
                     left: ident,
                     right: typing,
                     ..
                 }| {
                    let name = ident.to_string();
                    let typing = typing
                        .clone()
                        .hir_from_ast(&location, symbol_table, errors)?;
                    let id = symbol_table.insert_identifier(
                        name.clone(),
                        Some(typing),
                        true,
                        location.clone(),
                        errors,
                    )?;
                    Ok(id)
                },
            )
            .collect::<TRes<Vec<_>>>()?;

        symbol_table.global();

        let _ = symbol_table.insert_function(
            self.ident.to_string(),
            inputs,
            None,
            false,
            location,
            errors,
        )?;

        Ok(())
    }
}

pub trait TypExt {
    /// Get imports from type.
    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<lir::item::Import>;

    /// Get generics from type.
    fn get_generics(
        &mut self,
        identifier_creator: &mut hir::IdentifierCreator,
    ) -> Vec<(String, Typ)>;

    /// Transforms AST into HIR and check identifiers good use.
    fn hir_from_ast(
        self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ>;
}

mod typ {
    prelude! {
        ast::Typ,
        lir::item::Import,
        hir::IdentifierCreator,
    }

    impl TypExt for Typ {
        fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
            use itertools::*;
            match self {
                Typ::Any
                | Typ::Integer
                | Typ::Float
                | Typ::Boolean
                | Typ::Unit
                | Typ::Time
                | Typ::ComponentEvent => {
                    vec![]
                }
                Typ::Enumeration { name, .. } => vec![Import::Enumeration(name.clone())],
                Typ::Structure { name, .. } => vec![Import::Structure(name.clone())],
                Typ::Array(typing, _)
                | Typ::SMEvent(typing)
                | Typ::SMTimeout(typing)
                | Typ::Signal(typing)
                | Typ::Event(typing)
                | Typ::Timeout(typing) => typing.get_imports(symbol_table),
                Typ::Tuple(elements_types) => elements_types
                    .iter()
                    .flat_map(|typing| typing.get_imports(symbol_table))
                    .unique()
                    .collect(),
                Typ::Abstract(inputs_types, output_type) => {
                    let mut imports = output_type.get_imports(symbol_table);
                    let mut inputs_imports = inputs_types
                        .iter()
                        .flat_map(|typing| typing.get_imports(symbol_table))
                        .unique()
                        .collect::<Vec<_>>();
                    imports.append(&mut inputs_imports);
                    imports
                }
                Typ::NotDefinedYet(_) | Typ::Polymorphism(_) | Typ::Generic(_) => unreachable!(),
            }
        }

        /// Get generics from type.
        fn get_generics(
            &mut self,
            identifier_creator: &mut IdentifierCreator,
        ) -> Vec<(String, Typ)> {
            match self {
                Typ::Integer
                | Typ::Float
                | Typ::Boolean
                | Typ::Enumeration { .. }
                | Typ::Structure { .. }
                | Typ::Any
                | Typ::Unit
                | Typ::Time
                | Typ::ComponentEvent => vec![],
                Typ::Array(typing, _)
                | Typ::SMEvent(typing)
                | Typ::SMTimeout(typing)
                | Typ::Signal(typing)
                | Typ::Event(typing)
                | Typ::Timeout(typing) => typing.get_generics(identifier_creator),
                Typ::Abstract(inputs_types, output_type) => {
                    let mut generics = output_type.get_generics(identifier_creator);
                    let mut inputs_generics = inputs_types
                        .iter_mut()
                        .flat_map(|typing| typing.get_generics(identifier_creator))
                        .collect::<Vec<_>>();
                    generics.append(&mut inputs_generics);

                    // create fresh identifier for the generic type implementing function and add it to
                    // the generics
                    let fresh_name = identifier_creator.new_type_identifier("F");
                    generics.push((fresh_name.clone(), self.clone()));

                    // modify self to be a generic type
                    *self = Typ::Generic(fresh_name);

                    generics
                }
                Typ::Tuple(elements_types) => elements_types
                    .iter_mut()
                    .flat_map(|typing| typing.get_generics(identifier_creator))
                    .collect(),
                Typ::NotDefinedYet(_) | Typ::Polymorphism(_) | Typ::Generic(_) => unreachable!(),
            }
        }

        /// Transforms AST into HIR and check identifiers good use.
        fn hir_from_ast(
            self,
            location: &Location,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Typ> {
            // precondition: Typedefs are stored in symbol table
            // postcondition: construct a new Type without `Typ::NotDefinedYet`
            match self {
                Typ::Array(array_type, array_size) => Ok(Typ::array(
                    array_type.hir_from_ast(location, symbol_table, errors)?,
                    array_size,
                )),
                Typ::SMEvent(event_type) => Ok(Typ::sm_event(event_type.hir_from_ast(
                    location,
                    symbol_table,
                    errors,
                )?)),
                Typ::SMTimeout(timeout_type) => Ok(Typ::sm_timeout(timeout_type.hir_from_ast(
                    location,
                    symbol_table,
                    errors,
                )?)),
                Typ::Tuple(tuple_types) => Ok(Typ::tuple(
                    tuple_types
                        .into_iter()
                        .map(|element_type| element_type.hir_from_ast(location, symbol_table, errors))
                        .collect::<TRes<Vec<_>>>()?,
                )),
                Typ::NotDefinedYet(name) => symbol_table
                    .get_struct_id(&name, false, location.clone(), &mut vec![])
                    .map(|id| Typ::structure(name.clone(), id))
                    .or_else(|_| {
                        symbol_table
                            .get_enum_id(&name, false, location.clone(), &mut vec![])
                            .map(|id| Typ::enumeration(name.clone(), id))
                    })
                    .or_else(|_| {
                        let id = symbol_table.get_array_id(&name, false, location.clone(), errors)?;
                        Ok(symbol_table.get_array(id))
                    }),
                Typ::Abstract(inputs_types, output_type) => {
                    let inputs_types = inputs_types
                        .into_iter()
                        .map(|input_type| input_type.hir_from_ast(location, symbol_table, errors))
                        .collect::<TRes<Vec<_>>>()?;
                    let output_type = output_type.hir_from_ast(location, symbol_table, errors)?;
                    Ok(Typ::function(inputs_types, output_type))
                }
                Typ::Signal(signal_type) => Ok(Typ::signal(signal_type.hir_from_ast(
                    location,
                    symbol_table,
                    errors,
                )?)),
                Typ::Event(event_type) => Ok(Typ::event(event_type.hir_from_ast(
                    location,
                    symbol_table,
                    errors,
                )?)),
                Typ::Timeout(timeout_type) => Ok(Typ::timeout(timeout_type.hir_from_ast(
                    location,
                    symbol_table,
                    errors,
                )?)),
                Typ::Integer | Typ::Float | Typ::Boolean | Typ::Unit| Typ::Time => Ok(self),
                Typ::Enumeration { .. }    // no enumeration at this time: they are `NotDefinedYet`
                | Typ::Structure { .. }    // no structure at this time: they are `NotDefinedYet`
                | Typ::ComponentEvent      // users can not write `ComponentEvent` type
                | Typ::Any                 // users can not write `Any` type
                | Typ::Polymorphism(_)     // users can not write `Polymorphism` type
                | Typ::Generic(_)          // users can not write `Generic` type
                 => unreachable!(),
            }
        }
    }
}

pub trait PatternExt {
    fn store(
        &self,
        is_declaration: bool,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Vec<(String, usize)>>;

    fn get_simple_patterns(self) -> Vec<ast::Pattern>;
}

mod pattern {
    prelude! {
        ast::pattern::*,
    }

    impl super::PatternExt for Pattern {
        fn store(
            &self,
            is_declaration: bool,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<Vec<(String, usize)>> {
            let location = Location::default();

            match self {
                Pattern::Identifier(name) => {
                    if is_declaration {
                        let id = symbol_table.insert_identifier(
                            name.clone(),
                            None,
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(vec![(name.clone(), id)])
                    } else {
                        let id = symbol_table.get_identifier_id(
                            name,
                            false,
                            location.clone(),
                            errors,
                        )?;
                        let typing = symbol_table.get_type(id).clone();

                        let id = symbol_table.insert_identifier(
                            name.clone(),
                            Some(typing),
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(vec![(name.clone(), id)])
                    }
                }
                Pattern::Typed(Typed { pattern, .. }) => {
                    pattern.store(is_declaration, symbol_table, errors)
                }
                Pattern::Tuple(Tuple { elements }) => Ok(elements
                    .iter()
                    .map(|pattern| pattern.store(is_declaration, symbol_table, errors))
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Structure(Structure { fields, .. }) => Ok(fields
                    .iter()
                    .map(|(field, optional_pattern)| {
                        if let Some(pattern) = optional_pattern {
                            pattern.store(is_declaration, symbol_table, errors)
                        } else {
                            let id = symbol_table.insert_identifier(
                                field.clone(),
                                None,
                                true,
                                location.clone(),
                                errors,
                            )?;
                            Ok(vec![(field.clone(), id)])
                        }
                    })
                    .collect::<TRes<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect()),
                Pattern::Constant(_) | Pattern::Enumeration(_) | Pattern::Default => Ok(vec![]),
            }
        }

        fn get_simple_patterns(self) -> Vec<Pattern> {
            match self {
                Pattern::Identifier(_) | Pattern::Typed(_) => vec![self],
                Pattern::Tuple(Tuple { elements }) => elements
                    .into_iter()
                    .flat_map(|pattern| pattern.get_simple_patterns())
                    .collect(),
                _ => todo!(),
            }
        }
    }
}

pub trait TypedefExt {
    /// Store typedef's identifiers in symbol table.
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()>;
}

impl TypedefExt for ast::Typedef {
    /// Store typedef's identifiers in symbol table.
    fn store(&self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let location = Location::default();
        match self {
            ast::Typedef::Structure { ident, fields, .. } => {
                let id = ident.to_string();
                symbol_table.local();

                let field_ids = fields
                    .iter()
                    .map(|ast::Colon { left: ident, .. }| {
                        let field_name = ident.to_string();
                        let field_id = symbol_table.insert_identifier(
                            field_name.clone(),
                            None,
                            true,
                            location.clone(),
                            errors,
                        )?;
                        Ok(field_id)
                    })
                    .collect::<TRes<Vec<_>>>()?;

                symbol_table.global();

                let _ = symbol_table.insert_struct(
                    id.clone(),
                    field_ids.clone(),
                    false,
                    location.clone(),
                    errors,
                )?;
            }
            ast::Typedef::Enumeration {
                ident, elements, ..
            } => {
                let id = ident.to_string();
                let element_ids = elements
                    .iter()
                    .map(|element_ident| {
                        let element_name = element_ident.to_string();
                        let element_id = symbol_table.insert_enum_elem(
                            element_name.clone(),
                            id.clone(),
                            false,
                            location.clone(),
                            errors,
                        )?;
                        Ok(element_id)
                    })
                    .collect::<TRes<Vec<_>>>()?;

                let _ = symbol_table.insert_enum(
                    id.clone(),
                    element_ids.clone(),
                    false,
                    location.clone(),
                    errors,
                )?;
            }
            ast::Typedef::Array { ident, size, .. } => {
                let id = ident.to_string();
                let size = size.base10_parse().unwrap();
                let _ = symbol_table.insert_array(
                    id.clone(),
                    None,
                    size,
                    false,
                    location.clone(),
                    errors,
                )?;
            }
        }

        Ok(())
    }
}

pub trait StreamExprExt {
    fn check_is_constant(
        &self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()>;
}

mod stream_expr {
    prelude! {
        ast::{ expr::{Application, Array, Binop, IfThenElse, Structure, Tuple, Unop}, stream },
    }

    impl StreamExprExt for ast::stream::Expr {
        fn check_is_constant(
            &self,
            symbol_table: &mut SymbolTable,
            errors: &mut Vec<Error>,
        ) -> TRes<()> {
            match &self {
                // Constant by default
                stream::Expr::Constant { .. } | stream::Expr::Enumeration { .. } => Ok(()),
                // Not constant by default
                stream::Expr::TypedAbstraction { .. }
                | stream::Expr::Match { .. }
                | stream::Expr::When { .. }
                | stream::Expr::FieldAccess { .. }
                | stream::Expr::TupleElementAccess { .. }
                | stream::Expr::Map { .. }
                | stream::Expr::Fold { .. }
                | stream::Expr::Sort { .. }
                | stream::Expr::Zip { .. }
                | stream::Expr::Fby { .. } => {
                    let error = Error::ExpectConstant {
                        location: Location::default(),
                    };
                    errors.push(error);
                    Err(TerminationError)
                }
                // It depends
                stream::Expr::Identifier(id) => {
                    // check id exists
                    let id = symbol_table
                        .get_identifier_id(&id, false, Location::default(), &mut vec![])
                        .or_else(|_| {
                            symbol_table.get_function_id(&id, false, Location::default(), errors)
                        })?;
                    // check it is a function or and operator
                    if symbol_table.is_function(id) {
                        Ok(())
                    } else {
                        let error = Error::ExpectConstant {
                            location: Location::default(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
                stream::Expr::Unop(Unop { expression, .. }) => {
                    expression.check_is_constant(symbol_table, errors)
                }
                stream::Expr::Binop(Binop {
                    left_expression,
                    right_expression,
                    ..
                }) => {
                    left_expression.check_is_constant(symbol_table, errors)?;
                    right_expression.check_is_constant(symbol_table, errors)
                }
                stream::Expr::IfThenElse(IfThenElse {
                    expression,
                    true_expression,
                    false_expression,
                    ..
                }) => {
                    expression.check_is_constant(symbol_table, errors)?;
                    true_expression.check_is_constant(symbol_table, errors)?;
                    false_expression.check_is_constant(symbol_table, errors)
                }
                stream::Expr::Application(Application {
                    function_expression,
                    inputs,
                }) => {
                    function_expression.check_is_constant(symbol_table, errors)?;
                    inputs
                        .iter()
                        .map(|expression| expression.check_is_constant(symbol_table, errors))
                        .collect::<TRes<_>>()
                }
                stream::Expr::Structure(Structure { fields, .. }) => fields
                    .iter()
                    .map(|(_, expression)| expression.check_is_constant(symbol_table, errors))
                    .collect::<TRes<_>>(),
                stream::Expr::Array(Array { elements })
                | stream::Expr::Tuple(Tuple { elements }) => elements
                    .iter()
                    .map(|expression| expression.check_is_constant(symbol_table, errors))
                    .collect::<TRes<_>>(),
            }
        }
    }
}
