//! Basic types, helpers and re-exports.

/// Imports the compiler prelude.
#[macro_export]
macro_rules! prelude {
    { just $($imports:tt)* } => {
        use $crate::prelude::{$($imports)*};
    };
    { $($imports:tt)* } => {
        use $crate::prelude::{*, $($imports)*};
    };
}

pub use compiler_common::import::*;

/// Can compute its [Weight].
pub trait HasWeight {
    /// Self's weight.
    fn weight(&self, bounds: &synced::WeightBounds, ctx: &Ctx) -> synced::Weight;
}
impl HasWeight for synced::Weight {
    fn weight(&self, _: &synced::WeightBounds, _: &Ctx) -> synced::Weight {
        *self
    }
}

pub mod ir0 {
    pub use crate::{
        defs::{
            contract::{self, Contract},
            equation::{self, Eq, ReactEq},
            expr::{self, Expr},
            interface::{
                self, ExtCompDecl, ExtFunDecl, FlowExport, FlowImport, Service, TimeRange,
            },
            stmt::{self, LetDecl, LogStmt, Stmt},
            stream, Ast, Colon, Component, ConstDecl, Ctx, Function, Item, Top, Typedef,
        },
        symbol,
    };
}

pub use crate::parsing::ParsePrec;
pub use ir0::*;

pub trait ParseItem: Sized + syn::Parse {
    /// Description of the `Self`-item.
    const DESC: &'static str;
    /// Parses the attributes of a `Self`-item.
    fn parse_attributes(self, attrs: Vec<syn::Attribute>) -> syn::Res<Self> {
        if let Some(attr) = attrs.first() {
            Err(syn::Error::new_spanned(
                attr,
                format!("item {} does not accept attributes", Self::DESC),
            ))
        } else {
            Ok(self)
        }
    }

    /// Parses an item and its attributes.
    fn parse_item(input: ParseStream, attrs: Vec<syn::Attribute>) -> syn::Res<Self> {
        let slf: Self = input.parse()?;
        slf.parse_attributes(attrs)
    }
}
