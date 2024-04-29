use syn::parse::{Parse, ParseStream, Result};

use self::{component::Component, function::Function, interface::FlowStatement, typedef::Typedef};

mod component;
mod contract;
mod config;
mod equation;
mod expression;
mod function;
mod interface;
mod pattern;
mod statement;
mod stream_expression;
mod typedef;
pub mod keyword;

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
        /* if Component::peek_component(input) {
            Ok(Self::Component(input.parse()?))
        } else */
        if let Ok(item) = input.parse() {
            Ok(Self::Rust(item))
        } else {
            Err(input.error("expected component or Rust item"))
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
        let mut items = Vec::with_capacity(100);
        'parse_items: loop {
            if input.is_empty() {
                break 'parse_items;
            }
            items.push(input.parse()?);
        }
        items.shrink_to_fit();
        Ok(Self { items })
    }
}
