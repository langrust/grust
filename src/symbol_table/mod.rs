use std::collections::BTreeMap;
use strum::IntoEnumIterator;

use crate::{
    common::{
        location::Location,
        operator::{BinaryOperator, OtherOperator, UnaryOperator},
        r#type::Type,
        scope::Scope,
    },
    error::{Error, TerminationError},
};

#[derive(Clone)]
pub enum SymbolKind {
    Identifier {
        scope: Scope,
        typing: Option<Type>,
    },
    Function {
        inputs: Vec<usize>,
        output_type: Option<Type>,
        typing: Option<Type>,
    },
    Node {
        /// Is true when the node is a component.
        is_component: bool,
        /// Node's input identifiers.
        inputs: Vec<usize>,
        /// Node's output identifiers.
        outputs: BTreeMap<String, usize>,
        /// Node's local identifiers.
        locals: BTreeMap<String, usize>,
    },
    UnitaryNode {
        /// Is true when the node is a component.
        is_component: bool,
        /// Mother node identifier.
        mother_node: usize,
        /// Node's input identifiers.
        inputs: Vec<usize>,
        /// Node's output identifier.
        output: usize,
    },
    Structure {
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<usize>,
    },
    Enumeration {
        /// The enumeration's elements.
        elements: Vec<usize>,
    },
    EnumerationElement {
        enum_name: String,
    },
    Array {
        /// The array's type.
        array_type: Option<Type>,
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
            SymbolKind::Identifier { .. } => format!("identifier {}", self.name),
            SymbolKind::Function { .. } => format!("function {}", self.name),
            SymbolKind::Node { .. } => format!("node {}", self.name),
            SymbolKind::UnitaryNode { .. } => format!("unitary_node {}", self.name),
            SymbolKind::Structure { .. } => format!("struct {}", self.name),
            SymbolKind::Enumeration { .. } => format!("enum {}", self.name),
            SymbolKind::EnumerationElement { enum_name } => {
                format!("enum_elem {enum_name}::{}", self.name)
            }
            SymbolKind::Array { .. } => format!("array {}", self.name),
        }
    }
}

pub struct Context {
    current: BTreeMap<String, usize>,
    global_context: Option<Box<Context>>,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            current: BTreeMap::new(),
            global_context: None,
        }
    }
}
impl Context {
    fn new() -> Self {
        Self {
            current: BTreeMap::new(),
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
            current: BTreeMap::new(),
            global_context: Some(Box::new(self)),
        }
    }
    fn get_global_context(self) -> Context {
        *self.global_context.expect("there is no global context")
    }
}

pub struct SymbolTable {
    table: BTreeMap<usize, Symbol>,
    fresh_id: usize,
    known_symbols: Context,
}
impl Default for SymbolTable {
    fn default() -> Self {
        Self {
            table: BTreeMap::new(),
            fresh_id: 0,
            known_symbols: Default::default(),
        }
    }
}
impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table: BTreeMap::new(),
            fresh_id: 0,
            known_symbols: Context::new(),
        }
    }

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
        inputs: Vec<usize>,
        output_type: Option<Type>,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
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

    pub fn insert_node(
        &mut self,
        name: String,
        is_component: bool,
        local: bool,
        inputs: Vec<usize>,
        outputs: BTreeMap<String, usize>,
        locals: BTreeMap<String, usize>,
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

    pub fn insert_enum_elem(
        &mut self,
        name: String,
        enum_name: String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol = Symbol {
            kind: SymbolKind::EnumerationElement { enum_name },
            name,
        };

        self.insert_symbol(symbol, local, location, errors)
    }

    pub fn insert_array(
        &mut self,
        name: String,
        array_type: Option<Type>,
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

    pub fn insert_unitary_node(
        &mut self,
        node_name: String,
        output_name: String,
        is_component: bool,
        mother_node: usize,
        inputs: Vec<usize>,
        output: usize,
    ) -> usize {
        let name = format!("{node_name}_{output_name}");
        let symbol = Symbol {
            kind: SymbolKind::UnitaryNode {
                is_component,
                mother_node,
                inputs,
                output,
            },
            name,
        };

        self.insert_symbol(symbol, false, Location::default(), &mut vec![])
            .expect("you should not fail")
    }

    pub fn insert_fresh_signal(
        &mut self,
        fresh_name: String,
        scope: Scope,
        typing: Option<Type>,
    ) -> usize {
        let symbol = Symbol {
            kind: SymbolKind::Identifier { scope, typing },
            name: fresh_name,
        };

        self.insert_symbol(symbol, false, Location::default(), &mut vec![])
            .expect("you should not fail") // todo make it local
    }

    fn restore_context_from<'a>(&mut self, ids: impl Iterator<Item = &'a usize>) {
        ids.for_each(|id| {
            let symbol = self
                .get_symbol(id)
                .expect(&format!("expect symbol for {id}"))
                .clone();
            self.known_symbols.add_symbol(symbol, *id);
        })
    }

    pub fn restore_context(&mut self, id: &usize) {
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
                outputs,
                locals,
                ..
            } => {
                self.restore_context_from(inputs.iter());
                self.restore_context_from(outputs.values());
                self.restore_context_from(locals.values());
            }
            _ => unreachable!(),
        }
    }

    pub fn get_symbol(&self, id: &usize) -> Option<&Symbol> {
        self.table.get(id)
    }

    pub fn get_symbol_mut(&mut self, id: &usize) -> Option<&mut Symbol> {
        self.table.get_mut(id)
    }

    pub fn get_type(&self, id: &usize) -> &Type {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Identifier { typing, .. } => typing
                .as_ref()
                .expect(&format!("{} should be typed", symbol.name)),
            SymbolKind::Function { typing, .. } => typing
                .as_ref()
                .expect(&format!("{} should be typed", symbol.name)),
            _ => unreachable!(),
        }
    }

    pub fn get_function_output_type(&self, id: &usize) -> &Type {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { output_type, .. } => output_type.as_ref().expect("expect type"),
            _ => unreachable!(),
        }
    }

    pub fn get_function_input(&self, id: &usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { inputs, .. } => inputs,
            _ => unreachable!(),
        }
    }

    pub fn set_function_output_type(&mut self, id: &usize, new_type: Type) {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        let inputs_type = match &symbol.kind {
            SymbolKind::Function { ref inputs, .. } => inputs
                .iter()
                .map(|id| self.get_type(id).clone())
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
                *typing = Some(Type::Abstract(inputs_type, Box::new(new_type)))
            }
            _ => unreachable!(),
        }
    }

    pub fn is_function(&self, id: &usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Function { .. } => true,
            _ => false,
        }
    }

    pub fn get_unitary_node_output_type(&self, id: &usize) -> &Type {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::UnitaryNode { output, .. } => self.get_type(output),
            _ => unreachable!(),
        }
    }

    pub fn get_unitary_node_output_name(&self, id: &usize) -> &String {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::UnitaryNode { output, .. } => self.get_name(output),
            _ => unreachable!(),
        }
    }

    pub fn get_unitary_node_output_id(&self, id: &usize) -> &usize {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::UnitaryNode { output, .. } => output,
            _ => unreachable!(),
        }
    }

    pub fn get_unitary_node_used_inputs(&self, id: &usize) -> Vec<bool> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::UnitaryNode {
                mother_node,
                inputs,
                ..
            } => {
                let mother_node_inputs = self.get_node_inputs(mother_node);
                mother_node_inputs
                    .iter()
                    .map(|id| inputs.contains(id))
                    .collect()
            }
            _ => unreachable!(),
        }
    }

    pub fn get_unitary_node_inputs(&self, id: &usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::UnitaryNode { inputs, .. } => inputs,
            _ => unreachable!(),
        }
    }

    pub fn set_type(&mut self, id: &usize, new_type: Type) {
        let symbol = self
            .get_symbol_mut(id)
            .expect(&format!("expect symbol for {id}"));
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
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        &symbol.name
    }

    pub fn get_scope(&self, id: &usize) -> &Scope {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Identifier { scope, .. } => scope,
            _ => unreachable!(),
        }
    }

    pub fn set_scope(&mut self, id: &usize, new_scope: Scope) {
        let symbol = self
            .get_symbol_mut(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind {
            SymbolKind::Identifier { ref mut scope, .. } => *scope = new_scope,
            _ => unreachable!(),
        }
    }

    pub fn get_node_inputs(&self, id: &usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { inputs, .. } => inputs,
            _ => unreachable!(),
        }
    }

    pub fn is_component(&self, id: &usize) -> bool {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Node { is_component, .. } => *is_component,
            _ => unreachable!(),
        }
    }

    pub fn get_struct_fields(&self, id: &usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Structure { fields, .. } => fields,
            _ => unreachable!(),
        }
    }

    pub fn get_enum_elements(&self, id: &usize) -> &Vec<usize> {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Enumeration { elements, .. } => elements,
            _ => unreachable!(),
        }
    }

    pub fn is_node(&self, name: &String, local: bool) -> bool {
        let symbol_hash = format!("node {name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn set_array_type(&mut self, id: &usize, new_type: Type) {
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

    pub fn get_array(&self, id: &usize) -> Type {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { array_type, size } => Type::Array(
                Box::new(array_type.as_ref().expect("expect type").clone()),
                *size,
            ),
            _ => unreachable!(),
        }
    }

    pub fn get_array_type(&self, id: &usize) -> &Type {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { array_type, .. } => array_type.as_ref().expect("expect type"),
            _ => unreachable!(),
        }
    }

    pub fn get_array_size(&self, id: &usize) -> usize {
        let symbol = self
            .get_symbol(id)
            .expect(&format!("expect symbol for {id}"));
        match symbol.kind() {
            SymbolKind::Array { size, .. } => *size,
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
        let symbol_hash = format!("identifier {name}");
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
        let symbol_hash = format!("function {name}");
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
        let symbol_hash = format!("identifier {name}");
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
        let symbol_hash = format!("node {name}");
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

    pub fn get_struct_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol_hash = format!("struct {name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(*id),
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

    pub fn get_enum_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol_hash = format!("enum {name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(*id),
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

    pub fn get_enum_elem_id(
        &self,
        elem_name: &String,
        enum_name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol_hash = format!("enum_elem {enum_name}::{elem_name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(*id),
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

    pub fn get_array_id(
        &self,
        name: &String,
        local: bool,
        location: Location,
        errors: &mut Vec<Error>,
    ) -> Result<usize, TerminationError> {
        let symbol_hash = format!("array {name}");
        match self.known_symbols.get_id(&symbol_hash, local) {
            Some(id) => Ok(*id),
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

    pub fn get_unitary_node_id(&self, node_name: &String, output_name: &String) -> usize {
        let symbol_hash = format!("unitary_node {node_name}_{}", output_name);
        *self
            .known_symbols
            .get_id(&symbol_hash, false)
            .expect("there should be an unitary node")
    }
}
