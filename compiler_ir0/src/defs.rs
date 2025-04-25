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

    /// Gets a constant value.
    pub fn get_const(&self, ident: &Ident, errors: &mut Vec<Error>) -> TRes<&Constant> {
        self.table.get_const(ident, self.conf.levenshtein, errors)
    }

    /// Gets a variable (or function) identifier.
    pub fn get_ident(
        &self,
        name: &Ident,
        local: bool,
        or_function: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        self.table
            .get_ident(name, local, or_function, self.conf.levenshtein, errors)
    }

    /// Get identifier symbol identifier.
    pub fn get_identifier_id(
        &self,
        name: &Ident,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        self.table
            .get_identifier_id(name, local, self.conf.levenshtein, errors)
    }

    /// Get function symbol identifier.
    pub fn get_function_id(
        &self,
        name: &Ident,
        local: bool,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        self.table
            .get_function_id(name, local, self.conf.levenshtein, errors)
    }

    /// Get init symbol identifier.
    pub fn get_init_id(&self, name: &Ident, local: bool, errors: &mut Vec<Error>) -> TRes<usize> {
        self.table
            .get_init_id(name, local, self.conf.levenshtein, errors)
    }

    /// Get function result symbol identifier.
    pub fn get_function_result_id(
        &self,
        local: bool,
        loc: Loc,
        errors: &mut Vec<Error>,
    ) -> TRes<usize> {
        self.table
            .get_function_result_id(local, loc, self.conf.levenshtein, errors)
    }
}

#[derive(Debug)]
pub struct Colon<U, V> {
    pub left: U,
    pub colon: syn::token::Colon,
    pub right: V,
}

#[derive(Debug)]
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

/// Constant declaration.
pub struct ConstDecl {
    pub const_token: Token![const],
    /// Constant's identifier.
    pub ident: Ident,
    /// Colon token.
    pub colon_token: Token![:],
    /// Constant's type.
    pub ty: Typ,
    /// Equality token.
    pub eq_token: Token![=],
    /// Constant value.
    pub value: Constant,
    /// Closing semicolon.
    pub semi_token: Token![;],
}
impl HasLoc for ConstDecl {
    fn loc(&self) -> Loc {
        Loc::from(self.const_token.span).join(self.semi_token.span)
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
    /// Component's contract.
    pub contract: Contract,
    pub brace: syn::token::Brace,
    /// Component's equations.
    pub equations: Vec<equation::ReactEq>,
    /// User-specified weight.
    pub weight: Option<usize>,
}
impl HasLoc for Component {
    fn loc(&self) -> Loc {
        Loc::from(self.component_token.span).join(self.brace.span.join())
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
    /// User-specified weight.
    pub weight: Option<usize>,
}
impl HasLoc for Function {
    fn loc(&self) -> Loc {
        Loc::from(self.function_token.span).join(self.brace.span.join())
    }
}

/// Things that can appear in a GRust program.
pub enum Item {
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
    ExtFun(ExtFunDecl),
    ExtComp(ExtCompDecl),
    Const(ConstDecl),
}
impl HasLoc for Item {
    fn loc(&self) -> Loc {
        match self {
            Self::Component(c) => c.loc(),
            Self::Function(f) => f.loc(),
            Self::Typedef(t) => t.loc(),
            Self::Service(s) => s.loc(),
            Self::Import(i) => i.loc(),
            Self::Export(e) => e.loc(),
            Self::ExtComp(c) => c.loc(),
            Self::ExtFun(f) => f.loc(),
            Self::Const(c) => c.loc(),
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
