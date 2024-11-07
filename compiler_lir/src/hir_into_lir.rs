prelude! {}

mod interface;

/// Turns an [`hir`] element in a [`lir`] element, implemented for [`hir`] types.
pub trait HirIntoLir<Ctx> {
    /// LIR type constructed.
    type Lir;

    /// [`hir`] to [`lir`].
    fn into_lir(self, ctx: Ctx) -> Self::Lir;
    /// Option type information.
    fn try_get_typ(&self) -> Option<&Typ> {
        None
    }
    /// True if the [`lir`] element is an if-then-else operator.
    fn is_if_then_else(&self, _symbols: &SymbolTable) -> bool {
        false
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::Component {
    type Lir = Item;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        match self {
            hir::Component::Definition(comp_def) => {
                Item::StateMachine(comp_def.into_lir(&symbol_table))
            }
            hir::Component::Import(comp_import) => {
                Item::Import(comp_import.into_lir(&symbol_table))
            }
        }
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::ComponentDefinition {
    type Lir = StateMachine;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        // get node name
        let name = symbol_table.get_name(self.id);

        // get node inputs
        let inputs = symbol_table
            .get_node_inputs(self.id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(*id).clone(),
                    symbol_table.get_typ(*id).clone(),
                )
            })
            .collect::<Vec<_>>();

        // get node output type
        let outputs = symbol_table.get_node_outputs(self.id);
        let output_type = {
            let mut types = outputs
                .iter()
                .map(|(_, output_id)| symbol_table.get_typ(*output_id).clone())
                .collect::<Vec<_>>();
            if types.len() == 1 {
                types.pop().unwrap()
            } else {
                Typ::tuple(types)
            }
        };

        // get node output expression
        let outputs = symbol_table.get_node_outputs(self.id);
        let output_expression = {
            let mut identifiers = outputs
                .iter()
                .map(|(_, output_id)| Expr::Identifier {
                    identifier: symbol_table.get_name(*output_id).clone(),
                })
                .collect::<Vec<_>>();
            if identifiers.len() == 1 {
                identifiers.pop().unwrap()
            } else {
                Expr::Tuple {
                    elements: identifiers,
                }
            }
        };

        // get memory/state elements
        let (elements, state_elements_init, state_elements_step) =
            memory_state_elements(self.memory, symbol_table);

        // transform contract
        let contract = self.contract.into_lir(symbol_table);
        let invariant_initialization = vec![]; // TODO

        use state_machine::*;

        // 'init' method
        let init = Init::new(name, state_elements_init, invariant_initialization);

        // 'step' method
        let step = Step::new(
            name,
            output_type,
            self.statements
                .into_iter()
                .map(|equation| equation.into_lir(symbol_table))
                .collect(),
            state_elements_step,
            output_expression,
            contract,
        );

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

        StateMachine::new(name, input, state)
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::ComponentImport {
    type Lir = Import;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        // get node name
        let name = symbol_table.get_name(self.id).clone();
        let path = self.path;

        Import { name, path }
    }
}

/// Get state elements from memory.
pub fn memory_state_elements(
    mem: hir::Memory,
    symbol_table: &SymbolTable,
) -> (
    Vec<state_machine::StateElmInfo>,
    Vec<state_machine::StateElmInit>,
    Vec<state_machine::StateElmStep>,
) {
    use hir::memory::*;
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
        let mem_ident = format!("last_{}", ident);
        elements.push(StateElmInfo::buffer(&mem_ident, typing));
        inits.push(StateElmInit::buffer(
            &mem_ident,
            init.into_lir(symbol_table),
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
            elements.push(StateElmInfo::called_node(memory_name, node_name));
            inits.push(StateElmInit::called_node(memory_name, node_name));
        });

    (elements, inits, steps)
}

impl HirIntoLir<&'_ SymbolTable> for hir::Contract {
    type Lir = Contract;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        Contract {
            requires: self
                .requires
                .into_iter()
                .map(|term| term.into_lir(symbol_table))
                .collect(),
            ensures: self
                .ensures
                .into_iter()
                .map(|term| term.into_lir(symbol_table))
                .collect(),
            invariant: self
                .invariant
                .into_iter()
                .map(|term| term.into_lir(symbol_table))
                .collect(),
        }
    }
}

mod term {
    prelude! {
        hir::contract::{Term, Kind},
    }

    impl HirIntoLir<&'_ SymbolTable> for Term {
        type Lir = contract::Term;

        fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
            match self.kind {
                Kind::Constant { constant } => contract::Term::literal(constant),
                Kind::Identifier { id } => {
                    let name = symbol_table.get_name(id);
                    match symbol_table.get_scope(id) {
                        Scope::Input => contract::Term::input(name),
                        Scope::Output => contract::Term::ident("result"), // todo: this will broke for components with multiple outputs
                        Scope::Local => contract::Term::ident(name),
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
                Kind::Unary { op, term } => contract::Term::unop(op, term.into_lir(symbol_table)),
                Kind::Binary { op, left, right } => contract::Term::binop(
                    op,
                    left.into_lir(symbol_table),
                    right.into_lir(symbol_table),
                ),
                Kind::ForAll { id, term } => {
                    let name = symbol_table.get_name(id);
                    let ty = symbol_table.get_typ(id).clone();
                    let term = term.into_lir(symbol_table);
                    contract::Term::forall(name, ty, term)
                }
                Kind::Implication { left, right } => contract::Term::implication(
                    left.into_lir(symbol_table),
                    right.into_lir(symbol_table),
                ),
                Kind::PresentEvent { event_id, pattern } => match symbol_table.get_typ(event_id) {
                    Typ::SMEvent { .. } => {
                        contract::Term::some(contract::Term::ident(symbol_table.get_name(pattern)))
                    }
                    _ => unreachable!(),
                },
            }
        }
    }
}

impl<'a, E> HirIntoLir<&'a SymbolTable> for hir::expr::Kind<E>
where
    E: HirIntoLir<&'a SymbolTable, Lir = Expr>,
{
    type Lir = Expr;

    fn into_lir(self, symbol_table: &'a SymbolTable) -> Self::Lir {
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
                let expr = expr.into_lir(symbol_table);
                Expr::unop(op, expr)
            }
            Self::BinOp { op, lft, rgt } => {
                let lft = lft.into_lir(symbol_table);
                let rgt = rgt.into_lir(symbol_table);
                Expr::binop(op, lft, rgt)
            }
            Self::IfThenElse { cnd, thn, els } => {
                let cnd = cnd.into_lir(symbol_table);
                let thn = thn.into_lir(symbol_table);
                let els = els.into_lir(symbol_table);
                Expr::ite(
                    cnd,
                    Block::new(vec![Stmt::ExprLast { expr: thn }]),
                    Block::new(vec![Stmt::ExprLast { expr: els }]),
                )
            }
            Self::Application { fun, inputs, .. } => {
                let arguments = inputs
                    .into_iter()
                    .map(|input| input.into_lir(symbol_table))
                    .collect();
                Expr::FunctionCall {
                    function: Box::new(fun.into_lir(symbol_table)),
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
                    inputs,
                    output,
                    body: Box::new(Expr::Block {
                        block: Block {
                            statements: vec![Stmt::ExprLast {
                                expr: expr.into_lir(symbol_table),
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
                            expr.into_lir(symbol_table),
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
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
            Self::Tuple { elements } => Expr::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
            Self::Match { expr, arms, .. } => Expr::Match {
                matched: Box::new(expr.into_lir(symbol_table)),
                arms: arms
                    .into_iter()
                    .map(|(pattern, guard, body, expr)| {
                        (
                            pattern.into_lir(symbol_table),
                            guard.map(|expr| expr.into_lir(symbol_table)),
                            if body.is_empty() {
                                expr.into_lir(symbol_table)
                            } else {
                                let mut statements = body
                                    .into_iter()
                                    .map(|statement| statement.into_lir(symbol_table))
                                    .collect::<Vec<_>>();
                                statements.push(Stmt::ExprLast {
                                    expr: expr.into_lir(symbol_table),
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
                expr: Box::new(expr.into_lir(symbol_table)),
                field: FieldIdentifier::Named(field),
            },
            Self::TupleElementAccess {
                expr,
                element_number,
                ..
            } => Expr::FieldAccess {
                expr: Box::new(expr.into_lir(symbol_table)),
                field: FieldIdentifier::Unnamed(element_number),
            },
            Self::Map { expr, fun, .. } => Expr::Map {
                mapped: Box::new(expr.into_lir(symbol_table)),
                function: Box::new(fun.into_lir(symbol_table)),
            },
            Self::Fold {
                array, init, fun, ..
            } => Expr::fold(
                array.into_lir(symbol_table),
                init.into_lir(symbol_table),
                fun.into_lir(symbol_table),
            ),
            Self::Sort { expr, fun, .. } => Expr::Sort {
                sorted: Box::new(expr.into_lir(symbol_table)),
                function: Box::new(fun.into_lir(symbol_table)),
            },
            Self::Zip { arrays, .. } => Expr::Zip {
                arrays: arrays
                    .into_iter()
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
        }
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match self {
            Self::Identifier { id, .. } => OtherOp::IfThenElse
                .to_string()
                .eq(symbol_table.get_name(*id)),
            _ => false,
        }
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::Expr {
    type Lir = Expr;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        self.kind.into_lir(symbol_table)
    }
    fn try_get_typ(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        self.kind.is_if_then_else(symbol_table)
    }
}

impl HirIntoLir<SymbolTable> for hir::File {
    type Lir = Project;

    fn into_lir(self, mut symbol_table: SymbolTable) -> Project {
        let mut items = vec![];

        let typedefs = self
            .typedefs
            .into_iter()
            .map(|typedef| typedef.into_lir(&symbol_table));
        items.extend(typedefs);

        let functions = self
            .functions
            .into_iter()
            .map(|function| function.into_lir(&symbol_table))
            .map(Item::Function);
        items.extend(functions);

        let state_machines = self
            .components
            .into_iter()
            .map(|component| component.into_lir(&symbol_table));
        items.extend(state_machines);

        let execution_machines = self.interface.into_lir(&mut symbol_table);
        items.push(Item::ExecutionMachine(execution_machines));

        Project { items }
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::Function {
    type Lir = Function;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
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
            .collect::<Vec<_>>();

        // get function output type
        let output = symbol_table.get_function_output_type(self.id).clone();

        // Transforms into LIR statements
        let mut statements = self
            .statements
            .into_iter()
            .map(|statement| statement.into_lir(symbol_table))
            .collect::<Vec<_>>();
        statements.push(Stmt::ExprLast {
            expr: self.returned.into_lir(symbol_table),
        });

        // transform contract
        let contract = self.contract.into_lir(symbol_table);

        Function {
            name,
            inputs,
            output,
            body: Block { statements },
            contract,
        }
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::Pattern {
    type Lir = Pattern;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        use hir::pattern::Kind;
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
                                |pattern| pattern.into_lir(symbol_table),
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
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
            Kind::Some { pattern } => Pattern::Some {
                pattern: Box::new(pattern.into_lir(symbol_table)),
            },
            Kind::None => Pattern::None,
            Kind::Default => Pattern::Default,
            Kind::PresentEvent { event_id, pattern } => match symbol_table.get_typ(event_id) {
                Typ::SMEvent { .. } => Pattern::some(pattern.into_lir(symbol_table)),
                _ => unreachable!(),
            },
            Kind::NoEvent { event_id } => match symbol_table.get_typ(event_id) {
                Typ::SMEvent { .. } => Pattern::none(),
                _ => unreachable!(),
            },
        }
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::stmt::Pattern {
    type Lir = Pattern;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        match self.kind {
            hir::stmt::Kind::Identifier { id } => Pattern::Identifier {
                name: symbol_table.get_name(id).clone(),
            },
            hir::stmt::Kind::Typed { id, typ } => Pattern::Typed {
                pattern: Box::new(Pattern::Identifier {
                    name: symbol_table.get_name(id).clone(),
                }),
                typ,
            },
            hir::stmt::Kind::Tuple { elements } => Pattern::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| element.into_lir(symbol_table))
                    .collect(),
            },
        }
    }
}

impl<'a, E> HirIntoLir<&'a SymbolTable> for hir::Stmt<E>
where
    E: HirIntoLir<&'a SymbolTable, Lir = Expr>,
{
    type Lir = Stmt;

    fn into_lir(self, symbol_table: &'a SymbolTable) -> Self::Lir {
        Stmt::Let {
            pattern: self.pattern.into_lir(symbol_table),
            expr: self.expr.into_lir(symbol_table),
        }
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::stream::Expr {
    type Lir = Expr;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        use hir::stream::Kind::*;
        match self.kind {
            NodeApplication {
                memory_id,
                called_node_id,
                inputs,
                ..
            } => {
                let memory_ident = symbol_table
                    .get_name(
                        memory_id.expect("should be defined in `hir::stream::Expr::memorize`"),
                    )
                    .clone();
                let name = symbol_table.get_name(called_node_id).clone();
                let input_fields = inputs
                    .into_iter()
                    .map(|(id, expression)| {
                        (
                            symbol_table.get_name(id).clone(),
                            expression.into_lir(symbol_table),
                        )
                    })
                    .collect::<Vec<_>>();
                lir::Expr::NodeCall {
                    memory_ident,
                    node_identifier: name.clone(),
                    input_name: to_camel_case(&format!("{name}Input")),
                    input_fields,
                }
            }
            Expression { expr } => expr.into_lir(symbol_table),
            SomeEvent { expr } => lir::Expr::some(expr.into_lir(symbol_table)),
            NoneEvent => lir::Expr::none(),
            FollowedBy { id, .. } => {
                let name = symbol_table.get_name(id).clone();
                lir::Expr::MemoryAccess { identifier: name }
            }
            RisingEdge { .. } => unreachable!(),
        }
    }

    fn try_get_typ(&self) -> Option<&Typ> {
        self.typ.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match &self.kind {
            hir::stream::Kind::Expression { expr } => expr.is_if_then_else(symbol_table),
            _ => false,
        }
    }
}

impl HirIntoLir<&'_ SymbolTable> for hir::Typedef {
    type Lir = Item;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        use hir::typedef::Kind;
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
            Kind::Array => Item::ArrayAlias(lir::item::ArrayAlias {
                name: symbol_table.get_name(self.id).clone(),
                array_type: symbol_table.get_array_type(self.id).clone(),
                size: symbol_table.get_array_size(self.id),
            }),
        }
    }
}
