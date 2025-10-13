prelude! {
    interface::FlowKind,
}

#[derive(Clone)]
pub enum TimerKind {
    Period(u64),
    Deadline(u64),
    ServiceTimeout(usize, u64),
    ServiceDelay(usize, u64),
}

/// Symbol kinds.
#[derive(Clone)]
pub enum SymbolKind {
    /// Identifier kind.
    Identifier {
        /// Identifier scope.
        scope: Scope,
        /// Identifier type.
        typing: Option<Typ>,
        /// Constant value.
        constant: Option<Expr>,
    },
    /// Initialization kind.
    Init {
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
        /// Is timer.
        timer: Option<TimerKind>,
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
        /// Path to rewrite calls to this function with.
        path_opt: Option<syn::Path>,
        /// A weight hint, typically provided by users.
        ///
        /// **This value is understood as a weight percentage, whatever it means for users.** This
        /// value **can** go over `100%`.
        weight_percent_hint: Option<usize>,
    },
    /// Node kind.
    Node {
        /// Node's input identifiers.
        inputs: Vec<usize>,
        /// Node's output identifiers.
        outputs: Vec<(Ident, usize)>,
        /// Node's local identifiers.
        locals: Option<HashMap<Ident, usize>>,
        /// Node's initialized identifiers.
        inits: Option<HashMap<Ident, usize>>,
        /// Path to call component from.
        path_opt: Option<syn::Path>,
        /// A weight hint, typically provided by users.
        ///
        /// **This value is understood as a weight percentage, whatever it means for users.** This
        /// value **can** go over `100%`.
        weight_percent_hint: Option<usize>,
    },
    /// Service kind.
    Service,
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
        enum_name: Ident,
    },
    /// Array kind.
    Array {
        /// The array's type.
        array_type: Option<Typ>,
        /// The array's size.
        size: usize,
    },
}
impl SymbolKind {
    pub fn scope(&self) -> Option<&Scope> {
        match self {
            Self::Identifier { scope, .. } => Some(scope),
            _ => None,
        }
    }
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
    name: Ident,
    /// String version.
    name_string: String,
}
impl Symbol {
    pub fn new(kind: SymbolKind, name: Ident) -> Self {
        let name_string = name.to_string();
        Self {
            kind,
            name,
            name_string,
        }
    }
}
impl HasLoc for Symbol {
    fn loc(&self) -> Loc {
        self.name.loc()
    }
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
    pub fn name(&self) -> &Ident {
        &self.name
    }

    /// Get symbol's mutable kind.
    pub fn kind_mut(&mut self) -> &mut SymbolKind {
        &mut self.kind
    }

    fn hash(&self) -> SymbolKey {
        match &self.kind {
            SymbolKind::Identifier { .. } => SymbolKey::Identifier {
                name: self.name.clone(),
            },
            SymbolKind::Init { .. } => SymbolKey::Init {
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
            SymbolKind::Service => SymbolKey::Service {
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
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SymbolKey {
    Period,
    Deadline,
    Identifier { name: Ident },
    Init { name: Ident },
    Flow { name: Ident },
    Function { name: Ident },
    Node { name: Ident },
    Structure { name: Ident },
    Service { name: Ident },
    Enumeration { name: Ident },
    EnumerationElement { name: Ident, enum_name: Ident },
    Array { name: Ident },
    ExtFun { name: Ident },
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
        let _unique = self.current.insert(key, id);
        debug_assert!(_unique.is_none());
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
                    .and_then(|context| context.get_id(key, local))
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
        *self
            .global_context
            .expect("internal error: there is no global context")
    }
}

/// Symbol table.
pub struct Table {
    /// Table.
    table: HashMap<usize, Symbol>,
    /// The next fresh identifier.
    fresh_id: usize,
    /// Context of known symbols.
    known_symbols: Context,
}
impl Default for Table {
    fn default() -> Self {
        Self {
            table: HashMap::new(),
            fresh_id: 0,
            known_symbols: Default::default(),
        }
    }
}
impl Table {
    /// Create new symbol table.
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            fresh_id: 0,
            known_symbols: Context::new(),
        }
    }

    pub fn count_events(&self) -> usize {
        self.table
            .values()
            .filter(|symbol| {
                matches!(
                    symbol.kind(),
                    SymbolKind::Identifier {
                        typing: Some(Typ::Option { .. }),
                        ..
                    }
                )
            })
            .count()
    }

    pub fn levenshtein_closest(&self, name: impl AsRef<str>, at_least: usize) -> Option<&Symbol> {
        let name = name.as_ref();
        let min = at_least + 1;
        let mut symbol_opt: Option<&Symbol> = None;
        for symbol in self.table.values() {
            let distance = levenshtein(name, &symbol.name_string);
            if distance < min {
                symbol_opt = Some(symbol);
            }
        }
        symbol_opt
    }

    pub fn unknown_ident_error<T>(&self, id: &Ident, levenshtein: bool) -> Res<T> {
        let str = id.to_string();
        let max_levenshtein_distance = 2;
        let e = error!(@id.loc() => ErrorKind::unknown_ident(str.clone()));
        if levenshtein {
            if let Some(symbol) = self.levenshtein_closest(&str, max_levenshtein_distance) {
                return Err(e
                    .add_note(note!("did you mean `{}`?", symbol.name_string))
                    .add_note(note!(@symbol.name.loc() => "declared here")));
            }
        }
        Err(e)
    }

    pub fn unknown_init_error<T>(&self, id: &Ident, levenshtein: bool) -> Res<T> {
        let str = id.to_string();
        let max_levenshtein_distance = 2;
        let e = error!(@id.loc() => ErrorKind::unknown_init(str.clone()));
        if levenshtein {
            if let Some(symbol) = self.levenshtein_closest(&str, max_levenshtein_distance) {
                return Err(e
                    .add_note(note!("did you mean `{}`?", symbol.name_string))
                    .add_note(note!(@symbol.name.loc() => "declared here")));
            }
        }
        Err(e)
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

    /// Insert raw symbol in symbol table.
    fn insert_symbol(
        &mut self,
        symbol: Symbol,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let key = symbol.hash();
        let loc = symbol.loc();
        // if loc == Loc::builtin() {
        //     bad!(errors, @loc => "inserting symbol `{}` with builtin location", symbol.name)
        // }
        if self.known_symbols.contains(&key, local) {
            bad!(errors, @loc => ErrorKind::elm_redef(symbol.name.to_string()))
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

    /// Get a fresh id.
    pub fn get_fresh_id(&mut self) -> usize {
        let id = self.fresh_id;
        // update symbol table
        self.fresh_id += 1;
        id
    }

    /// Insert signal in symbol table.
    pub fn insert_signal(
        &mut self,
        name: Ident,
        scope: Scope,
        typing: Option<Typ>,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(
            SymbolKind::Identifier {
                scope,
                typing,
                constant: None,
            },
            name,
        );

        self.insert_symbol(symbol, local, errors)
    }

    /// Insert identifier in symbol table.
    pub fn insert_identifier(
        &mut self,
        name: Ident,
        typing: Option<Typ>,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(
            SymbolKind::Identifier {
                scope: Scope::Local,
                typing,
                constant: None,
            },
            name,
        );

        self.insert_symbol(symbol, local, errors)
    }

    /// Insert constant identifier in symbol table.
    pub fn insert_constant(
        &mut self,
        name: Ident,
        typing: Typ,
        constant: Expr,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(
            SymbolKind::Identifier {
                scope: Scope::Local,
                typing: Some(typing),
                constant: Some(constant),
            },
            name,
        );

        self.insert_symbol(symbol, false, errors)
    }

    /// Insert identifier in symbol table.
    pub fn insert_init(
        &mut self,
        name: Ident,
        typing: Option<Typ>,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(
            SymbolKind::Init {
                scope: Scope::Local,
                typing,
            },
            name,
        );

        self.insert_symbol(symbol, local, errors)
    }

    /// Insert function result in symbol table.
    pub fn insert_function_result(
        &mut self,
        typing: Typ,
        local: bool,
        loc: Loc,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(
            SymbolKind::Identifier {
                scope: Scope::Output,
                typing: Some(typing),
                constant: None,
            },
            Ident::result(loc.span),
        );

        self.insert_symbol(symbol, local, errors)
    }

    /// Insert flow in symbol table.
    pub fn insert_flow(
        &mut self,
        name: Ident,
        path: Option<syn::Path>,
        kind: FlowKind,
        typing: Typ,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(
            SymbolKind::Flow {
                path,
                kind,
                timer: None,
                typing,
            },
            name,
        );

        self.insert_symbol(symbol, local, errors)
    }

    /// Insert function in symbol table.
    pub fn insert_function(
        &mut self,
        name: Ident,
        inputs: Vec<usize>,
        path_opt: Option<syn::Path>,
        weight_percent_hint: Option<usize>,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(
            SymbolKind::Function {
                inputs,
                output_type: None,
                typing: None,
                path_opt,
                weight_percent_hint,
            },
            name,
        );

        self.insert_symbol(symbol, false, errors)
    }

    /// Insert node in symbol table.
    pub fn insert_node(
        &mut self,
        name: Ident,
        inputs_outputs: (Vec<usize>, Vec<(Ident, usize)>),
        locals_inits: Option<(HashMap<Ident, usize>, HashMap<Ident, usize>)>,
        path_opt: Option<syn::Path>,
        weight_percent_hint: Option<usize>,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let (inputs, outputs) = inputs_outputs;
        let (locals, inits) = locals_inits.unzip();
        let symbol = Symbol::new(
            SymbolKind::Node {
                inputs,
                outputs,
                locals,
                inits,
                path_opt,
                weight_percent_hint,
            },
            name,
        );

        self.insert_symbol(symbol, false, errors)
    }

    /// Insert service in symbol table.
    pub fn insert_service(
        &mut self,
        name: Ident,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(SymbolKind::Service, name);

        self.insert_symbol(symbol, local, errors)
    }

    /// Insert structure in symbol table.
    pub fn insert_struct(
        &mut self,
        name: Ident,
        fields: Vec<usize>,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(SymbolKind::Structure { fields }, name);
        self.insert_symbol(symbol, local, errors)
    }

    /// Insert enumeration in symbol table.
    pub fn insert_enum(
        &mut self,
        name: Ident,
        elements: Vec<usize>,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(SymbolKind::Enumeration { elements }, name);
        self.insert_symbol(symbol, local, errors)
    }

    /// Insert enumeration element in symbol table.
    pub fn insert_enum_elem(
        &mut self,
        name: Ident,
        enum_name: Ident,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(SymbolKind::EnumerationElement { enum_name }, name);
        self.insert_symbol(symbol, local, errors)
    }

    /// Insert array in symbol table.
    pub fn insert_array(
        &mut self,
        name: Ident,
        array_type: Option<Typ>,
        size: usize,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol = Symbol::new(SymbolKind::Array { array_type, size }, name);

        self.insert_symbol(symbol, local, errors)
    }

    /// Insert fresh signal in symbol table.
    pub fn insert_fresh_signal(
        &mut self,
        fresh_name: Ident,
        scope: Scope,
        typing: Option<Typ>,
    ) -> usize {
        let symbol = Symbol::new(
            SymbolKind::Identifier {
                scope,
                typing,
                constant: None,
            },
            fresh_name,
        );

        self.insert_symbol(symbol, false, &mut vec![])
            .expect("internal error: you should not fail")
    }

    /// Insert fresh flow in symbol table.
    pub fn insert_fresh_flow(&mut self, fresh_name: Ident, kind: FlowKind, typing: Typ) -> usize {
        let symbol = Symbol::new(
            SymbolKind::Flow {
                path: None,
                kind,
                timer: None,
                typing,
            },
            fresh_name,
        );

        self.insert_symbol(symbol, false, &mut vec![])
            .expect("internal error: you should not fail")
    }

    /// Insert fresh period timer in symbol table.
    pub fn insert_fresh_period(&mut self, fresh_name: Ident, period: u64) -> usize {
        let symbol = Symbol::new(
            SymbolKind::Flow {
                path: None,
                kind: FlowKind::Event(Default::default()),
                timer: Some(TimerKind::Period(period)),
                typing: Typ::event(Typ::unit()),
            },
            fresh_name,
        );

        self.insert_symbol(symbol, false, &mut vec![])
            .expect("internal error: you should not fail")
    }

    /// Insert fresh deadline timer in symbol table.
    pub fn insert_fresh_deadline(&mut self, fresh_name: Ident, deadline: u64) -> usize {
        let symbol = Symbol::new(
            SymbolKind::Flow {
                path: None,
                kind: FlowKind::Event(Default::default()),
                timer: Some(TimerKind::Deadline(deadline)),
                typing: Typ::event(Typ::unit()),
            },
            fresh_name,
        );

        self.insert_symbol(symbol, false, &mut vec![])
            .expect("internal error: you should not fail")
    }

    /// Insert service delay timer in symbol table.
    pub fn insert_service_delay(
        &mut self,
        fresh_name: Ident,
        service_id: usize,
        delay: u64,
    ) -> usize {
        let symbol = Symbol::new(
            SymbolKind::Flow {
                path: None,
                kind: FlowKind::Event(Default::default()),
                timer: Some(TimerKind::ServiceDelay(service_id, delay)),
                typing: Typ::event(Typ::unit()),
            },
            fresh_name,
        );

        self.insert_symbol(symbol, false, &mut vec![])
            .expect("internal error: you should not fail")
    }

    /// Insert service timeout timer in symbol table.
    pub fn insert_service_timeout(
        &mut self,
        fresh_name: Ident,
        service_id: usize,
        timeout: u64,
    ) -> usize {
        let symbol = Symbol::new(
            SymbolKind::Flow {
                path: None,
                kind: FlowKind::Event(Default::default()),
                timer: Some(TimerKind::ServiceTimeout(service_id, timeout)),
                typing: Typ::event(Typ::unit()),
            },
            fresh_name,
        );

        self.insert_symbol(symbol, false, &mut vec![])
            .expect("internal error: you should not fail")
    }

    /// Restore a local context from identifiers.
    fn restore_context_from<'a>(&mut self, ids: impl Iterator<Item = &'a usize>) {
        ids.for_each(|id| self.restore_context_from_id(*id))
    }
    fn restore_context_from_id(&mut self, id: usize) {
        let key = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"))
            .hash();
        self.known_symbols.add_symbol(key, id);
    }

    /// Put identifier back in context.
    pub fn put_back_in_context(
        &mut self,
        id: usize,
        local: bool,
        loc: Loc,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
        let key = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"))
            .hash();
        if self.known_symbols.contains(&key, local) {
            bad!(errors, @loc => ErrorKind::elm_redef(self.get_name(id).to_string()))
        } else {
            self.known_symbols.add_symbol(key, id);
            Ok(())
        }
    }

    /// Restore node body or function inputs in context.
    pub fn restore_context(&mut self, id: usize) {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"))
            .clone();
        match symbol.kind() {
            SymbolKind::Function { inputs, .. } => {
                self.restore_context_from(inputs.iter());
            }
            SymbolKind::Node {
                inputs,
                outputs,
                locals,
                inits,
                ..
            } => {
                self.restore_context_from(inputs.iter());
                self.restore_context_from(outputs.iter().map(|(_, id)| id));
                if let Some(locals) = locals {
                    self.restore_context_from(locals.values());
                }
                if let Some(inits) = inits {
                    self.restore_context_from(inits.values());
                }
            }
            _ => noErrorDesc!(),
        }
    }

    /// Get symbol from identifier.
    pub fn get_symbol(&self, id: usize) -> Option<&Symbol> {
        self.table.get(&id)
    }

    /// Get symbol from identifier.
    pub fn resolve_symbol(&self, loc: Loc, id: usize) -> Res<&Symbol> {
        self.table
            .get(&id)
            .ok_or_else(lerror!(@loc => "[fatal] failed to resolve symbol identifier {}", id))
    }

    /// Get mutable symbol from identifier.
    pub fn get_symbol_mut(&mut self, id: usize) -> Option<&mut Symbol> {
        self.table.get_mut(&id)
    }

    /// Get type from identifier.
    pub fn get_typ(&self, id: usize) -> &Typ {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Identifier { typing, .. } => typing
                .as_ref()
                .unwrap_or_else(|| panic!("{} should be typed", symbol.name)),
            SymbolKind::Init { typing, .. } => typing
                .as_ref()
                .unwrap_or_else(|| panic!("{} should be typed", symbol.name)),
            SymbolKind::Flow { typing, .. } => typing,
            SymbolKind::Function { typing, .. } => typing
                .as_ref()
                .unwrap_or_else(|| panic!("{} should be typed", symbol.name)),
            _ => noErrorDesc!(),
        }
    }

    /// Get function output type from identifier.
    pub fn get_function_output_type(&self, id: usize) -> &Typ {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { output_type, .. } => {
                output_type.as_ref().expect("internal error: expect type")
            }
            _ => noErrorDesc!(),
        }
    }

    /// Get function input identifiers from identifier.
    pub fn get_function_input(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { inputs, .. } => inputs,
            _ => noErrorDesc!(),
        }
    }

    /// Retrieves the weight percent hint of a node/function.
    pub fn get_weight_percent_hint(&self, id: usize) -> Option<synced::Weight> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function {
                weight_percent_hint,
                ..
            } => Some(weight_percent_hint.unwrap_or(synced::weight::mid)),
            SymbolKind::Node {
                weight_percent_hint,
                ..
            } => Some(weight_percent_hint.unwrap_or(synced::weight::threads_lbi)),
            _ => None,
        }
    }

    /// Set function output type.
    pub fn set_function_output_type(&mut self, id: usize, new_type: Typ) {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        let inputs_type = match &symbol.kind {
            SymbolKind::Function { ref inputs, .. } => inputs
                .iter()
                .map(|id| self.get_typ(*id).clone())
                .collect_vec(),
            _ => noErrorDesc!(),
        };

        let symbol = self
            .get_symbol_mut(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
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
                *typing = Some(Typ::function(inputs_type, new_type))
            }
            _ => noErrorDesc!(),
        }
    }

    /// Set function output type.
    pub fn set_function_weight_percent_hint(
        &mut self,
        id: usize,
        weight_percent_hint: Option<usize>,
    ) {
        let symbol = self
            .get_symbol_mut(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));

        match &mut symbol.kind {
            SymbolKind::Function {
                weight_percent_hint: target,
                ..
            } => {
                *target = weight_percent_hint;
            }
            _ => noErrorDesc!(),
        }
    }

    /// Tell if identifier refers to function.
    pub fn is_function(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        matches!(symbol.kind(), SymbolKind::Function { .. })
    }

    /// Function path, used to rewrite function calls for external functions.
    pub fn try_get_function_path(&self, id: usize) -> Option<&syn::Path> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { path_opt, .. } => path_opt.as_ref(),
            _ => None,
        }
    }

    /// Set identifier's type.
    pub fn set_type(&mut self, id: usize, new_type: Typ) {
        let symbol = self
            .get_symbol_mut(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match &mut symbol.kind {
            SymbolKind::Identifier { ref mut typing, .. } => {
                if typing.is_some() {
                    panic!("type of {} can not be modified", symbol.name)
                }
                *typing = Some(new_type)
            }
            _ => noErrorDesc!(),
        }
    }

    /// Set flow's path.
    pub fn set_path(&mut self, id: usize, new_path: syn::Path) {
        let symbol = self
            .get_symbol_mut(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match &mut symbol.kind {
            SymbolKind::Flow { ref mut path, .. } => {
                if path.is_some() {
                    panic!("path of {} can not be modified", symbol.name)
                }
                *path = Some(new_path)
            }
            _ => noErrorDesc!(),
        }
    }

    /// Get flow's path.
    pub fn get_path(&self, id: usize) -> &syn::Path {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match &symbol.kind {
            SymbolKind::Flow { path, .. } => path.as_ref().unwrap(),
            _ => noErrorDesc!(),
        }
    }

    /// Get identifier's name.
    pub fn get_name(&self, id: usize) -> &Ident {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        &symbol.name
    }

    /// Get identifier's scope.
    pub fn get_scope(&self, id: usize) -> &Scope {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Identifier { scope, .. } => scope,
            _ => noErrorDesc!(),
        }
    }

    /// Set identifier's scope.
    pub fn set_scope(&mut self, id: usize, new_scope: Scope) {
        let symbol = self
            .get_symbol_mut(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind {
            SymbolKind::Identifier { ref mut scope, .. } => *scope = new_scope,
            _ => noErrorDesc!(),
        }
    }

    /// Get node input identifiers from identifier.
    pub fn get_node_inputs(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { inputs, .. } => inputs,
            _ => noErrorDesc!(),
        }
    }

    /// Get node output identifiers from identifier.
    pub fn get_node_outputs(&self, id: usize) -> &Vec<(Ident, usize)> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { outputs, .. } => outputs,
            _ => noErrorDesc!(),
        }
    }

    /// Get node local identifiers from identifier.
    pub fn get_node_locals(&self, id: usize) -> Vec<&usize> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { locals, .. } => {
                locals.as_ref().map_or(vec![], |h| h.values().collect())
            }
            _ => noErrorDesc!(),
        }
    }

    /// Get node's number of identifiers.
    pub fn node_idents_number(&self, id: usize) -> usize {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node {
                inputs,
                outputs,
                locals,
                ..
            } => inputs.len() + outputs.len() + locals.as_ref().map_or(0, |h| h.len()),
            _ => noErrorDesc!(),
        }
    }

    /// Get flow's kind from identifier.
    pub fn get_flow_kind(&self, id: usize) -> FlowKind {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { kind, .. } => kind.clone(),
            _ => noErrorDesc!(),
        }
    }

    /// Component path, used to rewrite component calls for external components.
    pub fn try_get_comp_path(&self, id: usize) -> Option<&syn::Path> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { path_opt, .. } => path_opt.as_ref(),
            _ => None,
        }
    }

    /// Tell wether the id is a timer.
    pub fn is_timer(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { timer, .. } => timer.is_some(),
            _ => noErrorDesc!(),
        }
    }

    /// Tell wether the id is a deadline timer.
    pub fn is_deadline(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { timer, .. } => timer
                .as_ref()
                .is_some_and(|timer| matches!(timer, TimerKind::Deadline(_))),
            _ => noErrorDesc!(),
        }
    }

    /// Tell wether the id is a periodic timer.
    pub fn is_period(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { timer, .. } => timer
                .as_ref()
                .is_some_and(|timer| matches!(timer, TimerKind::Period(_))),
            _ => noErrorDesc!(),
        }
    }

    /// Tell wether the id is a service delay timer.
    pub fn is_service_delay(&self, service_id: usize, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { timer, .. } => timer
                .as_ref()
                .is_some_and(|timer|
                    matches!(timer, TimerKind::ServiceDelay(other_service_id, _) if service_id == *other_service_id)),
            _ => noErrorDesc!(),
        }
    }

    /// Tell wether the id is a service timeout timer.
    pub fn is_timeout(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { timer, .. } => timer
                .as_ref()
                .is_some_and(|timer| matches!(timer, TimerKind::ServiceTimeout(_, _))),
            _ => noErrorDesc!(),
        }
    }

    /// Tell wether the id is the service delay timer.
    pub fn is_delay(&self, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { timer, .. } => timer
                .as_ref()
                .is_some_and(|timer| matches!(timer, TimerKind::ServiceDelay(_, _))),
            _ => noErrorDesc!(),
        }
    }

    /// Tell wether the id corresponds to a signal.
    pub fn is_signal(&self, id: usize) -> bool {
        !self.get_typ(id).is_event()
    }

    /// Tell wether the id is the service timeout timer.
    pub fn is_service_timeout(&self, service_id: usize, id: usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { timer, .. } => timer
                .as_ref()
                .is_some_and(|timer|
                    matches!(timer, TimerKind::ServiceTimeout(other_service_id, _) if service_id == *other_service_id)),
            _ => noErrorDesc!(),
        }
    }

    /// Tell get optional period of timer.
    pub fn get_period(&self, id: usize) -> Option<&u64> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Flow { timer, .. } => timer.as_ref().and_then(|timer| match timer {
                TimerKind::Period(period) => Some(period),
                _ => None,
            }),
            _ => noErrorDesc!(),
        }
    }

    /// Get structure' field identifiers from identifier.
    pub fn get_struct_fields(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Structure { fields, .. } => fields,
            _ => noErrorDesc!(),
        }
    }

    /// Get enumeration' element identifiers from identifier.
    pub fn get_enum_elements(&self, id: usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Enumeration { elements, .. } => elements,
            _ => noErrorDesc!(),
        }
    }

    /// Tell if identifier is a node.
    pub fn is_node(&self, name: &Ident, local: bool) -> bool {
        let symbol_hash = SymbolKey::Node { name: name.clone() };
        self.known_symbols.get_id(&symbol_hash, local).is_some()
    }

    /// Set array type from identifier.
    pub fn set_array_type(&mut self, id: usize, new_type: Typ) {
        let symbol = self
            .get_symbol_mut(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match &mut symbol.kind {
            SymbolKind::Array {
                ref mut array_type, ..
            } => {
                if array_type.is_some() {
                    panic!("a symbol type can not be modified")
                }
                *array_type = Some(new_type)
            }
            _ => noErrorDesc!(),
        }
    }

    /// Get array type from identifier.
    pub fn get_array(&self, id: usize) -> Typ {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { array_type, size } => Typ::array(
                array_type
                    .as_ref()
                    .expect("internal error: expect array element type")
                    .clone(),
                *size,
            ),
            _ => noErrorDesc!(),
        }
    }

    /// Get array element type from identifier.
    pub fn get_array_type(&self, id: usize) -> &Typ {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { array_type, .. } => array_type
                .as_ref()
                .expect("internal error: expect array element type"),
            _ => noErrorDesc!(),
        }
    }

    /// Get array size from identifier.
    pub fn get_array_size(&self, id: usize) -> usize {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { size, .. } => *size,
            _ => noErrorDesc!(),
        }
    }

    /// Gets a constant value.
    pub fn get_const(
        &self,
        ident: &Ident,
        levenshtein: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<&Expr> {
        let id = self.get_ident(ident, false, false, levenshtein, errors)?;
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Identifier {
                constant: Some(constant),
                ..
            } => Ok(constant),
            _ => Err(error!(@ident.loc() => ErrorKind::expected_constant())),
        }
        .dewrap(errors)
    }
    /// Tries to get constant value.
    pub fn try_get_const(&self, id: usize) -> Option<&Expr> {
        let symbol = self
            .get_symbol(id)
            .unwrap_or_else(|| panic!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Identifier { constant, .. } => constant.as_ref(),
            _ => None,
        }
    }

    /// Gets a variable (or function) identifier.
    pub fn get_ident(
        &self,
        name: &Ident,
        local: bool,
        or_function: bool,
        levenshtein: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Identifier { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                if or_function {
                    self.get_function_id(name, local, levenshtein, errors)
                } else {
                    self.unknown_ident_error(name, levenshtein).dewrap(errors)
                }
            }
        }
    }
    /// Get identifier symbol identifier.
    pub fn get_identifier_id(
        &self,
        name: &Ident,
        local: bool,
        levenshtein: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        self.get_ident(name, local, false, levenshtein, errors)
    }

    /// Get function symbol identifier.
    pub fn get_function_id(
        &self,
        name: &Ident,
        local: bool,
        levenshtein: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Function { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                let symbol_hash = SymbolKey::ExtFun { name: name.clone() };
                match self.known_symbols.get_id(&symbol_hash, local) {
                    Some(id) => Ok(id),
                    None => {
                        let mut current = &self.known_symbols;
                        loop {
                            for pair in current.current.iter() {
                                println!("- {:?} => {}", pair.0, pair.1);
                            }
                            if let Some(next) = current.global_context.as_ref() {
                                println!("next");
                                current = next
                            } else {
                                break;
                            }
                        }
                        self.unknown_ident_error(name, levenshtein)
                            .err_note(|| note!(@name.span() => "bad"))
                            .dewrap(errors)
                    }
                }
            }
        }
    }

    /// Get init symbol identifier.
    pub fn get_init_id(
        &self,
        name: &Ident,
        local: bool,
        levenshtein: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Init { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => self.unknown_init_error(name, levenshtein).dewrap(errors),
            // bad!(errors, @name.loc() => ErrorKind::unknown_ident(name.to_string())),
        }
    }

    /// Get function result symbol identifier.
    pub fn get_function_result_id(
        &self,
        local: bool,
        loc: Loc,
        levenshtein: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let ident = Ident::result(loc.span);
        let symbol_hash = SymbolKey::Identifier {
            name: ident.clone(),
        };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => self.unknown_ident_error(&ident, levenshtein).dewrap(errors),
            // None => bad!(errors, @loc => ErrorKind::unknown_ident(name)),
        }
    }

    /// Get flow symbol identifier.
    pub fn get_flow_id(&self, name: &Ident, local: bool, errors: &mut Vec<Error>) -> TRes<usize> {
        let symbol_hash = SymbolKey::Flow { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => bad!(errors, @name.loc() => ErrorKind::unknown_signal(name.to_string())),
        }
    }

    /// Get node symbol identifier.
    pub fn get_node_id(&self, name: &Ident, local: bool, errors: &mut Vec<Error>) -> TRes<usize> {
        let symbol_hash = SymbolKey::Node { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => bad!(errors, @name.loc() => ErrorKind::unknown_node(name.to_string())),
        }
    }

    /// Get structure symbol identifier.
    pub fn get_struct_id(
        &self,
        name: &Ident,
        local: bool,
        loc: Loc,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Structure { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => bad!(errors, @loc => ErrorKind::unknown_type(name.to_string())),
        }
    }

    /// Get enumeration symbol identifier.
    pub fn get_enum_id(
        &self,
        name: &Ident,
        local: bool,
        loc: Loc,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Enumeration { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => bad!(errors, @loc => ErrorKind::unknown_type(name.to_string())),
        }
    }

    /// Get enumeration element symbol identifier.
    pub fn get_enum_elem_id(
        &self,
        elem_name: &Ident,
        enum_name: &Ident,
        local: bool,
        loc: Loc,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::EnumerationElement {
            enum_name: enum_name.clone(),
            name: elem_name.clone(),
        };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => {
                bad!(errors, @loc =>
                    ErrorKind::unknown_enum_elem(enum_name.to_string(), elem_name.to_string())
                )
            }
        }
    }

    /// Get array symbol identifier.
    pub fn get_array_id(
        &self,
        name: &Ident,
        local: bool,
        loc: Loc,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        let symbol_hash = SymbolKey::Array { name: name.clone() };
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(id),
            None => bad!(errors, @loc => ErrorKind::unknown_type(name.to_string())),
        }
    }
}
