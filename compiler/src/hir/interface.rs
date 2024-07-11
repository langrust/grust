prelude! {
    syn::Token,
    graph::DiGraphMap,
    ast::keyword,
    hir::{flow, Pattern},
}

pub struct Service {
    /// Service's identifier.
    pub id: usize,
    /// Service's statements.
    pub statements: HashMap<usize, FlowStatement>,
    /// Flows dependency graph.
    pub graph: DiGraphMap<usize, ()>,
}
impl Service {
    pub fn get_flows_names(&self, symbol_table: &SymbolTable) -> Vec<String> {
        self.statements
            .values()
            .flat_map(|statement| match statement {
                FlowStatement::Declaration(FlowDeclaration { pattern, .. })
                | FlowStatement::Instantiation(FlowInstantiation { pattern, .. }) => pattern
                    .identifiers()
                    .into_iter()
                    .map(|id| symbol_table.get_name(id).clone()),
            })
            .collect()
    }
    pub fn get_flows_ids<'a>(&'a self) -> impl IntoIterator<Item = usize> + 'a {
        self.statements
            .values()
            .flat_map(|statement| match statement {
                FlowStatement::Declaration(FlowDeclaration { pattern, .. })
                | FlowStatement::Instantiation(FlowInstantiation { pattern, .. }) => {
                    pattern.identifiers()
                }
            })
    }
}

pub struct Interface {
    pub imports: HashMap<usize, FlowImport>,
    pub exports: HashMap<usize, FlowExport>,
    /// GRust interface's services.
    pub services: Vec<Service>,
}

/// Flow statement HIR.
#[derive(Clone)]
pub struct FlowDeclaration {
    pub let_token: Token![let],
    /// Pattern of flows and their types.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: flow::Expr,
    pub semi_token: Token![;],
}
/// Flow statement HIR.
#[derive(Clone)]
pub struct FlowInstantiation {
    /// Pattern of flows and their types.
    pub pattern: Pattern,
    pub eq_token: Token![=],
    /// The expression defining the flow.
    pub flow_expression: flow::Expr,
    pub semi_token: Token![;],
}
/// Flow statement HIR.
#[derive(Clone)]
pub struct FlowImport {
    pub import_token: keyword::import,
    /// Identifier of the flow and its type.
    pub id: usize,
    pub path: syn::Path,
    pub colon_token: Token![:],
    pub flow_type: Typ,
    pub semi_token: Token![;],
}
/// Flow statement HIR.
#[derive(Clone)]
pub struct FlowExport {
    pub export_token: keyword::export,
    /// Identifier of the flow and its type.
    pub id: usize,
    pub path: syn::Path,
    pub colon_token: Token![:],
    pub flow_type: Typ,
    pub semi_token: Token![;],
}

#[derive(Clone)]
pub enum FlowStatement {
    Declaration(FlowDeclaration),
    Instantiation(FlowInstantiation),
}
impl FlowStatement {
    /// Retrieves the component index and its inputs if the statement contains an invocation.
    pub fn try_get_call(&self) -> Option<(usize, &Vec<(usize, flow::Expr)>)> {
        use FlowStatement::*;
        match self {
            Declaration(FlowDeclaration {
                flow_expression:
                    flow::Expr {
                        kind:
                            flow::Kind::ComponentCall {
                                component_id,
                                inputs,
                            },
                        ..
                    },
                ..
            })
            | Instantiation(FlowInstantiation {
                flow_expression:
                    flow::Expr {
                        kind:
                            flow::Kind::ComponentCall {
                                component_id,
                                inputs,
                            },
                        ..
                    },
                ..
            }) => Some((*component_id, inputs)),
            Declaration(_) | Instantiation(_) => None,
        }
    }

    /// Tells if the statement is a component call.
    pub fn is_comp_call(&self) -> bool {
        use FlowStatement::*;
        match self {
            Declaration(FlowDeclaration {
                flow_expression:
                    flow::Expr {
                        kind: flow::Kind::ComponentCall { .. },
                        ..
                    },
                ..
            })
            | Instantiation(FlowInstantiation {
                flow_expression:
                    flow::Expr {
                        kind: flow::Kind::ComponentCall { .. },
                        ..
                    },
                ..
            }) => true,
            Declaration(_) | Instantiation(_) => false,
        }
    }

    /// Retrieves the identifiers the statement defines.
    pub fn get_identifiers(&self) -> Vec<usize> {
        use FlowStatement::*;
        match self {
            Declaration(FlowDeclaration { pattern, .. })
            | Instantiation(FlowInstantiation { pattern, .. }) => pattern.identifiers(),
        }
    }

    /// Retrieves the statement's dependencies.
    pub fn get_dependencies(&self) -> Vec<usize> {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                flow_expression, ..
            })
            | FlowStatement::Instantiation(FlowInstantiation {
                flow_expression, ..
            }) => flow_expression.get_dependencies(),
        }
    }
}
