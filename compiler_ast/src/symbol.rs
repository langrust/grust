use strum::IntoEnumIterator;

prelude! {
    interface::FlowKind,
    operator::*,
}

/// Symbol kinds.
#[derive(Clone)]
pub enum SymbolKind {
    /// Event kind.
    Event,
    /// EventEnumeration kind.
    EventEnumeration {
        /// The event of this type.
        event_id: usize,
        /// The enumeration's elements.
        elements: Vec<usize>,
    },
    /// EventElement kind.
    EventElement {
        /// Enumeration name.
        enum_name: String,
        /// Event type.
        typing: Typ,
    },
    /// Identifier kind.
    Identifier {
        /// Identifier scope.
        scope: Scope,
        /// Identifier type.
        typing: Option<Typ>,
    },
    /// Flow kind.
    Flow {
        /// Flow path (local flows don't have path in real system).
        path: Option<syn::Path>,
        /// FLow kind.
        kind: FlowKind,
        /// Is periodic timer.
        period: bool,
        /// Is deadline timer.
        deadline: bool,
        /// Flow type.
        typing: Typ,
    },
    /// Function kind.
    Function {
        /// Inputs identifiers.
        inputs: Vec<usize>,
        /// Output type.
        output_type: Option<Typ>,
        /// Function type.
        typing: Option<Typ>,
    },
    /// Node kind.
    Node {
        /// Node's input identifiers.
        inputs: Vec<usize>,
        /// Node's event identifiers.
        event_enum: Option<usize>,
        /// Node's output identifiers.
        outputs: Vec<(String, usize)>,
        /// Node's local identifiers.
        locals: HashMap<String, usize>,
        /// Node's period of execution.
        period: Option<u64>,
    },
    /// Structure kind.
    Structure {
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<usize>,
    },
    /// Enumeration kind.
    Enumeration {
        /// The enumeration's elements.
        elements: Vec<usize>,
    },
    /// Enumeration element kind.
    EnumerationElement {
        /// Enumeration name.
        enum_name: String,
    },
    /// Array kind.
    Array {
        /// The array's type.
        array_type: Option<Typ>,
        /// The array's size.
        size: usize,
    },
}
impl PartialEq for SymbolKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier { .. }, Self::Identifier { .. }) => true,
            (Self::Function { .. }, Self::Function { .. }) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

/// Symbol from the symbol table.
#[derive(Clone)]
pub struct Symbol {
    /// Symbol kind.
    kind: SymbolKind,
    /// Symbol name.
    name: String,
}
impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name
    }
}
impl Symbol {
    /// Get symbol's kind.
    pub fn kind(&self) -> &SymbolKind {
        &self.kind
    }

    /// Get symbol's mutable kind.
    pub fn kind_mut(&mut self) -> &mut SymbolKind {
        &mut self.kind
    }

    fn hash(&self) -> SymbolKey {
        match &self.kind {
            SymbolKind::Event => SymbolKey::Event,
            SymbolKind::EventEnumeration { .. } => SymbolKey::EventEnumeration,
            SymbolKind::EventElement { .. } => SymbolKey::EventElement {
                name: self.name.clone(),
            },
            SymbolKind::Identifier { .. } => SymbolKey::Identifier {
                name: self.name.clone(),
            },
            SymbolKind::Flow { .. } => SymbolKey::Flow {
                name: self.name.clone(),
            },
            SymbolKind::Function { .. } => SymbolKey::Function {
                name: self.name.clone(),
            },
            SymbolKind::Node { .. } => SymbolKey::Node {
                name: self.name.clone(),
            },
            SymbolKind::Structure { .. } => SymbolKey::Structure {
                name: self.name.clone(),
            },
            SymbolKind::Enumeration { .. } => SymbolKey::Enumeration {
                name: self.name.clone(),
            },
            SymbolKind::EnumerationElement { enum_name } => SymbolKey::EnumerationElement {
                enum_name: enum_name.clone(),
                name: self.name.clone(),
            },
            SymbolKind::Array { .. } => SymbolKey::Array {
                name: self.name.clone(),
            },
        }
    }
}

/// Key of symbol in the context table.
#[derive(PartialEq, Eq, Hash)]
pub enum SymbolKey {
    Period,
    Deadline,
    Event,            // only one event per component
    EventEnumeration, // only one event enumeration per component
    EventElement { name: String },
    Identifier { name: String },
    Flow { name: String },
    Function { name: String },
    Node { name: String },
    Structure { name: String },
    Enumeration { name: String },
    EnumerationElement { name: String, enum_name: String },
    Array { name: String },
}
/// Context table.
pub struct Context {
    /// Current scope context.
    current: HashMap<SymbolKey, usize>,
    /// Global context.
    global_context: Option<Box<Context>>,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            current: HashMap::new(),
            global_context: None,
        }
    }
}
impl Context {
    fn new() -> Self {
        Self {
            current: HashMap::new(),
            global_context: None,
        }
    }
    fn add_symbol(&mut self, key: SymbolKey, id: usize) {
        self.current.insert(key, id);
    }
    fn contains(&self, key: &SymbolKey, local: bool) -> bool {
        let contains = self.current.contains_key(key);
        if local {
            contains
        } else {
            match &self.global_context {
                Some(context) => contains || context.contains(key, local),
                None => contains,
            }
        }
    }
    fn get_id(&self, key: &SymbolKey, local: bool) -> Option<usize> {
        let contains = self.current.get(key).cloned();
        if local {
            contains
        } else {
            contains.or_else(|| {
                self.global_context
                    .as_ref()
                    .map(|context| context.get_id(key, local))
                    .flatten()
            })
        }
    }
    fn create_local_context(self) -> Context {
        Context {
            current: HashMap::new(),
            global_context: Some(Box::new(self)),
        }
    }
    fn get_global_context(self) -> Context {
        *self.global_context.expect("there is no global context")
    }
}

/// Symbol table.
pub struct SymbolTable {
    /// Table.
    table: HashMap<usize, Symbol>,
    /// The next fresh identifier.
    fresh_id: usize,
    /// Context of known symbols.
    known_symbols: Context,
    /// Current node.
    current_node: Option<usize>,
}
impl Default for SymbolTable {
    fn default() -> Self {
        Self {
            table: HashMap::new(),
            fresh_id: 0,
            known_symbols: Default::default(),
            current_node: None,
        }
    }
}
impl SymbolTable {
    /// Create new symbol table.
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            fresh_id: 0,
            known_symbols: Context::new(),
            current_node: None,
        }
    }

    /// Initialize symbol table with builtin operators.
    pub fn initialize(&mut self) {
        // initialize with unary, binary and other operators
        UnaryOperator::iter().for_each(|op| {
            let symbol = Symbol {
                kind: SymbolKind::Function {
                    inputs: vec![],
                    output_type: None,
                    typing: Some(op.get_type()),
                },
                name: op.to_string(),
            };

            self.insert_symbol(symbol, false, Location::default(), &mut vec![])
                .expect("you should not fail");
        });
        BinaryOperator::iter().for_each(|op| {
            let symbol = Symbol {
                kind: SymbolKind::Function {
                    inputs: vec![],
                    output_type: None,
                    typing: Some(op.get_type()),
                },
                name: op.to_string(),
            };

            self.insert_symbol(symbol, false, Location::default(), &mut vec![])
                .expect("you should not fail");
        });
        OtherOperator::iter().for_each(|op| {
            let symbol = Symbol {
                kind: SymbolKind::Function {
                    inputs: vec![],
                    output_type: None,
                    typing: Some(op.get_type()),
                },
                name: op.to_string(),
            };

            self.insert_symbol(symbol, false, Location::default(), &mut vec![])
                .expect("you should not fail");
        });
    }

    /// Create local context in symbol table.
    pub fn local(&mut self) {
        let prev = std::mem::take(&mut self.known_symbols);
        self.known_symbols = prev.create_local_context();
    }

    /// Return to global context in symbol table.
    pub fn global(&mut self) {
        let prev = std::mem::take(&mut self.known_symbols);
        self.known_symbols = prev.get_global_context();
    }

    /// Set the current node identifier, just to remember it.
    pub fn enter_in_node(&mut self, node_id: usize) {
        debug_assert!(self.current_node.is_none());
        self.current_node = Some(node_id)
    }

    /// Erase the current node id.
    pub fn leave_node(&mut self) {
        self.current_node = None
    }

    /// Get the current node identifier.
    pub fn get_current_node_id(&self) -> usize {
        debug_assert!(self.current_node.is_some());
        self.current_node.expect("current node should be set")
    }

    /// Insert raw symbol in symbol table.
    fn insert_symbol(
        &mut self,
        symbol: Symbol,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let key = symbol.hash();
        if self.known_symbols.contains(&key, local) {
            let error = Error::AlreadyDefinedElement {
                name: symbol.name.clone(),
                location,
            };
            errors.push(error);
            Err(TerminationError)
        } else {
            let id = self.fresh_id;
            // update symbol table
            self.table.insert(id, symbol.clone());
            self.fresh_id += 1;
            self.known_symbols.add_symbol(key, id);
            // return symbol's id
            Ok(id)
        }
    }

    /// Insert signal in symbol table.
    pub fn insert_signal(
        &mut self,
        name: String,
        scope: Scope,
        typing: Option<Typ>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Identifier { scope, typing },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert event enumeration in symbol table.
    pub fn insert_event_enum(
        &mut self,
        name: String,
        event_id: usize,
        elements: Vec<usize>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::EventEnumeration { event_id, elements },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert event in symbol table.
    pub fn insert_event(
        &mut self,
        name: String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Event,
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert event element in symbol table.
    pub fn insert_event_element(
        &mut self,
        name: String,
        enum_name: String,
        typing: Typ,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::EventElement { enum_name, typing },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert identifier in symbol table.
    pub fn insert_identifier(
        &mut self,
        name: String,
        typing: Option<Typ>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Identifier {
                scope: Scope::Local,
                typing,
            },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert flow in symbol table.
    pub fn insert_flow(
        &mut self,
        name: String,
        path: Option<syn::Path>,
        kind: FlowKind,
        typing: Typ,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Flow {
                path,
                kind,
                period: false,
                deadline: false,
                typing,
            },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert function in symbol table.
    pub fn insert_function(
        &mut self,
        name: String,
        inputs: Vec<usize>,
        output_type: Option<Typ>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Function {
                inputs,
                output_type,
                typing: None,
            },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert node in symbol table.
    pub fn insert_node(
        &mut self,
        name: String,
        local: bool,
        inputs: Vec<usize>,
        event_enum: Option<usize>,
        outputs: Vec<(String, usize)>,
        locals: HashMap<String, usize>,
        period: Option<u64>,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Node {
                inputs,
                event_enum,
                outputs,
                locals,
                period,
            },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert structure in symbol table.
    pub fn insert_struct(
        &mut self,
        name: String,
        fields: Vec<usize>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Structure { fields },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert enumeration in symbol table.
    pub fn insert_enum(
        &mut self,
        name: String,
        elements: Vec<usize>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Enumeration { elements },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert enumeration element in symbol table.
    pub fn insert_enum_elem(
        &mut self,
        name: String,
        enum_name: String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::EnumerationElement { enum_name },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert array in symbol table.
    pub fn insert_array(
        &mut self,
        name: String,
        array_type: Option<Typ>,
        size: usize,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol {
            kind: SymbolKind::Array { array_type, size },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    /// Insert fresh signal in symbol table.
    pub fn insert_fresh_signal(
        &mut self,
        fresh_name: String,
        scope: Scope,
        typing: Option<Typ>,
    ) -> usize {
        let symbol = Symbol {
            kind: SymbolKind::Identifier { scope, typing },
            name: fresh_name,
        };

        self.insert_symbol(symbol, false, Location::default(), &mut vec![])
            .expect("you should not fail") // todo make it local
    }

    /// Insert fresh flow in symbol table.
    pub fn insert_fresh_flow(&mut self, fresh_name: String, kind: FlowKind, typing: Typ) -> usize {
        let symbol = Symbol {
            kind: SymbolKind::Flow {
                path: None,
                kind,
                period: false,
                deadline: false,
                typing,
            },
            name: fresh_name,
        };

        self.insert_symbol(symbol, false, Location::default(), &mut vec![])
            .expect("you should not fail") // todo make it local
    }

    /// Insert fresh period timer in symbol table.
    pub fn insert_fresh_period(&mut self, fresh_name: String) -> usize {
        let symbol = Symbol {
            kind: SymbolKind::Flow {
                path: None,
                kind: FlowKind::Event(Default::default()),
                period: true,
                deadline: false,
                typing: Typ::Event(Box::new(Typ::Time)),
            },
            name: fresh_name,
        };

        self.insert_symbol(symbol, false, Location::default(), &mut vec![])
            .expect("you should not fail") // todo make it local
    }

    /// Insert fresh deadline timer in symbol table.
    pub fn insert_fresh_deadline(&mut self, fresh_name: String) -> usize {
        let symbol = Symbol {
            kind: SymbolKind::Flow {
                path: None,
                kind: FlowKind::Event(Default::default()),
                period: false,
                deadline: true,
                typing: Typ::Event(Box::new(Typ::Time)),
            },
            name: fresh_name,
        };

        self.insert_symbol(symbol, false, Location::default(), &mut vec![])
            .expect("you should not fail") // todo make it local
    }

    /// Restore a local context from identifiers.
    fn restore_context_from<'a>(&mut self, ids: impl Iterator<Item = &'a usize>) {
        ids.for_each(|id| self.restore_context_from_id(*id))
    }
    fn restore_context_from_id(&mut self, id: usize) {
        let key = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"))
            .hash();
        self.known_symbols.add_symbol(key, id);
    }

    /// Restore node or function body context.
    pub fn restore_context(&mut self, id: usize) {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"))
            .clone();
        match symbol.kind() {
            SymbolKind::Function { inputs, .. } => {
                self.restore_context_from(inputs.iter());
            }
            SymbolKind::Node {
                inputs,
                event_enum,
                outputs,
                locals,
                ..
            } => {
                self.restore_context_from(inputs.iter());
                self.restore_context_from(outputs.iter().map(|(_, id)| id));
                self.restore_context_from(locals.values());
                // restore event enumeration and elements (if they exist)
                if let Some(event_enum_id) = event_enum {
                    let symbol = self
                        .get_symbol(*event_enum_id)
                        .expect(&format!("expect symbol for {event_enum_id}"))
                        .clone();
                    match symbol.kind() {
                        SymbolKind::EventEnumeration { event_id, elements } => {
                            self.restore_context_from_id(*event_id);
                            self.restore_context_from(elements.iter());
                        }
                        _ => unreachable!(),
                    }
                    self.known_symbols.add_symbol(symbol.hash(), *event_enum_id);
                }
            }
            _ => unreachable!(),
        }
    }

    /// Get symbol from identifier.
    pub fn get_symbol(&self, id: usize) -> Option<&Symbol> {
        self.table.get(&id)
    }

    /// Get mutable symbol from identifier.
    pub fn get_symbol_mut(&mut self, id: usize) -> Option<&mut Symbol> {
        self.table.get_mut(&id)
    }

    /// Get type from identifier.
    pub fn get_type(&self, id: usize) -> &Typ {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Identifier { typing, .. } => typing
                .as_ref()
                .expect(&format!("{} should be typed", symbol.name)),
            SymbolKind::Flow { typing, .. } => typing,
            SymbolKind::EventElement { typing, .. } => typing,
            SymbolKind::Function { typing, .. } => typing
                .as_ref()
                .expect(&format!("{} should be typed", symbol.name)),
            _ => unreachable!(),
        }
    }

    /// Get function output type from identifier.
    pub fn get_function_output_type(&self, id: usize) -> &Typ {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { output_type, .. } => output_type.as_ref().expect("expect type"),
            _ => unreachable!(),
        }
    }

    /// Get function input identifiers from identifier.
    pub fn get_function_input(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { inputs, .. } => inputs,
            _ => unreachable!(),
        }
    }

    /// Set function output type.
    pub fn set_function_output_type(&mut self, id: usize, new_type: Typ) {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        let inputs_type = match &symbol.kind {
            SymbolKind::Function { ref inputs, .. } => inputs
                .iter()
                .map(|id| self.get_type(*id).clone())
                .collect::<Vec<_>>(),
            _ => unreachable!(),
        };

        let symbol = self
            .get_symbol_mut(id)
            .expect(&format!("expect symbol for {id}"));
        match &mut symbol.kind {
            SymbolKind::Function {
                ref mut output_type,
                ref mut typing,
                ..
            } => {
                if output_type.is_some() {
                    panic!("a symbol type can not be modified")
                }
                *output_type = Some(new_type.clone());
                *typing = Some(Typ::Abstract(inputs_type, Box::new(new_type)))
            }
            _ => unreachable!(),
        }
    }

    /// Tell if identifier refers to function.
    pub fn is_function(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { .. } => true,
            _ => false,
        }
    }

    // /// Get unitary node output type from identifier.
    // pub fn get_unitary_node_output_type(&self, id: usize) -> &Typ {
    //     let symbol = self
    //         .get_symbol(id)
    //         .expect(&format!("expect symbol for {id}"));
    //     match symbol.kind() {
    //         SymbolKind::UnitaryNode { output, .. } => self.get_type(*output),
    //         _ => unreachable!(),
    //     }
    // }

    // /// Get unitary node output name from identifier.
    // pub fn get_unitary_node_output_name(&self, id: usize) -> &String {
    //     let symbol = self
    //         .get_symbol(id)
    //         .expect(&format!("expect symbol for {id}"));
    //     match symbol.kind() {
    //         SymbolKind::UnitaryNode { output, .. } => self.get_name(*output),
    //         _ => unreachable!(),
    //     }
    // }

    // /// Get unitary node output identifier from identifier.
    // pub fn get_unitary_node_output_id(&self, id: usize) -> usize {
    //     let symbol = self
    //         .get_symbol(id)
    //         .expect(&format!("expect symbol for {id}"));
    //     match symbol.kind() {
    //         SymbolKind::UnitaryNode { output, .. } => *output,
    //         _ => unreachable!(),
    //     }
    // }

    // /// Get unitary node hashmap of used inputs from identifier.
    // pub fn get_unitary_node_used_inputs(&self, id: usize) -> HashMap<usize, bool> {
    //     let symbol = self
    //         .get_symbol(id)
    //         .expect(&format!("expect symbol for {id}"));
    //     match symbol.kind() {
    //         SymbolKind::UnitaryNode {
    //             mother_node,
    //             inputs,
    //             ..
    //         } => {
    //             let mother_node_inputs = self.get_node_inputs(*mother_node);
    //             mother_node_inputs
    //                 .iter()
    //                 .map(|id| (*id, inputs.contains(id)))
    //                 .collect()
    //         }
    //         _ => unreachable!(),
    //     }
    // }

    // /// Get unitary node input identifiers from identifier.
    // pub fn get_unitary_node_inputs(&self, id: usize) -> &Vec<usize> {
    //     let symbol = self
    //         .get_symbol(id)
    //         .expect(&format!("expect symbol for {id}"));
    //     match symbol.kind() {
    //         SymbolKind::UnitaryNode { inputs, .. } => inputs,
    //         _ => unreachable!(),
    //     }
    // }

    /// Set identifier's type.
    pub fn set_type(&mut self, id: usize, new_type: Typ) {
        let symbol = self
            .get_symbol_mut(id)
            .expect(&format!("expect symbol for {id}"));
        match &mut symbol.kind {
            SymbolKind::Identifier { ref mut typing, .. } => {
                if typing.is_some() {
                    panic!("type of {} can not be modified", symbol.name)
                }
                *typing = Some(new_type)
            }
            _ => unreachable!(),
        }
    }

    /// Set flow's path.
    pub fn set_path(&mut self, id: usize, new_path: syn::Path) {
        let symbol = self
            .get_symbol_mut(id)
            .expect(&format!("expect symbol for {id}"));
        match &mut symbol.kind {
            SymbolKind::Flow { ref mut path, .. } => {
                if path.is_some() {
                    panic!("path of {} can not be modified", symbol.name)
                }
                *path = Some(new_path)
            }
            _ => unreachable!(),
        }
    }

    /// Get identifier's name.
    pub fn get_name(&self, id: usize) -> &String {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        &symbol.name
    }

    /// Get identifier's scope.
    pub fn get_scope(&self, id: usize) -> &Scope {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Identifier { scope, .. } => scope,
            _ => unreachable!(),
        }
    }

    /// Set identifier's scope.
    pub fn set_scope(&mut self, id: usize, new_scope: Scope) {
        let symbol = self
            .get_symbol_mut(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind {
            SymbolKind::Identifier { ref mut scope, .. } => *scope = new_scope,
            _ => unreachable!(),
        }
    }

    /// Get node input identifiers from identifier.
    pub fn get_node_inputs(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { inputs, .. } => inputs,
            _ => unreachable!(),
        }
    }

    /// Get node output identifiers from identifier.
    pub fn get_node_outputs(&self, id: usize) -> &Vec<(String, usize)> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { outputs, .. } => outputs,
            _ => unreachable!(),
        }
    }

    /// Get node event_enum from identifier.
    pub fn get_node_event_enum(&self, id: usize) -> Option<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { event_enum, .. } => *event_enum,
            _ => unreachable!(),
        }
    }

    /// Tell if the node has events.
    pub fn has_events(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { event_enum, .. } => event_enum.is_some(),
            _ => unreachable!(),
        }
    }

    /// Get node event from identifier.
    pub fn get_node_event(&self, id: usize) -> Option<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { event_enum, .. } => {
                if let Some(event_enum) = event_enum {
                    let symbol = self
                        .get_symbol(*event_enum)
                        .expect(&format!("expect symbol for {id}"));
                    match symbol.kind() {
                        SymbolKind::EventEnumeration { event_id, .. } => Some(*event_id),
                        _ => unreachable!(),
                    }
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
    }

    /// Get node period from identifier.
    pub fn get_node_period(&self, id: usize) -> Option<u64> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { period, .. } => *period,
            _ => unreachable!(),
        }
    }

    /// Get event_enum's elements from identifier.
    pub fn get_event_enum_elements(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::EventEnumeration { elements, .. } => elements,
            _ => unreachable!(),
        }
    }

    /// Get flow's kind from identifier.
    pub fn get_flow_kind(&self, id: usize) -> FlowKind {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { kind, .. } => kind.clone(),
            _ => unreachable!(),
        }
    }

    /// Tell wether the id is a deadline timer.
    pub fn is_deadline(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { deadline, .. } => *deadline,
            _ => unreachable!(),
        }
    }

    /// Tell wether the id is a periodic timer.
    pub fn is_period(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { period, .. } => *period,
            _ => unreachable!(),
        }
    }

    /// Get structure' field identifiers from identifier.
    pub fn get_struct_fields(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Structure { fields, .. } => fields,
            _ => unreachable!(),
        }
    }

    /// Get enumeration' element identifiers from identifier.
    pub fn get_enum_elements(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Enumeration { elements, .. } => elements,
            _ => unreachable!(),
        }
    }

    /// Tell if identifier is a node.
    pub fn is_node(&self, name: &String, local: bool) -> bool {
        let symbol_hash = SymbolKey::Node { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(_) => true,
            None => false,
        }
    }

    /// Set array type from identifier.
    pub fn set_array_type(&mut self, id: usize, new_type: Typ) {
        let symbol = self
            .get_symbol_mut(id)
            .expect(&format!("expect symbol for {id}"));
        match &mut symbol.kind {
            SymbolKind::Array {
                ref mut array_type, ..
            } => {
                if array_type.is_some() {
                    panic!("a symbol type can not be modified")
                }
                *array_type = Some(new_type)
            }
            _ => unreachable!(),
        }
    }

    /// Get array type from identifier.
    pub fn get_array(&self, id: usize) -> Typ {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { array_type, size } => Typ::Array(
                Box::new(array_type.as_ref().expect("expect type").clone()),
                *size,
            ),
            _ => unreachable!(),
        }
    }

    /// Get array element type from identifier.
    pub fn get_array_type(&self, id: usize) -> &Typ {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { array_type, .. } => array_type.as_ref().expect("expect type"),
            _ => unreachable!(),
        }
    }

    /// Get array size from identifier.
    pub fn get_array_size(&self, id: usize) -> usize {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { size, .. } => *size,
            _ => unreachable!(),
        }
    }

    /// Get identifier symbol identifier.
    pub fn get_identifier_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Identifier { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownElement {
                    name: name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Get function symbol identifier.
    pub fn get_function_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Function { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownElement {
                    name: name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Get signal symbol identifier.
    pub fn get_signal_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Identifier { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownSignal {
                    name: name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Get event enumeration symbol identifier.
    pub fn get_event_enumeration_id(
        &self,
        local: bool,
        _location: Location,
        _errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::EventEnumeration;
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                todo!("no events")
                // let error = <#TODO>;
                // errors.push(error);
                // Err(TerminationError)
            }
        }
    }

    /// Get event element symbol identifier.
    pub fn get_event_element_id(
        &self,
        name: &String,
        local: bool,
        _location: Location,
        _errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::EventElement { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                println!("{name}");
                todo!("unknown event")
                // let error = <#TODO>;
                // errors.push(error);
                // Err(TerminationError)
            }
        }
    }

    /// Get event symbol identifier.
    pub fn get_event_id(
        &self,
        local: bool,
        _location: Location,
        _errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Event;
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                todo!("no events")
                // let error = <#TODO>;
                // errors.push(error);
                // Err(TerminationError)
            }
        }
    }

    /// Get flow symbol identifier.
    pub fn get_flow_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Flow { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownSignal {
                    name: name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Get node symbol identifier.
    pub fn get_node_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Node { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownNode {
                    name: name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Get structure symbol identifier.
    pub fn get_struct_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Structure { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownType {
                    name: name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Get enumeration symbol identifier.
    pub fn get_enum_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Enumeration { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownType {
                    name: name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Get enumeration element symbol identifier.
    pub fn get_enum_elem_id(
        &self,
        elem_name: &String,
        enum_name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::EnumerationElement {
            enum_name: enum_name.clone(),
            name: elem_name.clone(),
        };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownElement {
                    name: elem_name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }

    /// Get array symbol identifier.
    pub fn get_array_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Array { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let error = Error::UnknownType {
                    name: name.to_string(),
                    location,
                };
                errors.push(error);
                Err(TerminationError)
            }
        }
    }
}
