prelude! {
    syn::{
        parse::Parse,
        punctuated::Punctuated,
        braced, bracketed, parenthesized, token, Token,
    },
    operator::{BinaryOperator, UnaryOperator},
}

use super::keyword;

pub trait ParsePrec
where
    Self: Sized,
{
    fn parse_term(input: syn::parse::ParseStream) -> syn::Result<Self>;
    fn parse_prec1(input: syn::parse::ParseStream) -> syn::Result<Self>;
    fn parse_prec2(input: syn::parse::ParseStream) -> syn::Result<Self>;
    fn parse_prec3(input: syn::parse::ParseStream) -> syn::Result<Self>;
    fn parse_prec4(input: syn::parse::ParseStream) -> syn::Result<Self>;
}

/// Unop expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Unop<E> {
    /// The unary operator.
    pub op: UnaryOperator,
    /// The input expression.
    pub expression: Box<E>,
}
impl<E> Unop<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        UnaryOperator::peek(input)
    }
}
impl<E> Parse for Unop<E>
where
    E: ParsePrec,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let expression = Box::new(E::parse_term(input)?);
        Ok(Unop { op, expression })
    }
}

/// Binop expression.
///
/// TODO: precedence
#[derive(Debug, PartialEq, Clone)]
pub struct Binop<E> {
    /// The unary operator.
    pub op: BinaryOperator,
    /// The left expression.
    pub left_expression: Box<E>,
    /// The right expression.
    pub right_expression: Box<E>,
}

mk_new! { impl{E} Binop<E> =>
    new {
        op : BinaryOperator,
        left_expression: E = left_expression.into(),
        right_expression: E = right_expression.into(),
    }

}

impl<E> Binop<E>
where
    E: ParsePrec,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        BinaryOperator::peek(input)
    }
    pub fn parse_term(lhs: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let rhs = E::parse_term(input)?;
        Ok(Binop::new(op, lhs, rhs))
    }
    pub fn parse_prec1(lhs: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let rhs = E::parse_prec1(input)?;
        Ok(Binop::new(op, lhs, rhs))
    }
    pub fn parse_prec2(lhs: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let rhs = E::parse_prec2(input)?;
        Ok(Binop::new(op, lhs, rhs))
    }
    pub fn parse_prec3(lhs: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let rhs = E::parse_prec3(input)?;
        Ok(Binop::new(op, lhs, rhs))
    }
    pub fn parse_prec4(lhs: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = input.parse()?;
        let rhs = E::parse_prec4(input)?;
        Ok(Binop::new(op, lhs, rhs))
    }
}

/// IfThenElse expression.
#[derive(Debug, PartialEq, Clone)]
pub struct IfThenElse<E> {
    /// The test expression.
    pub expression: Box<E>,
    /// The 'true' expression.
    pub true_expression: Box<E>,
    /// The 'false' expression.
    pub false_expression: Box<E>,
}

mk_new! { impl{E} IfThenElse<E> =>
    new {
        expression: E = expression.into(),
        true_expression: E = true_expression.into(),
        false_expression: E = false_expression.into()
    }
}

impl<E> IfThenElse<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![if])
    }
}
impl<E> Parse for IfThenElse<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![if] = input.parse()?;
        let expression = Box::new(input.parse()?);
        let _: keyword::then = input.parse()?;
        let true_expression = Box::new(input.parse()?);
        let _: Token![else] = input.parse()?;
        let false_expression = Box::new(input.parse()?);
        Ok(IfThenElse {
            expression,
            true_expression,
            false_expression,
        })
    }
}

/// Application expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Application<E> {
    /// The expression applied.
    pub function_expression: Box<E>,
    /// The inputs to the expression.
    pub inputs: Vec<E>,
}

mk_new! { impl{E} Application<E> =>
    new {
        function_expression: E = function_expression.into(),
        inputs: Vec<E>,
    }
}

impl<E> Application<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(token::Paren)
    }

    pub fn parse(function: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = syn::parenthesized!(content in input);
        let inputs: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Application::new(function, inputs.into_iter().collect()))
    }
}
impl<E> Parse for Application<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let function = input.parse()?;
        let content;
        let _ = syn::parenthesized!(content in input);
        let inputs: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Application::new(function, inputs.into_iter().collect()))
    }
}

/// Abstraction expression with inputs types.
#[derive(Debug, PartialEq, Clone)]
pub struct TypedAbstraction<E> {
    /// The inputs to the abstraction.
    pub inputs: Vec<(String, Typ)>,
    /// The expression abstracted.
    pub expression: Box<E>,
}

mk_new! { impl{E} TypedAbstraction<E> =>
    new {
        inputs: Vec<(String, Typ)>,
        expression: E = expression.into(),
    }
}

impl<E> TypedAbstraction<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![|])
    }
}
impl<E> Parse for TypedAbstraction<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![|] = input.parse()?;
        let mut inputs: Punctuated<Colon<syn::Ident, Typ>, Token![,]> = Punctuated::new();
        loop {
            if input.peek(Token![|]) {
                break;
            }
            let value = input.parse()?;
            inputs.push_value(value);
            if input.peek(Token![|]) {
                break;
            }
            let punct: Token![,] = input.parse()?;
            inputs.push_punct(punct);
        }
        let _: Token![|] = input.parse()?;
        let expression: E = input.parse()?;
        Ok(TypedAbstraction::new(
            inputs
                .into_iter()
                .map(|Colon { left, right, .. }| (left.to_string(), right))
                .collect(),
            expression,
        ))
    }
}

/// Structure expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Structure<E> {
    /// The structure name.
    pub name: String,
    /// The fields associated with their expressions.
    pub fields: Vec<(String, E)>,
}

mk_new! { impl{E} Structure<E> =>
    new {
        name: impl Into<String> = name.into(),
        fields: Vec<(String, E)>,
    }
}

impl<E> Structure<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        forked.call(Structure::<E>::parse).is_ok()
    }
}
impl<E> Parse for Structure<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let content;
        let _ = braced!(content in input);
        let fields: Punctuated<Colon<syn::Ident, E>, Token![,]> =
            Punctuated::parse_terminated(&content)?;
        Ok(Structure::new(
            ident.to_string(),
            fields
                .into_iter()
                .map(|Colon { left, right, .. }| (left.to_string(), right))
                .collect(),
        ))
    }
}

/// Tuple expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple<E> {
    /// The elements.
    pub elements: Vec<E>,
}

mk_new! { impl{E} Tuple<E> =>
    new { elements: Vec<E> }
}

impl<E> Tuple<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(token::Paren)
    }
}
impl<E> Parse for Tuple<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let elements: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Tuple::new(elements.into_iter().collect()))
    }
}

/// Enumeration expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration<E> {
    /// The enumeration name.
    pub enum_name: String,
    /// The enumeration element.
    pub elem_name: String,
    /// Marker for the unused type param.
    pub mark: std::marker::PhantomData<E>,
}

impl<E> Enumeration<E> {
    pub fn new(enum_name: impl Into<String>, elem_name: impl Into<String>) -> Self {
        Self {
            enum_name: enum_name.into(),
            elem_name: elem_name.into(),
            mark: std::marker::PhantomData,
        }
    }
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(syn::Ident::parse).is_err() {
            return false;
        }
        forked.peek(Token![::])
    }
}
impl<E> Parse for Enumeration<E> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident_enum: syn::Ident = input.parse()?;
        let _: Token![::] = input.parse()?;
        let ident_elem: syn::Ident = input.parse()?;
        Ok(Enumeration::new(
            ident_enum.to_string(),
            ident_elem.to_string(),
        ))
    }
}

/// Array expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Array<E> {
    /// The elements inside the array.
    pub elements: Vec<E>,
}

mk_new! { impl{E} Array<E> =>
    new { elements: Vec<E> }
}

impl<E> Array<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(token::Bracket)
    }
}
impl<E> Parse for Array<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = bracketed!(content in input);
        let elements: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Array::new(elements.into_iter().collect()))
    }
}

/// Arm for matching expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Arm<E> {
    /// The pattern to match.
    pub pattern: Pattern,
    /// The optional guard.
    pub guard: Option<E>,
    /// The expression.
    pub expression: E,
}

mk_new! { impl{E} Arm<E> =>
    new_with_guard {
        pattern: Pattern,
        expression: E,
        guard: Option<E>,
    }
}
impl<E> Arm<E> {
    pub fn new(pattern: Pattern, expression: E) -> Self {
        Self {
            pattern,
            expression,
            guard: None,
        }
    }
}

impl<E> Parse for Arm<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern = input.parse()?;
        let guard = {
            if input.fork().peek(Token![if]) {
                let _: Token![if] = input.parse()?;
                let guard = input.parse()?;
                Some(guard)
            } else {
                None
            }
        };
        let _: Token![=>] = input.parse()?;
        let expression = input.parse()?;
        Ok(Arm {
            pattern,
            guard,
            expression,
        })
    }
}

/// Pattern matching expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Match<E> {
    /// The expression to match.
    pub expression: Box<E>,
    /// The different matching cases.
    pub arms: Vec<Arm<E>>,
}

mk_new! { impl{E} Match<E> =>
    new {
        expression: E = expression.into(),
        arms: Vec<Arm<E>>,
    }
}

impl<E> Match<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![match])
    }
}
impl<E> Parse for Match<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![match] = input.parse()?;
        let expression = input.parse()?;
        let content;
        let _ = braced!(content in input);
        let arms: Punctuated<Arm<E>, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Match::new(expression, arms.into_iter().collect()))
    }
}

/// Field access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<E> {
    /// The structure expression.
    pub expression: Box<E>,
    /// The field to access.
    pub field: String,
}

mk_new! { impl{E} FieldAccess<E> =>
    new {
        expression: E = expression.into(),
        field: impl Into<String> = field.into(),
    }
}

impl<E> FieldAccess<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.call(syn::Ident::parse).is_ok()
    }

    pub fn parse(expression: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![.] = input.parse()?;
        let field: syn::Ident = input.parse()?;
        Ok(FieldAccess::new(expression, field.to_string()))
    }
}
impl<E> Parse for FieldAccess<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = input.parse()?;
        let _: Token![.] = input.parse()?;
        let field: syn::Ident = input.parse()?;
        Ok(FieldAccess::new(expression, field.to_string()))
    }
}

/// Tuple element access expression.
#[derive(Debug, PartialEq, Clone)]
pub struct TupleElementAccess<E> {
    /// The tuple expression.
    pub expression: Box<E>,
    /// The element to access.
    pub element_number: usize,
}

mk_new! { impl{E} TupleElementAccess<E> =>
    new {
        expression: E = expression.into(),
        element_number: usize,
    }
}

impl<E> TupleElementAccess<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.call(syn::LitInt::parse).is_ok()
    }

    pub fn parse(expression: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![.] = input.parse()?;
        let element_number: syn::LitInt = input.parse()?;
        Ok(TupleElementAccess::new(
            expression,
            element_number.base10_parse().unwrap(),
        ))
    }
}
impl<E> Parse for TupleElementAccess<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = input.parse()?;
        let _: Token![.] = input.parse()?;
        let element_number: syn::LitInt = input.parse()?;
        Ok(TupleElementAccess::new(
            expression,
            element_number.base10_parse().unwrap(),
        ))
    }
}

/// Array map operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Map<E> {
    /// The array expression.
    pub expression: Box<E>,
    /// The function expression.
    pub function_expression: Box<E>,
}

mk_new! { impl{E} Map<E> =>
    new {
        expression: E = expression.into(),
        function_expression: E = function_expression.into(),
    }
}

impl<E> Map<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::map)
    }

    pub fn parse(expression: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![.] = input.parse()?;
        let _: keyword::map = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let function_expression = content.parse()?;
        if content.is_empty() {
            Ok(Self::new(expression, function_expression))
        } else {
            Err(input.error("expected only one expression"))
        }
    }
}
impl<E> Parse for Map<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = input.parse()?;
        let _: Token![.] = input.parse()?;
        let _: keyword::map = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let function_expression = content.parse()?;
        if content.is_empty() {
            Ok(Self::new(expression, function_expression))
        } else {
            Err(input.error("expected only one expression"))
        }
    }
}

/// Array fold operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Fold<E> {
    /// The array expression.
    pub expression: Box<E>,
    /// The initialization expression.
    pub initialization_expression: Box<E>,
    /// The function expression.
    pub function_expression: Box<E>,
}

mk_new! { impl{E} Fold<E> =>
    new {
        expression: E = expression.into(),
        initialization_expression: E = initialization_expression.into(),
        function_expression: E = function_expression.into(),
    }
}

impl<E> Fold<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::fold)
    }

    pub fn parse(expression: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![.] = input.parse()?;
        let _: keyword::fold = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let init = content.parse()?;
        let _: Token![,] = content.parse()?;
        let function = content.parse()?;
        if content.is_empty() {
            Ok(Self::new(expression, init, function))
        } else {
            Err(input.error("expected only two expressions"))
        }
    }
}
impl<E> Parse for Fold<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = input.parse()?;
        let _: Token![.] = input.parse()?;
        let _: keyword::fold = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let init = content.parse()?;
        let _: Token![,] = content.parse()?;
        let function = content.parse()?;
        if content.is_empty() {
            Ok(Self::new(expression, init, function))
        } else {
            Err(input.error("expected only two expressions"))
        }
    }
}

/// Array sort operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Sort<E> {
    /// The array expression.
    pub expression: Box<E>,
    /// The function expression.
    pub function_expression: Box<E>,
}

mk_new! { impl{E} Sort<E> =>
    new {
        expression: E = expression.into(),
        function_expression: E = function_expression.into(),
    }
}

impl<E> Sort<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(token::Dot::parse).is_err() {
            return false;
        }
        forked.peek(keyword::sort)
    }

    pub fn parse(expression: E, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![.] = input.parse()?;
        let _: keyword::sort = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let function_expression = content.parse()?;
        if content.is_empty() {
            Ok(Self::new(expression, function_expression))
        } else {
            Err(input.error("expected only one expression"))
        }
    }
}
impl<E> Parse for Sort<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = input.parse()?;
        let _: Token![.] = input.parse()?;
        let _: keyword::sort = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let function_expression = content.parse()?;
        if content.is_empty() {
            Ok(Self::new(expression, function_expression))
        } else {
            Err(input.error("expected only one expression"))
        }
    }
}

/// Arrays zip operator expression.
#[derive(Debug, PartialEq, Clone)]
pub struct Zip<E> {
    /// The array expressions.
    pub arrays: Vec<E>,
}

mk_new! { impl{E} Zip<E> =>
    new { arrays: Vec<E> }
}

impl<E> Zip<E>
where
    E: Parse,
{
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::zip)
    }
}
impl<E> Parse for Zip<E>
where
    E: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: keyword::zip = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let arrays: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Zip::new(arrays.into_iter().collect()))
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust expression AST.
pub enum Expr {
    /// Constant expression.
    Constant(Constant),
    /// Identifier expression.
    Identifier(String),
    /// Unop expression.
    Unop(Unop<Self>),
    /// Binop expression.
    Binop(Binop<Self>),
    /// IfThenElse expression.
    IfThenElse(IfThenElse<Self>),
    /// Application expression.
    Application(Application<Self>),
    /// Abstraction expression with inputs types.
    TypedAbstraction(TypedAbstraction<Self>),
    /// Structure expression.
    Structure(Structure<Self>),
    /// Tuple expression.
    Tuple(Tuple<Self>),
    /// Enumeration expression.
    Enumeration(Enumeration<Self>),
    /// Array expression.
    Array(Array<Self>),
    /// Pattern matching expression.
    Match(Match<Self>),
    /// Field access expression.
    FieldAccess(FieldAccess<Self>),
    /// Tuple element access expression.
    TupleElementAccess(TupleElementAccess<Self>),
    /// Array map operator expression.
    Map(Map<Self>),
    /// Array fold operator expression.
    Fold(Fold<Self>),
    /// Array sort operator expression.
    Sort(Sort<Self>),
    /// Arrays zip operator expression.
    Zip(Zip<Self>),
}

mk_new! { impl Expr =>
    Constant: constant (val: Constant = val)
    Constant: cst (val: Constant = val)
    Identifier: ident (val: impl Into<String> = val.into())
    Unop: unop (val: Unop<Self> = val)
    Binop: binop (val: Binop<Self> = val)
    IfThenElse: ite (val: IfThenElse<Self> = val)
    Application: app (val: Application<Self> = val)
    TypedAbstraction: typed_abstraction (val: TypedAbstraction<Self> = val)
    Structure: structure (val: Structure<Self> = val)
    Tuple: tuple (val: Tuple<Self> = val)
    Enumeration: enumeration (val: Enumeration<Self> = val)
    Array: array (val: Array<Self> = val)
    Match: pat_match (val: Match<Self> = val)
    FieldAccess: field_access (val: FieldAccess<Self> = val)
    TupleElementAccess: tuple_access (val: TupleElementAccess<Self> = val)
    Map: map (val: Map<Self> = val)
    Fold: fold (val: Fold<Self> = val)
    Sort: sort (val: Sort<Self> = val)
    Zip: zip (val: Zip<Self> = val)
}

impl ParsePrec for Expr {
    fn parse_term(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = if input.fork().call(Constant::parse).is_ok() {
            Self::cst(input.parse()?)
        } else if Unop::<Self>::peek(input) {
            Self::unop(input.parse()?)
        } else if Zip::<Self>::peek(input) {
            Self::zip(input.parse()?)
        } else if Match::<Self>::peek(input) {
            Self::pat_match(input.parse()?)
        } else if Tuple::<Self>::peek(input) {
            let mut tuple: Tuple<Self> = input.parse()?;
            if tuple.elements.len() == 1 {
                tuple.elements.pop().unwrap()
            } else {
                Self::tuple(tuple)
            }
        } else if Array::<Self>::peek(input) {
            Self::array(input.parse()?)
        } else if Structure::<Self>::peek(input) {
            Self::structure(input.parse()?)
        } else if Enumeration::<Self>::peek(input) {
            Self::enumeration(input.parse()?)
        } else if input.fork().call(syn::Ident::parse).is_ok() {
            let ident: syn::Ident = input.parse()?;
            Self::ident(ident.to_string())
        } else {
            return Err(input.error("expected expression"));
        };

        loop {
            if Sort::<Self>::peek(input) {
                expression = Self::sort(Sort::parse(expression, input)?);
            } else if Map::<Self>::peek(input) {
                expression = Self::map(Map::parse(expression, input)?)
            } else if Fold::<Self>::peek(input) {
                expression = Self::fold(Fold::parse(expression, input)?)
            } else if TupleElementAccess::<Self>::peek(input) {
                expression = Self::tuple_access(TupleElementAccess::parse(expression, input)?)
            } else if FieldAccess::<Self>::peek(input) {
                expression = Self::field_access(FieldAccess::parse(expression, input)?)
            } else if Application::<Self>::peek(input) {
                expression = Self::app(Application::parse(expression, input)?)
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec1(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = Expr::parse_term(input)?;

        loop {
            if BinaryOperator::peek_prec1(input) {
                expression = Expr::binop(Binop::parse_term(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec2(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = Expr::parse_prec1(input)?;

        loop {
            if BinaryOperator::peek_prec2(input) {
                expression = Expr::Binop(Binop::parse_prec1(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec3(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = Expr::parse_prec2(input)?;

        loop {
            if BinaryOperator::peek_prec3(input) {
                expression = Expr::binop(Binop::parse_prec2(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
    fn parse_prec4(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression = Expr::parse_prec3(input)?;

        loop {
            if BinaryOperator::peek_prec4(input) {
                expression = Expr::binop(Binop::parse_prec3(expression, input)?);
            } else {
                break;
            }
        }
        Ok(expression)
    }
}
impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expression = if TypedAbstraction::<Self>::peek(input) {
            Self::typed_abstraction(input.parse()?)
        } else if IfThenElse::<Self>::peek(input) {
            Self::ite(input.parse()?)
        } else {
            Self::parse_prec4(input)?
        };

        Ok(expression)
    }
}

#[cfg(test)]
mod parse_expression {
    prelude! {
        expr::*,
        operator::BinaryOperator,
    }

    #[test]
    fn should_parse_constant() {
        let expression: Expr = syn::parse_quote! {1};
        let control = Expr::cst(Constant::int(syn::parse_quote! {1}));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_identifier() {
        let expression: Expr = syn::parse_quote! {x};
        let control = Expr::ident("x");
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_application() {
        let expression: Expr = syn::parse_quote! {f(x)};
        let control = Expr::app(Application::new(Expr::ident("f"), vec![Expr::ident("x")]));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_binop() {
        let expression: Expr = syn::parse_quote! {a+b};
        let control = Expr::binop(Binop::new(
            BinaryOperator::Add,
            Expr::ident("a"),
            Expr::ident("b"),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_binop_with_precedence() {
        let expression: Expr = syn::parse_quote! {a+b*c};
        let control = Expr::binop(Binop::new(
            BinaryOperator::Add,
            Expr::ident("a"),
            Expr::Binop(Binop::new(
                BinaryOperator::Mul,
                Expr::ident("b"),
                Expr::ident("c"),
            )),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_typed_abstraction() {
        let expression: Expr = syn::parse_quote! {|x: int| f(x)};
        let control = Expr::typed_abstraction(TypedAbstraction::new(
            vec![("x".into(), Typ::int())],
            Expr::app(Application::new(Expr::ident("f"), vec![Expr::ident("x")])),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_structure() {
        let expression: Expr = syn::parse_quote! {Point {x: 0, y: 1}};
        let control = Expr::structure(Structure::new(
            "Point",
            vec![
                ("x".into(), Expr::cst(Constant::int(syn::parse_quote! {0}))),
                ("y".into(), Expr::cst(Constant::int(syn::parse_quote! {1}))),
            ],
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple() {
        let expression: Expr = syn::parse_quote! {(x, 0)};
        let control = Expr::tuple(Tuple::new(vec![
            Expr::ident("x"),
            Expr::cst(Constant::int(syn::parse_quote! {0})),
        ]));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_enumeration() {
        let expression: Expr = syn::parse_quote! {Color::Pink};
        let control = Expr::enumeration(Enumeration::new("Color", "Pink"));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_array() {
        let expression: Expr = syn::parse_quote! {[1, 2, 3]};
        let control = Expr::array(Array::new(vec![
            Expr::cst(Constant::int(syn::parse_quote! {1})),
            Expr::cst(Constant::int(syn::parse_quote! {2})),
            Expr::cst(Constant::int(syn::parse_quote! {3})),
        ]));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_match() {
        let expression: Expr = syn::parse_quote! {
            match a {
                Point {x: 0, y: _} => 0,
                Point {x: x, y: _} if f(x) => -1,
                _ => 1,
            }
        };
        let control = Expr::pat_match(Match::new(
            Expr::ident("a"),
            vec![
                Arm::new(
                    Pattern::structure(pattern::Structure::new(
                        "Point",
                        vec![
                            (
                                "x".into(),
                                Some(Pattern::cst(Constant::int(syn::parse_quote! {0}))),
                            ),
                            ("y".into(), Some(Pattern::default())),
                        ],
                        None,
                    )),
                    Expr::Constant(Constant::Integer(syn::parse_quote! {0})),
                ),
                Arm::new_with_guard(
                    Pattern::Structure(pattern::Structure::new(
                        "Point",
                        vec![
                            ("x".into(), Some(Pattern::ident("x"))),
                            ("y".into(), Some(Pattern::default())),
                        ],
                        None,
                    )),
                    Expr::cst(Constant::int(syn::parse_quote! {-1})),
                    Some(Expr::app(Application::new(
                        Expr::ident("f"),
                        vec![Expr::ident("x")],
                    ))),
                ),
                Arm::new(
                    Pattern::Default,
                    Expr::cst(Constant::int(syn::parse_quote! {1})),
                ),
            ],
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_field_access() {
        let expression: Expr = syn::parse_quote! {p.x};
        let control = Expr::field_access(FieldAccess::new(Expr::ident("p"), "x"));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_tuple_element_access() {
        let expression: Expr = syn::parse_quote! {t.0};
        let control = Expr::tuple_access(TupleElementAccess::new(Expr::ident("t"), 0));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_map() {
        let expression: Expr = syn::parse_quote! {a.map(f)};
        let control = Expr::map(Map::new(Expr::ident("a"), Expr::ident("f")));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_fold() {
        let expression: Expr = syn::parse_quote! {a.fold(0, sum)};
        let control = Expr::fold(Fold::new(
            Expr::ident("a"),
            Expr::cst(Constant::int(syn::parse_quote! {0})),
            Expr::ident("sum"),
        ));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_sort() {
        let expression: Expr = syn::parse_quote! {a.sort(order)};
        let control = Expr::sort(Sort::new(Expr::ident("a"), Expr::ident("order")));
        assert_eq!(expression, control)
    }

    #[test]
    fn should_parse_zip() {
        let expression: Expr = syn::parse_quote! {zip(a, b, c)};
        let control = Expr::zip(Zip::new(vec![
            Expr::ident("a"),
            Expr::ident("b"),
            Expr::ident("c"),
        ]));
        assert_eq!(expression, control)
    }
}
