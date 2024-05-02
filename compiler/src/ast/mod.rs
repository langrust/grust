use syn::parse::{Parse, ParseStream, Result};

use self::{component::Component, function::Function, interface::FlowStatement, typedef::Typedef};

pub mod component;
pub mod config;
pub mod contract;
pub mod equation;
pub mod expression;
pub mod function;
pub mod ident_colon;
pub mod interface;
pub mod keyword;
pub mod pattern;
pub mod statement;
pub mod stream_expression;
pub mod typedef;

/// Things that can appear in a GRust program.
pub enum Item {
    /// GRust synchronous component.
    Component(Component),
    /// GRust function.
    Function(Function),
    /// GRust typedef.
    Typedef(Typedef),
    /// GRust FRP flow statement.
    FlowStatement(FlowStatement),
    /// Rust item that can appear inside of a module.
    Rust(syn::Item),
}
impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        if Component::peek(input) {
            Ok(Item::Component(input.parse()?))
        } else if Function::peek(input) {
            Ok(Item::Function(input.parse()?))
        } else if Typedef::peek(input) {
            Ok(Item::Typedef(input.parse()?))
        } else if FlowStatement::peek(input) {
            Ok(Item::FlowStatement(input.parse()?))
        } else {
            Ok(Item::Rust(input.parse()?))
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
