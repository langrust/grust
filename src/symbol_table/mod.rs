use std::collections::HashMap;

use crate::{
    common::{location::Location, r#type::Type, scope::Scope},
    error::{Error, TerminationError},
};

#[derive(Clone)]
pub enum SymbolKind {
    Identifier {
        scope: Scope,
        typing: Option<Type>,
    },
    Function {
        inputs_typing: Vec<Type>,
        output_typing: Type,
    },
    Node {
        /// Is true when the node is a component.
        is_component: bool,
        /// Node's input signals.
        inputs: Vec<usize>,
        /// Node's output signals.
        outputs: HashMap<String, usize>,
        /// Node's local signals.
        locals: HashMap<String, usize>,
    },
    Structure {
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<usize>,
    },
    Enumeration {
        /// The enumeration's elements.
        elements: Vec<usize>,
    },
    Array {
        /// The array's type.
        array_type: Type,
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

#[derive(Clone)]
pub struct Symbol {
    kind: SymbolKind,
    name: String,
}
impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name
    }
}
impl Symbol {
    pub fn kind(&self) -> &SymbolKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut SymbolKind {
        &mut self.kind
    }

    fn hash_as_string(&self) -> String {
        match &self.kind {
            SymbolKind::Identifier { .. } => format!("identifier_{}", self.name),
            SymbolKind::Function { .. } => format!("function_{}", self.name),
            SymbolKind::Node { .. } => format!("node_{}", self.name),
            SymbolKind::Structure { .. } => format!("struct_{}", self.name),
            SymbolKind::Enumeration { .. } => format!("enum_{}", self.name),
            SymbolKind::Array { .. } => format!("array_{}", self.name),
        }
    }
}

pub struct Context {
    current: HashMap<String, usize>,
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
    fn add_symbol(&mut self, symbol: Symbol, id: usize) {
        self.current.insert(symbol.hash_as_string(), id);
    }
    fn contains(&self, symbol: &Symbol, local: bool) -> bool {
        let contains = self.current.contains_key(&symbol.hash_as_string());
        if local {
            contains
        } else {
            match &self.global_context {
                Some(context) => contains || context.contains(symbol, local),
                None => contains,
            }
        }
    }
    fn get_id(&self, symbol_hash: &String, local: bool) -> Option<&usize> {
        let contains = self.current.get(symbol_hash);
        if local {
            contains
        } else {
            contains.or_else(|| {
                self.global_context
                    .as_ref()
                    .map(|context| context.get_id(symbol_hash, local))
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

pub struct SymbolTable {
    table: HashMap<usize, Symbol>,
    fresh_id: usize,
    known_symbols: Context,
}
impl Default for SymbolTable {
    fn default() -> Self {
        Self {
            table: HashMap::new(),
            fresh_id: 0,
            known_symbols: Default::default(),
        }
    }
}
impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            fresh_id: 0,
            known_symbols: Context::new(),
        }
    }

    pub fn local(&mut self) {
        let prev = std::mem::take(&mut self.known_symbols);
        self.known_symbols = prev.create_local_context();
    }

    pub fn global(&mut self) {
        let prev = std::mem::take(&mut self.known_symbols);
        self.known_symbols = prev.get_global_context();
    }

    fn insert_symbol(
        &mut self,
        symbol: Symbol,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        if self.known_symbols.contains(&symbol, local) {
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
            self.known_symbols.add_symbol(symbol, id);
            // return symbol's id
            Ok(id)
        }
    }

    pub fn insert_signal(
        &mut self,
        name: String,
        scope: Scope,
        typing: Option<Type>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol = Symbol {
            kind: SymbolKind::Identifier { scope, typing },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    pub fn insert_identifier(
        &mut self,
        name: String,
        typing: Option<Type>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol = Symbol {
            kind: SymbolKind::Identifier {
                scope: Scope::Local,
                typing,
            },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    pub fn insert_function(
        &mut self,
        name: String,
        inputs_typing: Vec<Type>,
        output_typing: Type,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol = Symbol {
            kind: SymbolKind::Function {
                inputs_typing,
                output_typing,
            },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    pub fn insert_node(
        &mut self,
        name: String,
        is_component: bool,
        local: bool,
        inputs: Vec<usize>,
        outputs: HashMap<String, usize>,
        locals: HashMap<String, usize>,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol = Symbol {
            kind: SymbolKind::Node {
                is_component,
                inputs,
                outputs,
                locals,
            },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    pub fn insert_struct(
        &mut self,
        name: String,
        fields: Vec<usize>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol = Symbol {
            kind: SymbolKind::Structure { fields },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    pub fn insert_enum(
        &mut self,
        name: String,
        elements: Vec<usize>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol = Symbol {
            kind: SymbolKind::Enumeration { elements },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    pub fn insert_array(
        &mut self,
        name: String,
        array_type: Type,
        size: usize,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol = Symbol {
            kind: SymbolKind::Array { array_type, size },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    pub fn restore_context<'a>(&mut self, ids: impl Iterator<Item = &'a usize>) {
        ids.for_each(|id| {
            let symbol = self.get_symbol(id).unwrap().clone();
            self.known_symbols.add_symbol(symbol, *id);
        })
    }

    pub fn get_symbol(&self, id: &usize) -> Option<&Symbol> {
        self.table.get(id)
    }

    pub fn get_symbol_mut(&mut self, id: &usize) -> Option<&mut Symbol> {
        self.table.get_mut(id)
    }

    pub fn get_type(&self, id: &usize) -> &Type {
        let symbol = self.get_symbol(id).expect("expect symbol");
        match symbol.kind() {
            SymbolKind::Identifier { typing, .. } => typing.as_ref().expect("should be typed"),
            _ => unreachable!(),
        }
    }

    pub fn get_output_type(&self, id: &usize) -> &Type {
        let symbol = self.get_symbol(id).expect("expect symbol");
        match symbol.kind() {
            SymbolKind::Function { output_typing, .. } => output_typing,
            _ => unreachable!(),
        }
    }

    pub fn set_type(&mut self, id: &usize, new_type: Type) {
        let symbol = self.get_symbol_mut(id).expect("expect symbol");
        match &mut symbol.kind {
            SymbolKind::Identifier { ref mut typing, .. } => {
                if typing.is_some() {
                    panic!("a symbol type can not be modified")
                }
                *typing = Some(new_type)
            }
            _ => unreachable!(),
        }
    }

    pub fn get_name(&self, id: &usize) -> &String {
        let symbol = self.get_symbol(id).expect("expect symbol");
        &symbol.name
    }

    pub fn set_name(&mut self, id: &usize, new_name: String) {
        let symbol = self.get_symbol_mut(id).expect("expect symbol");
        symbol.name = new_name;
    }

    pub fn get_scope(&self, id: &usize) -> &Scope {
        let symbol = self.get_symbol(id).expect("expect symbol");
        match symbol.kind() {
            SymbolKind::Identifier { scope, .. } => scope,
            _ => unreachable!(),
        }
    }

    pub fn set_scope(&mut self, id: &usize, new_scope: Scope) {
        let symbol = self.get_symbol_mut(id).expect("expect symbol");
        match symbol.kind {
            SymbolKind::Identifier { ref mut scope, .. } => *scope = new_scope,
            _ => unreachable!(),
        }
    }

    pub fn get_node_input(&self, id: &usize) -> &Vec<usize> {
        let symbol = self.get_symbol(id).expect("expect symbol");
        match symbol.kind() {
            SymbolKind::Node { inputs, .. } => inputs,
            _ => unreachable!(),
        }
    }

    pub fn get_identifier_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol_hash = format!("identifier_{name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(*id),
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

    pub fn get_function_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol_hash = format!("function_{name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(*id),
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

    pub fn get_signal_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol_hash = format!("signal_{name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(*id),
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

    pub fn get_node_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol_hash = format!("node_{name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(*id),
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
}
