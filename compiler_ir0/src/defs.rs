//! [ir0] definitions.

pub mod contract;
pub mod equation;
pub mod expr;
pub mod interface;
pub mod stmt;
pub mod stream;

prelude! {}

/// An `ir0` context, gathers a symbol table and a [`Conf`].
///
/// For convenience, this type [`std::ops::Deref`]s/[`std::ops::DerefMut`]s to [`symbol::Table`].
pub struct Ctx {
    pub table: symbol::Table,
    pub conf: conf::Conf,
}
impl std::ops::Deref for Ctx {
    type Target = symbol::Table;
    fn deref(&self) -> &Self::Target {
        &self.table
    }
}
impl std::ops::DerefMut for Ctx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.table
    }
}
impl Ctx {
    /// Constructor.
    pub fn new(table: symbol::Table, conf: Conf) -> Self {
        Self { table, conf }
    }

    /// Constructor from an existing configuration (empty table).
    pub fn from_conf(conf: Conf) -> Self {
        Self::new(symbol::Table::new(), conf)
    }

    /// Empty context: empty symbol table and default configuration.
    pub fn empty() -> Self {
        Self::from_conf(Conf::default())
    }
}

pub struct Colon<U, V> {
    pub left: U,
    pub colon: Token![:],
    pub right: V,
}

/// GRust user defined type AST.
pub enum Typedef {
    /// Represents a structure definition.
    Structure {
        struct_token: Token![struct],
        /// Typedef identifier.
        ident: Ident,
        brace: syn::token::Brace,
        /// The structure's fields: a field has an identifier and a type.
        fields: syn::Punctuated<Colon<Ident, Typ>, Token![,]>,
    },
    /// Represents an enumeration definition.
    Enumeration {
        enum_token: Token![enum],
        /// Typedef identifier.
        ident: Ident,
        brace: syn::token::Brace,
        /// The structure's fields: a field has an identifier and a type.
        elements: syn::Punctuated<Ident, Token![,]>,
    },
    /// Represents an array definition.
    Array {
        array_token: keyword::array,
        /// Typedef identifier.
        ident: Ident,
        bracket_token: syn::token::Bracket,
        /// The array's type.
        array_type: Typ,
        semi_token: Token![;],
        /// The array's size.
        size: syn::LitInt,
    },
}
impl HasLoc for Typedef {
    fn loc(&self) -> Loc {
        match self {
            Self::Structure {
                struct_token,
                brace,
                ..
            } => Loc::from(struct_token.span).join(brace.span.join()),
            Self::Enumeration {
                enum_token, brace, ..
            } => Loc::from(enum_token.span).join(brace.span.join()),
            Self::Array {
                array_token,
                bracket_token,
                ..
            } => Loc::from(array_token.span).join(bracket_token.span.join()),
        }
    }
}

/// GRust component AST.
pub struct Component {
    pub component_token: keyword::component,
    /// Component identifier.
    pub ident: Ident,
    pub args_paren: syn::token::Paren,
    /// Component's inputs identifiers and their types.
    pub args: syn::Punctuated<Colon<Ident, Typ>, Token![,]>,
    pub arrow_token: Token![->],
    pub outs_paren: syn::token::Paren,
    /// Component's outputs identifiers and their types.
    pub outs: syn::Punctuated<Colon<Ident, Typ>, Token![,]>,
    /// Component's computation period.
    pub period: Option<(Token![@], syn::LitInt, keyword::ms)>,
    /// Component's contract.
    pub contract: Contract,
    pub brace: syn::token::Brace,
    /// Component's equations.
    pub equations: Vec<equation::ReactEq>,
}
impl HasLoc for Component {
    fn loc(&self) -> Loc {
        Loc::from(self.component_token.span).join(self.brace.span.join())
    }
}

/// GRust component import AST.
pub struct ComponentImport {
    pub import_token: keyword::import,
    pub component_token: keyword::component,
    pub path: syn::Path,
    pub colon_token: Token![:],
    pub args_paren: syn::token::Paren,
    /// Component's inputs identifiers and their types.
    pub args: syn::Punctuated<Colon<Ident, Typ>, Token![,]>,
    pub arrow_token: Token![->],
    pub outs_paren: syn::token::Paren,
    /// Component's outputs identifiers and their types.
    pub outs: syn::Punctuated<Colon<Ident, Typ>, Token![,]>,
    /// Component's computation period.
    pub period: Option<(Token![@], syn::LitInt, keyword::ms)>,
    pub semi_token: Token![;],
}
impl HasLoc for ComponentImport {
    fn loc(&self) -> Loc {
        Loc::from(self.import_token.span).join(self.semi_token.span)
    }
}

/// GRust function AST.
pub struct Function {
    pub function_token: keyword::function,
    /// Function identifier.
    pub ident: Ident,
    pub args_paren: syn::token::Paren,
    /// Function's inputs identifiers and their types.
    pub args: syn::Punctuated<Colon<Ident, Typ>, Token![,]>,
    pub arrow_token: Token![->],
    pub output_type: Typ,
    /// Function's contract.
    pub contract: Contract,
    pub brace: syn::token::Brace,
    /// Function's statements.
    pub statements: Vec<Stmt>,
}
impl HasLoc for Function {
    fn loc(&self) -> Loc {
        Loc::from(self.function_token.span).join(self.brace.span.join())
    }
}

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
impl HasLoc for Item {
    fn loc(&self) -> Loc {
        match self {
            Self::ComponentImport(ci) => ci.loc(),
            Self::Component(c) => c.loc(),
            Self::Function(f) => f.loc(),
            Self::Typedef(t) => t.loc(),
            Self::Service(s) => s.loc(),
            Self::Import(i) => i.loc(),
            Self::Export(e) => e.loc(),
        }
    }
}

/// Complete AST of GRust program.
pub struct Ast {
    /// Items contained in the GRust program.
    pub items: Vec<Item>,
}

pub struct Top {
    pub ast: Ast,
    pub conf: Conf,
}
impl Top {
    pub fn init(self) -> (Ast, Ctx) {
        (self.ast, Ctx::from_conf(self.conf))
    }
}
