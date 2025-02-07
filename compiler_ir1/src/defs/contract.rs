//! [Contract] module.

//! [Term] module.

prelude! {
    graph::*,
}

#[derive(Debug, PartialEq, Clone)]
/// A contract term kind.
pub enum Kind {
    /// Constant term: 3
    Constant {
        /// The constant
        constant: Constant,
    },
    /// Identifier term: x
    Identifier {
        /// Signal's identifier in Symbol Table.
        id: usize,
    },
    /// Last term: last x
    Last {
        /// Signal's memory in Symbol Table.
        init_id: usize,
        /// Signal's identifier in Symbol Table.
        signal_id: usize,
    },
    /// Enumeration term
    Enumeration {
        /// The enumeration id.
        enum_id: usize,
        /// The element id.
        element_id: usize,
    },
    /// Unary term: !x
    Unary {
        /// The operator
        op: UOp,
        /// The term
        term: Box<Term>,
    },
    /// Binary term: x == y
    Binary {
        /// The operator
        op: BOp,
        /// Left term
        left: Box<Term>,
        /// Right term
        right: Box<Term>,
    },
    /// Forall term: forall x, P(x)
    ForAll { id: usize, term: Box<Term> },
    /// Implication term: P => Q
    Implication { left: Box<Term>, right: Box<Term> },
    /// Present event pattern
    PresentEvent {
        /// The event identifier
        event_id: usize,
        /// The event pattern
        pattern: usize,
    },
    /// Application term.
    Application {
        /// The term applied.
        fun: Box<Term>,
        /// The inputs to the term.
        inputs: Vec<Term>,
    },
    /// Component call term.
    ComponentCall {
        /// Identifier to the memory location of the component.
        memory_id: Option<usize>,
        /// The called component identifier.
        comp_id: usize,
        /// The inputs to the term.
        inputs: Vec<(usize, Term)>,
    },
}

mk_new! { impl Kind =>
    Constant: constant { constant: Constant }
    Identifier: ident { id: usize }
    Last: last { init_id: usize, signal_id: usize }
    Enumeration: enumeration {
        enum_id: usize,
        element_id: usize,
    }
    Unary: unary {
        op: UOp,
        term: Term = term.into(),
    }
    Binary: binary {
        op: BOp,
        left: Term = left.into(),
        right: Term = right.into(),
    }
    ForAll: forall {
        id: usize,
        term: Term = term.into(),
    }
    Implication: implication {
        left: Term = left.into(),
        right: Term = right.into(),
    }
    PresentEvent: present {
        event_id: usize,
        pattern: usize,
    }
    Application: app {
        fun: Term = fun.into(),
        inputs: impl Into<Vec<Term>> = inputs.into(),
    }
    ComponentCall: call {
        memory_id = None,
        comp_id: usize,
        inputs: Vec<(usize, Term)>,
    }
}

#[derive(Debug, PartialEq, Clone)]
/// A contract term.
pub struct Term {
    /// The kind of the term
    pub kind: Kind,
    /// The type of the term
    pub typing: Option<Typ>,
    /// The location in source code
    pub loc: Loc,
}

mk_new! { impl Term =>
    new {
        kind: Kind,
        typing: Option<Typ>,
        loc: Loc,
    }
}

impl Term {
    /// Compute dependencies of a term.
    pub fn compute_dependencies(&self) -> Vec<usize> {
        match &self.kind {
            Kind::Unary { term, .. } => term.compute_dependencies(),
            Kind::Binary { left, right, .. } | Kind::Implication { left, right, .. } => {
                let mut dependencies = right.compute_dependencies();
                dependencies.extend(left.compute_dependencies());
                dependencies
            }
            Kind::Constant { .. } | Kind::Enumeration { .. } => {
                vec![]
            }
            Kind::Identifier { id } | Kind::PresentEvent { pattern: id, .. } => vec![*id],
            Kind::ForAll { id, term, .. } => term
                .compute_dependencies()
                .into_iter()
                .filter(|signal| id != signal)
                .collect(),
            Kind::Last { .. } => vec![],
            Kind::Application { fun, inputs, .. } => {
                let mut dependencies = fun.compute_dependencies();
                dependencies.extend(inputs.iter().flat_map(Term::compute_dependencies));
                dependencies
            }
            Kind::ComponentCall { inputs, .. } => inputs
                .iter()
                .flat_map(|(_, term)| term.compute_dependencies())
                .collect(),
        }
    }

    /// Add dependencies of a term to the graph.
    pub fn add_term_dependencies(&self, node_graph: &mut DiGraphMap<usize, Label>) {
        let dependencies = self.compute_dependencies();
        // signals used in the term depend on each other
        dependencies.iter().for_each(|id1| {
            dependencies.iter().for_each(|id2| {
                if id1 != id2 {
                    add_edge(node_graph, *id1, *id2, Label::Contract);
                    add_edge(node_graph, *id2, *id1, Label::Contract);
                }
            })
        })
    }

    /// Increment memory with ghost component applications.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        ctx: &mut Ctx,
    ) {
        match &mut self.kind {
            contract::Kind::ComponentCall {
                comp_id,
                memory_id: comp_memory_id,
                ..
            } => {
                debug_assert!(comp_memory_id.is_none());
                // create fresh identifier for the new memory buffer
                let comp_name = ctx.get_name(*comp_id);
                let memory_name =
                    identifier_creator.new_identifier(comp_name.loc(), &comp_name.to_string());
                let memory_id = ctx.insert_fresh_signal(memory_name, Scope::Local, None);
                memory.add_ghost_node(memory_id, *comp_id);
                // put the 'memory_id' of the called node
                *comp_memory_id = Some(memory_id);
            }
            Kind::Constant { .. }
            | Kind::Identifier { .. }
            | Kind::Last { .. }
            | Kind::Enumeration { .. }
            | Kind::PresentEvent { .. } => (),
            Kind::Unary { term, .. } | Kind::ForAll { term, .. } => {
                term.memorize(identifier_creator, memory, ctx)
            }
            Kind::Binary { left, right, .. } | Kind::Implication { left, right, .. } => {
                left.memorize(identifier_creator, memory, ctx);
                right.memorize(identifier_creator, memory, ctx);
            }
            Kind::Application { fun, inputs } => {
                fun.memorize(identifier_creator, memory, ctx);
                inputs
                    .iter_mut()
                    .for_each(|term| term.memorize(identifier_creator, memory, ctx));
            }
        }
    }

    /// Substitute an identifier with another one.
    pub fn substitution(&mut self, old_id: usize, new_id: usize) {
        match &mut self.kind {
            Kind::Constant { .. } | Kind::Enumeration { .. } => (),
            Kind::Identifier { ref mut id }
            | Kind::PresentEvent {
                pattern: ref mut id,
                ..
            } => {
                if *id == old_id {
                    *id = new_id
                }
            }
            Kind::Last {
                ref mut init_id,
                ref mut signal_id,
            } => {
                if *signal_id == old_id {
                    *signal_id = new_id
                }
                if *init_id == old_id {
                    *init_id = new_id
                }
            }
            Kind::Unary { ref mut term, .. } => {
                term.substitution(old_id, new_id);
            }
            Kind::Binary {
                ref mut left,
                ref mut right,
                ..
            }
            | Kind::Implication {
                ref mut left,
                ref mut right,
                ..
            } => {
                left.substitution(old_id, new_id);
                right.substitution(old_id, new_id);
            }
            Kind::ForAll { id, term, .. } => {
                if old_id != *id {
                    term.substitution(old_id, new_id)
                }
                // if 'id to replace' is equal to 'id of the forall' then nothing to do
            }
            Kind::Application { fun, inputs } => {
                fun.substitution(old_id, new_id);
                inputs
                    .iter_mut()
                    .for_each(|term| term.substitution(old_id, new_id));
            }
            Kind::ComponentCall {
                memory_id, inputs, ..
            } => {
                if *memory_id == Some(old_id) {
                    *memory_id = Some(new_id)
                }
                inputs
                    .iter_mut()
                    .for_each(|(_, term)| term.substitution(old_id, new_id));
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
/// Contract to prove using Creusot.
pub struct Contract {
    /// Requirements clauses to suppose
    pub requires: Vec<Term>,
    /// Ensures clauses to prove
    pub ensures: Vec<Term>,
    /// Invariant clauses to prove
    pub invariant: Vec<Term>,
}

impl Contract {
    /// Substitutes an identifier from another.
    pub fn substitution(&mut self, old_id: usize, new_id: usize) {
        self.requires
            .iter_mut()
            .for_each(|term| term.substitution(old_id, new_id));
        self.ensures
            .iter_mut()
            .for_each(|term| term.substitution(old_id, new_id));
        self.invariant
            .iter_mut()
            .for_each(|term| term.substitution(old_id, new_id));
    }

    /// Add dependencies of a contract to the graph.
    pub fn add_dependencies(&self, node_graph: &mut DiGraphMap<usize, Label>) {
        self.requires
            .iter()
            .for_each(|term| term.add_term_dependencies(node_graph));
        self.ensures
            .iter()
            .for_each(|term| term.add_term_dependencies(node_graph));
        self.invariant
            .iter()
            .for_each(|term| term.add_term_dependencies(node_graph));
    }

    /// Increment memory with ghost component applications.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        ctx: &mut Ctx,
    ) {
        self.requires
            .iter_mut()
            .for_each(|term| term.memorize(identifier_creator, memory, ctx));
        self.ensures
            .iter_mut()
            .for_each(|term| term.memorize(identifier_creator, memory, ctx));
        self.invariant
            .iter_mut()
            .for_each(|term| term.memorize(identifier_creator, memory, ctx));
    }
}
