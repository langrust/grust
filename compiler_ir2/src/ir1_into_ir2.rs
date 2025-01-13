prelude! {}

mod interface;
mod isles;
mod trigger;

pub use self::{isles::*, trigger::*};

/// Turns an [ir1] element in a [ir2] element, implemented for [ir1] types.
pub trait Ir1IntoIr2<Ctx> {
    /// [ir2] type constructed.
    type Ir2;

    /// [ir1] to [ir2].
    fn into_ir2(self, ctx: Ctx) -> Self::Ir2;
    /// Option type information.
    fn try_get_typ(&self) -> Option<&Typ> {
        None
    }
    /// True if the [ir2] element is an if-then-else operator.
    fn is_if_then_else(&self, _symbols: &SymbolTable) -> bool {
        false
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::Component {
    type Ir2 = Item;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        match self {
            ir1::Component::Definition(comp_def) => {
                Item::StateMachine(comp_def.into_ir2(&symbol_table))
            }
            ir1::Component::Import(comp_import) => {
                Item::Import(comp_import.into_ir2(&symbol_table))
            }
        }
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::ComponentDefinition {
    type Ir2 = StateMachine;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        // get node name
        let name = symbol_table.get_name(self.id);

        // get node inputs
        let inputs = symbol_table.get_node_inputs(self.id).into_iter().map(|id| {
            (
                symbol_table.get_name(*id).clone(),
                symbol_table.get_typ(*id).clone(),
            )
        });

        // get node output type
        let outputs = symbol_table.get_node_outputs(self.id);
        let output_type = {
            iter_1! {
                outputs.iter(),
                |iter| Typ::tuple(
                    iter.map(|(_, id)| symbol_table.get_typ(*id).clone()).collect()
                ),
                |just_one| symbol_table.get_typ(just_one.1).clone()
            }
        };

        // get node output expression
        let outputs = symbol_table.get_node_outputs(self.id);
        let output_expression = {
            iter_1! {
                outputs.iter(),
                |iter| Expr::Tuple {
                    elements: iter.map(|(_, output_id)| Expr::Identifier {
                        identifier: symbol_table.get_name(*output_id).clone(),
                    }).collect()
                },
                |just_one| Expr::Identifier {
                    identifier: symbol_table.get_name(just_one.1).clone(),
                },
            }
        };

        // get memory/state elements
        let (elements, state_elements_init, state_elements_step) =
            memory_state_elements(self.memory, symbol_table);

        // transform contract
        let contract = self.contract.into_ir2(symbol_table);
        let invariant_initialization = vec![]; // TODO

        use state_machine::*;

        // 'init' method
        let init = Init::new(name.clone(), state_elements_init, invariant_initialization);

        // 'step' method
        let step = {
            let body = match para::Stmts::of_ir1(&self.statements, symbol_table, &self.graph) {
                Ok(stmts) => stmts,
                Err(e) => panic!(
                    "failed to generate (step) synced body of component `{}`:\n{}",
                    symbol_table.get_name(self.id),
                    e
                ),
            };
            Step::new(
                name.clone(),
                output_type,
                body,
                state_elements_step,
                output_expression,
                contract,
            )
        };

        // 'input' structure
        let input = Input {
            node_name: name.clone(),
            elements: inputs
                .into_iter()
                .map(|(identifier, typ)| InputElm::new(identifier, typ))
                .collect(),
        };

        // 'state' structure
        let state = State {
            node_name: name.clone(),
            elements,
            step,
            init,
        };

        StateMachine::new(name.clone(), input, state)
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::ComponentImport {
    type Ir2 = Import;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        // get node name
        let name = symbol_table.get_name(self.id).clone();
        let path = self.path;

        Import {
            name: name.clone(),
            path,
        }
    }
}

/// Get state elements from memory.
pub fn memory_state_elements(
    mem: ir1::Memory,
    symbol_table: &SymbolTable,
) -> (
    Vec<state_machine::StateElmInfo>,
    Vec<state_machine::StateElmInit>,
    Vec<state_machine::StateElmStep>,
) {
    use ir1::memory::*;
    use itertools::Itertools;
    use state_machine::{StateElmInfo, StateElmInit, StateElmStep};

    let (mut elements, mut inits, mut steps) = (vec![], vec![], vec![]);
    for (
        _,
        Buffer {
            ident,
            typing,
            init,
            id,
            ..
        },
    ) in mem.buffers.into_iter().sorted_by_key(|(id, _)| id.clone())
    {
        let scope = symbol_table.get_scope(id);
        let mem_ident = Ident::new(&format!("last_{}", ident), ident.loc().into());
        elements.push(StateElmInfo::buffer(mem_ident.clone(), typing));
        inits.push(StateElmInit::buffer(
            mem_ident.clone(),
            init.into_ir2(symbol_table),
        ));
        steps.push(StateElmStep::new(
            mem_ident,
            match scope {
                Scope::Input => Expr::input_access(ident),
                Scope::Output | Scope::Local => Expr::ident(ident),
                Scope::VeryLocal => unreachable!(),
            },
        ))
    }
    mem.called_nodes
        .into_iter()
        .sorted_by_key(|(id, _)| *id)
        .for_each(|(memory_id, CalledNode { node_id, .. })| {
            let memory_name = symbol_table.get_name(memory_id);
            let node_name = symbol_table.get_name(node_id);
            elements.push(StateElmInfo::called_node(
                memory_name.clone(),
                node_name.clone(),
            ));
            inits.push(StateElmInit::called_node(
                memory_name.clone(),
                node_name.clone(),
            ));
        });

    (elements, inits, steps)
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::Contract {
    type Ir2 = Contract;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        Contract {
            requires: self
                .requires
                .into_iter()
                .map(|term| term.into_ir2(symbol_table))
                .collect(),
            ensures: self
                .ensures
                .into_iter()
                .map(|term| term.into_ir2(symbol_table))
                .collect(),
            invariant: self
                .invariant
                .into_iter()
                .map(|term| term.into_ir2(symbol_table))
                .collect(),
        }
    }
}

mod term {
    prelude! {
        ir1::contract::{Term, Kind},
    }

    impl Ir1IntoIr2<&'_ SymbolTable> for Term {
        type Ir2 = contract::Term;

        fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
            match self.kind {
                Kind::Constant { constant } => contract::Term::literal(constant),
                Kind::Identifier { id } => {
                    let name = symbol_table.get_name(id);
                    match symbol_table.get_scope(id) {
                        Scope::Input => contract::Term::input(name.clone()),
                        // todo: this will broke for components with multiple outputs
                        Scope::Output => {
                            contract::Term::ident(Ident::new("result", name.loc().into()))
                        }
                        Scope::Local => contract::Term::ident(name.clone()),
                        Scope::VeryLocal => unreachable!("you should not do that with this ident"),
                    }
                }
                Kind::Enumeration {
                    enum_id,
                    element_id,
                } => contract::Term::enumeration(
                    symbol_table.get_name(enum_id).clone(),
                    symbol_table.get_name(element_id).clone(),
                    None,
                ),
                Kind::Unary { op, term } => contract::Term::unop(op, term.into_ir2(symbol_table)),
                Kind::Binary { op, left, right } => contract::Term::binop(
                    op,
                    left.into_ir2(symbol_table),
                    right.into_ir2(symbol_table),
                ),
                Kind::ForAll { id, term } => {
                    let name = symbol_table.get_name(id);
                    let ty = symbol_table.get_typ(id).clone();
                    let term = term.into_ir2(symbol_table);
                    contract::Term::forall(name.clone(), ty, term)
                }
                Kind::Implication { left, right } => contract::Term::implication(
                    left.into_ir2(symbol_table),
                    right.into_ir2(symbol_table),
                ),
                Kind::PresentEvent { event_id, pattern } => match symbol_table.get_typ(event_id) {
                    Typ::SMEvent { .. } => contract::Term::some(contract::Term::ident(
                        symbol_table.get_name(pattern).clone(),
                    )),
                    _ => unreachable!(),
                },
            }
        }
    }
}

impl<'a, E> Ir1IntoIr2<&'a SymbolTable> for ir1::expr::Kind<E>
where
    E: Ir1IntoIr2<&'a SymbolTable, Ir2 = Expr>,
{
    type Ir2 = Expr;

    fn into_ir2(self, symbol_table: &'a SymbolTable) -> Self::Ir2 {
        match self {
            Self::Constant { constant, .. } => Expr::Literal { literal: constant },
            Self::Identifier { id, .. } => {
                let name = symbol_table.get_name(id).clone();
                if symbol_table.is_function(id) {
                    Expr::Identifier { identifier: name }
                } else {
                    let scope = symbol_table.get_scope(id);
                    match scope {
                        Scope::Input => Expr::InputAccess { identifier: name },
                        Scope::Output | Scope::Local | Scope::VeryLocal => {
                            Expr::Identifier { identifier: name }
                        }
                    }
                }
            }
            Self::UnOp { op, expr } => {
                let expr = expr.into_ir2(symbol_table);
                Expr::unop(op, expr)
            }
            Self::BinOp { op, lft, rgt } => {
                let lft = lft.into_ir2(symbol_table);
                let rgt = rgt.into_ir2(symbol_table);
                Expr::binop(op, lft, rgt)
            }
            Self::IfThenElse { cnd, thn, els } => {
                let cnd = cnd.into_ir2(symbol_table);
                let thn = thn.into_ir2(symbol_table);
                let els = els.into_ir2(symbol_table);
                Expr::ite(
                    cnd,
                    Block::new(vec![Stmt::ExprLast { expr: thn }]),
                    Block::new(vec![Stmt::ExprLast { expr: els }]),
                )
            }
            Self::Application { fun, inputs, .. } => {
                let arguments = inputs
                    .into_iter()
                    .map(|input| input.into_ir2(symbol_table))
                    .collect();
                Expr::FunctionCall {
                    function: Box::new(fun.into_ir2(symbol_table)),
                    arguments,
                }
            }
            Self::Abstraction { inputs, expr, .. } => {
                let inputs = inputs
                    .iter()
                    .map(|id| {
                        (
                            symbol_table.get_name(*id).clone(),
                            symbol_table.get_typ(*id).clone(),
                        )
                    })
                    .collect();
                let output = expr.try_get_typ().expect("it should be typed").clone();
                Expr::Lambda {
                    is_move: true,
                    inputs,
                    output,
                    body: Box::new(Expr::Block {
                        block: Block {
                            statements: vec![Stmt::ExprLast {
                                expr: expr.into_ir2(symbol_table),
                            }],
                        },
                    }),
                }
            }
            Self::Structure { id, fields, .. } => Expr::Structure {
                name: symbol_table.get_name(id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, expr)| {
                        (
                            symbol_table.get_name(id).clone(),
                            expr.into_ir2(symbol_table),
                        )
                    })
                    .collect(),
            },
            Self::Enumeration { enum_id, elem_id } => Expr::Enumeration {
                name: symbol_table.get_name(enum_id).clone(),
                element: symbol_table.get_name(elem_id).clone(),
            },
            Self::Array { elements } => Expr::Array {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_ir2(symbol_table))
                    .collect(),
            },
            Self::Tuple { elements } => Expr::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_ir2(symbol_table))
                    .collect(),
            },
            Self::Match { expr, arms, .. } => Expr::Match {
                matched: Box::new(expr.into_ir2(symbol_table)),
                arms: arms
                    .into_iter()
                    .map(|(pattern, guard, body, expr)| {
                        (
                            pattern.into_ir2(symbol_table),
                            guard.map(|expr| expr.into_ir2(symbol_table)),
                            if body.is_empty() {
                                expr.into_ir2(symbol_table)
                            } else {
                                let mut statements = body
                                    .into_iter()
                                    .map(|statement| statement.into_ir2(symbol_table))
                                    .collect_vec();
                                statements.push(Stmt::ExprLast {
                                    expr: expr.into_ir2(symbol_table),
                                });
                                Expr::Block {
                                    block: Block { statements },
                                }
                            },
                        )
                    })
                    .collect(),
            },
            Self::FieldAccess { expr, field, .. } => Expr::FieldAccess {
                expr: Box::new(expr.into_ir2(symbol_table)),
                field: FieldIdentifier::Named(field),
            },
            Self::TupleElementAccess {
                expr,
                element_number,
                ..
            } => Expr::FieldAccess {
                expr: Box::new(expr.into_ir2(symbol_table)),
                field: FieldIdentifier::Unnamed(element_number),
            },
            Self::Map { expr, fun, .. } => Expr::Map {
                mapped: Box::new(expr.into_ir2(symbol_table)),
                function: Box::new(fun.into_ir2(symbol_table)),
            },
            Self::Fold {
                array, init, fun, ..
            } => Expr::fold(
                array.into_ir2(symbol_table),
                init.into_ir2(symbol_table),
                fun.into_ir2(symbol_table),
            ),
            Self::Sort { expr, fun, .. } => Expr::Sort {
                sorted: Box::new(expr.into_ir2(symbol_table)),
                function: Box::new(fun.into_ir2(symbol_table)),
            },
            Self::Zip { arrays, .. } => Expr::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|element| element.into_ir2(symbol_table))
                    .collect(),
            },
        }
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match self {
            Self::Identifier { id, .. } => OtherOp::IfThenElse
                .to_string()
                .eq(&symbol_table.get_name(*id).to_string()),
            _ => false,
        }
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::Expr {
    type Ir2 = Expr;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        self.kind.into_ir2(symbol_table)
    }
    fn try_get_typ(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        self.kind.is_if_then_else(symbol_table)
    }
}

impl Ir1IntoIr2<SymbolTable> for ir1::File {
    type Ir2 = Project;

    fn into_ir2(self, mut symbol_table: SymbolTable) -> Project {
        let mut items = vec![];

        let typedefs = self
            .typedefs
            .into_iter()
            .map(|typedef| typedef.into_ir2(&symbol_table));
        items.extend(typedefs);

        let functions = self
            .functions
            .into_iter()
            .map(|function| function.into_ir2(&symbol_table))
            .map(Item::Function);
        items.extend(functions);

        let state_machines = self
            .components
            .into_iter()
            .map(|component| component.into_ir2(&symbol_table));
        items.extend(state_machines);

        let execution_machines = self.interface.into_ir2(&mut symbol_table);
        items.push(Item::ExecutionMachine(execution_machines));

        Project { items }
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::Function {
    type Ir2 = Function;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        // get function name
        let name = symbol_table.get_name(self.id).clone();

        // get function inputs
        let inputs = symbol_table
            .get_function_input(self.id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(*id).clone(),
                    symbol_table.get_typ(*id).clone(),
                )
            })
            .collect_vec();

        // get function output type
        let output = symbol_table.get_function_output_type(self.id).clone();

        // Transforms into [ir2] statements
        let mut statements = self
            .statements
            .into_iter()
            .map(|statement| statement.into_ir2(symbol_table))
            .collect_vec();
        statements.push(Stmt::ExprLast {
            expr: self.returned.into_ir2(symbol_table),
        });

        // transform contract
        let contract = self.contract.into_ir2(symbol_table);

        Function {
            name,
            inputs,
            output,
            body: Block { statements },
            contract,
        }
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::Pattern {
    type Ir2 = Pattern;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        use ir1::pattern::Kind;
        match self.kind {
            Kind::Identifier { id } => Pattern::Identifier {
                name: symbol_table.get_name(id).clone(),
            },
            Kind::Constant { constant } => Pattern::Literal { literal: constant },
            Kind::Structure { id, fields } => Pattern::Structure {
                name: symbol_table.get_name(id).clone(),
                fields: fields
                    .into_iter()
                    .map(|(id, optional_pattern)| {
                        (
                            symbol_table.get_name(id).clone(),
                            optional_pattern.map_or(
                                Pattern::Identifier {
                                    name: symbol_table.get_name(id).clone(),
                                },
                                |pattern| pattern.into_ir2(symbol_table),
                            ),
                        )
                    })
                    .collect(),
            },
            Kind::Enumeration { enum_id, elem_id } => Pattern::Enumeration {
                enum_name: symbol_table.get_name(enum_id).clone(),
                elem_name: symbol_table.get_name(elem_id).clone(),
                element: None,
            },
            Kind::Tuple { elements } => Pattern::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_ir2(symbol_table))
                    .collect(),
            },
            Kind::Some { pattern } => Pattern::Some {
                pattern: Box::new(pattern.into_ir2(symbol_table)),
            },
            Kind::None => Pattern::None,
            Kind::Default(loc) => Pattern::Default(loc),
            Kind::PresentEvent { event_id, pattern } => match symbol_table.get_typ(event_id) {
                Typ::SMEvent { .. } => Pattern::some(pattern.into_ir2(symbol_table)),
                _ => unreachable!(),
            },
            Kind::NoEvent { event_id } => match symbol_table.get_typ(event_id) {
                Typ::SMEvent { .. } => Pattern::none(),
                _ => unreachable!(),
            },
        }
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::stmt::Pattern {
    type Ir2 = Pattern;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        match self.kind {
            ir1::stmt::Kind::Identifier { id } => Pattern::Identifier {
                name: symbol_table.get_name(id).clone(),
            },
            ir1::stmt::Kind::Typed { id, typ } => Pattern::Typed {
                pattern: Box::new(Pattern::Identifier {
                    name: symbol_table.get_name(id).clone(),
                }),
                typ,
            },
            ir1::stmt::Kind::Tuple { elements } => Pattern::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_ir2(symbol_table))
                    .collect(),
            },
        }
    }
}

impl<'a, E> Ir1IntoIr2<&'a SymbolTable> for ir1::Stmt<E>
where
    E: Ir1IntoIr2<&'a SymbolTable, Ir2 = Expr>,
{
    type Ir2 = Stmt;

    fn into_ir2(self, symbol_table: &'a SymbolTable) -> Self::Ir2 {
        Stmt::Let {
            pattern: self.pattern.into_ir2(symbol_table),
            expr: self.expr.into_ir2(symbol_table),
        }
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::stream::Expr {
    type Ir2 = Expr;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        use ir1::stream::Kind::*;
        match self.kind {
            NodeApplication {
                memory_id,
                called_node_id,
                inputs,
                ..
            } => {
                let memory_ident = symbol_table
                    .get_name(
                        memory_id.expect("should be defined in `ir1::stream::Expr::memorize`"),
                    )
                    .clone();
                let name = symbol_table.get_name(called_node_id).clone();
                let input_fields = inputs
                    .into_iter()
                    .map(|(id, expression)| {
                        (
                            symbol_table.get_name(id).clone(),
                            expression.into_ir2(symbol_table),
                        )
                    })
                    .collect_vec();
                let input_name = {
                    Ident::new(
                        &to_camel_case(&format!("{}Input", name.to_string())),
                        name.span(),
                    )
                };
                ir2::Expr::NodeCall {
                    memory_ident,
                    node_identifier: name,
                    input_name,
                    input_fields,
                }
            }
            Expression { expr } => expr.into_ir2(symbol_table),
            SomeEvent { expr } => ir2::Expr::some(expr.into_ir2(symbol_table)),
            NoneEvent => ir2::Expr::none(),
            FollowedBy { id, .. } => {
                let name = symbol_table.get_name(id).clone();
                ir2::Expr::MemoryAccess { identifier: name }
            }
            RisingEdge { .. } => unreachable!(),
        }
    }

    fn try_get_typ(&self) -> Option<&Typ> {
        self.typ.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match &self.kind {
            ir1::stream::Kind::Expression { expr } => expr.is_if_then_else(symbol_table),
            _ => false,
        }
    }
}

impl Ir1IntoIr2<&'_ SymbolTable> for ir1::Typedef {
    type Ir2 = Item;

    fn into_ir2(self, symbol_table: &SymbolTable) -> Self::Ir2 {
        use ir1::typedef::Kind;
        match self.kind {
            Kind::Structure { fields, .. } => Item::Structure(Structure {
                name: symbol_table.get_name(self.id).clone(),
                fields: fields
                    .into_iter()
                    .map(|id| {
                        (
                            symbol_table.get_name(id).clone(),
                            symbol_table.get_typ(id).clone(),
                        )
                    })
                    .collect(),
            }),
            Kind::Enumeration { elements, .. } => Item::Enumeration(Enumeration {
                name: symbol_table.get_name(self.id).clone(),
                elements: elements
                    .into_iter()
                    .map(|id| symbol_table.get_name(id).clone())
                    .collect(),
            }),
            Kind::Array => Item::ArrayAlias(ir2::item::ArrayAlias {
                name: symbol_table.get_name(self.id).clone(),
                array_type: symbol_table.get_array_type(self.id).clone(),
                size: symbol_table.get_array_size(self.id),
            }),
        }
    }
}
