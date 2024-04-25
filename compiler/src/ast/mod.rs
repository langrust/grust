use syn::parse::{Parse, ParseStream, Result};

mod config;

pub enum Item {
    Node(Node),
    Rust(syn::Item),
}
impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        if Node::peek_node(input) {
            Ok(Self::Node(input.parse()?))
        } else if let Ok(item) = input.parse() {
            Ok(Self::Rust(item))
        } else {
            Err(input.error("expected node or Rust item"))
        }
    }
}

pub struct Ast {
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

