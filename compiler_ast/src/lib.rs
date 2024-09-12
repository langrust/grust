pub extern crate compiler_common as common;

#[macro_use]
pub mod prelude;

prelude! {
    syn::parse::{Parse, ParseStream, Result},
}

mod colon;
mod component;
mod config;
mod function;

pub mod contract;
pub mod equation;
pub mod expr;
pub mod interface;
pub mod stmt;
pub mod stream;
pub mod symbol;
pub mod typedef;

/// Things that can appear in a GRust program.
pub enum Item {
    ComponentImport(ComponentImport),
    /// GRust synchronous component.
    Component(Component),
    /// GRust function.
    Function(Function),
    /// GRust typedef.
    Typedef(Typedef),
    /// GRust service.
    Service(Service),
    Import(FlowImport),
    Export(FlowExport),
}
impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        if ComponentImport::peek(input) {
            Ok(Item::ComponentImport(input.parse()?))
        } else if Component::peek(input) {
            Ok(Item::Component(input.parse()?))
        } else if Function::peek(input) {
            Ok(Item::Function(input.parse()?))
        } else if Typedef::peek(input) {
            Ok(Item::Typedef(input.parse()?))
        } else if Service::peek(input) {
            Ok(Item::Service(input.parse()?))
        } else if FlowImport::peek(input) {
            Ok(Item::Import(input.parse()?))
        } else if FlowExport::peek(input) {
            Ok(Item::Export(input.parse()?))
        } else {
            Err(input.error(
                "expected flow import or export, type, component definition or import, function or service definition",
            ))
        }
    }
}

/// Complete AST of GRust program.
pub struct Ast {
    /// Items contained in the GRust program.
    pub items: Vec<Item>,
}
impl Parse for Ast {
    fn parse(input: ParseStream) -> Result<Self> {
        let _: config::Config = input.parse()?;
        let items: Vec<Item> = {
            let mut items = Vec::with_capacity(100);
            while !input.is_empty() {
                items.push(input.parse()?);
            }
            items.shrink_to_fit();
            items
        };
        Ok(Self { items })
    }
}
