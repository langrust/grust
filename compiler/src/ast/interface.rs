use crate::common::{location::Location, r#type::Type};

// #[derive(Debug, PartialEq, Clone, serde::Serialize)]
// /// LanGRust interface AST.
// pub struct Interface {
//     /// Interface identifier.
//     pub id: String,
//     /// Interface's imports and their types.
//     pub imports: Vec<(Type, FlowPath)>,
//     /// Interface's exports and their types.
//     pub exports: Vec<FlowPath>,
//     /// Interface's flow statements.
//     pub flow_statements: Vec<FlowStatement>,
//     /// Interface location.
//     pub location: Location,
// }

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Flow statement AST.
pub struct FlowStatement {
    /// Identifier of the new flow.
    pub ident: String,
    /// Flow type.
    pub flow_type: Type,
    /// The expression defining the flow.
    pub flow_expression: FlowExpression,
    /// Flow statement location.
    pub location: Location,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Flow expression kinds.
pub enum FlowExpressionKind {
    /// Flow identifier call.
    Ident {
        /// The identifier of the flow to call.
        ident: String,
    },
    /// GReact `tiemout` operator.
    Timeout {
        /// Input expression.
        flow_expression: Box<FlowExpression>,
        /// Time of the timeout in milliseconds.
        timeout_ms: u64,
    },
    /// GReact `merge` operator.
    Merge {
        /// Input expression 1.
        flow_expression_1: Box<FlowExpression>,
        /// Input expression 2.
        flow_expression_2: Box<FlowExpression>,
    },
    /// GReact `zip` operator.
    Zip {
        /// Input expression 1.
        flow_expression_1: Box<FlowExpression>,
        /// Input expression 2.
        flow_expression_2: Box<FlowExpression>,
    },
    /// Component call.
    ComponentCall {
        /// Identifier to the component to call.
        ident_component: String,
        /// Input expressions.
        inputs: Vec<FlowExpression>,
        /// Identifier to the component output signal to call.
        ident_signal: String,
    },
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// Flow expression AST.
pub struct FlowExpression {
    /// Flow expression's kind.
    pub kind: FlowExpressionKind,
    /// Flow expression location.
    pub location: Location,
}

// #[derive(Debug, PartialEq, Clone, serde::Serialize)]
// /// Flow path kinds.
// pub enum FlowPathKind {
//     /// Path `module.path`.
//     Path {
//         /// Name of the module.
//         ident: String,
//         /// Rest of the path.
//         path: Box<FlowPath>,
//     },
//     /// Path `name as other_name`.
//     Rename {
//         /// Name of the flow.
//         ident: String,
//         /// Alias of the flow.
//         rename: String,
//     },
//     /// Path `name`.
//     Name {
//         /// Name of the flow.
//         ident: String,
//     },
// }

// #[derive(Debug, PartialEq, Clone, serde::Serialize)]
// /// Real flow path in the system.
// pub struct FlowPath {
//     /// Flow path kind.
//     pub kind: FlowPathKind,
//     /// Flow path loaction.
//     pub location: Location,
// }
// impl FlowPath {
//     /// Returns the name of the imported flow.
//     pub fn get_name(&self) -> String {
//         match &self.kind {
//             FlowPathKind::Name { ident } => ident.clone(),
//             FlowPathKind::Rename { rename, .. } => rename.clone(),
//             FlowPathKind::Path { path, .. } => path.get_name(),
//         }
//     }
// }
