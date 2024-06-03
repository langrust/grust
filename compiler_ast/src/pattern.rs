prelude! {
    syn::{braced, parenthesized, parse::Parse, punctuated::Punctuated, token, Token},
}

/// Typed pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct Typed {
    /// The pattern.
    pub pattern: Box<Pattern>,
    /// The colon token.
    pub colon_token: Token![:],
    /// The type.
    pub typing: Typ,
}
mk_new! { impl Typed =>
    new {
        pattern: Pattern = pattern.into(),
        colon_token: Token![:],
        typing: Typ,
    }
}
impl Typed {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![:])
    }

    pub fn parse(pattern: Box<Pattern>, input: syn::parse::ParseStream) -> syn::Result<Self> {
        let colon_token: Token![:] = input.parse()?;
        let typing = input.parse()?;
        Ok(Typed {
            pattern,
            colon_token,
            typing,
        })
    }
}

/// Structure pattern that matches the structure and its fields.
#[derive(Debug, PartialEq, Clone)]
pub struct Structure {
    /// The structure name.
    pub name: String,
    /// The structure fields with the corresponding patterns to match.
    pub fields: Vec<(String, Option<Pattern>)>,
    /// The rest of the fields
    pub rest: Option<Token![..]>,
}
mk_new! { impl Structure =>
    new {
        name: impl Into<String> = name.into(),
        fields: Vec<(String, Option<Pattern>)>,
        rest: Option<Token![..]>,
    }
}
impl Structure {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(syn::Ident::parse).is_err() {
            return false;
        }
        forked.peek(token::Brace)
    }
}
impl Parse for Structure {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let content;
        let _ = braced!(content in input);
        let mut fields: Punctuated<(syn::Ident, Option<(Token![:], Pattern)>), Token![,]> =
            Punctuated::new();
        let mut rest = None;
        while !content.is_empty() {
            if content.peek(Token![..]) {
                rest = Some(content.parse()?);
                break;
            }

            let member: syn::Ident = content.parse()?;
            let optional_pattern = if content.peek(Token![:]) {
                let colon_token = content.parse()?;
                let pattern = content.parse()?;
                Some((colon_token, pattern))
            } else {
                None
            };
            fields.push_value((member, optional_pattern));

            if content.is_empty() {
                break;
            }
            fields.push_punct(content.parse()?);
        }

        Ok(Structure {
            name: ident.to_string(),
            fields: fields
                .into_iter()
                .map(|(ident, optional_pattern)| {
                    (
                        ident.to_string(),
                        optional_pattern.map(|(_, pattern)| pattern),
                    )
                })
                .collect(),
            rest,
        })
    }
}

/// Enumeration pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct Enumeration {
    /// The enumeration type name.
    pub enum_name: String,
    /// The element name.
    pub elem_name: String,
}
mk_new! { impl Enumeration =>
    new {
        enum_name: impl Into<String> = enum_name.into(),
        elem_name: impl Into<String> = elem_name.into(),
    }
}
impl Enumeration {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        let forked = input.fork();
        if forked.call(syn::Ident::parse).is_err() {
            return false;
        }
        forked.peek(Token![::])
    }
}
impl Parse for Enumeration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident_enum: syn::Ident = input.parse()?;
        let _: Token![::] = input.parse()?;
        let ident_elem: syn::Ident = input.parse()?;
        Ok(Enumeration {
            enum_name: ident_enum.to_string(),
            elem_name: ident_elem.to_string(),
        })
    }
}

/// Tuple pattern that matches tuples.
#[derive(Debug, PartialEq, Clone)]
pub struct Tuple {
    /// The elements of the tuple.
    pub elements: Vec<Pattern>,
}
mk_new! { impl Tuple =>
    new { elements: Vec<Pattern> }
}
impl Tuple {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(token::Paren)
    }
}
impl Parse for Tuple {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let elements: Punctuated<Pattern, Token![,]> = Punctuated::parse_terminated(&content)?;
        Ok(Tuple {
            elements: elements.into_iter().collect(),
        })
    }
}

/// Some pattern that matches when an optional has a value which match the pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct PatSome {
    /// The pattern matching the value.
    pub pattern: Box<Pattern>,
}
mk_new! { impl PatSome =>
    new { pattern : Pattern = pattern.into() }
}
impl PatSome {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(keyword::some)
    }
}
impl Parse for PatSome {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: keyword::some = input.parse()?;
        let content;
        let _ = parenthesized!(content in input);
        let pattern = Box::new(content.parse()?);
        if content.is_empty() {
            Ok(PatSome { pattern })
        } else {
            Err(input.error("expected only one pattern"))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
/// GRust matching pattern AST.
pub enum Pattern {
    /// Constant pattern.
    Constant(Constant),
    /// Identifier pattern.
    Identifier(String),
    /// Typed pattern.
    Typed(Typed),
    /// Structure pattern that matches the structure and its fields.
    Structure(Structure),
    /// Enumeration pattern.
    Enumeration(Enumeration),
    /// Tuple pattern that matches tuples.
    Tuple(Tuple),
    /// Default pattern.
    Default,
}
impl Pattern {
    mk_new! {
        Constant: constant(c: Constant = c)
        Constant: cst(c: Constant = c)
        Identifier: ident(s: impl Into<String> = s.into())
        Typed: typed(t: Typed = t)
        Structure: structure(s: Structure = s)
        Enumeration: enumeration(e: Enumeration = e)
        Tuple: tuple(t: Tuple = t)
        Default: default()
    }
}
impl Parse for Pattern {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut pattern = if input.fork().call(Constant::parse).is_ok() {
            Pattern::Constant(input.parse()?)
        } else if Structure::peek(input) {
            Pattern::Structure(input.parse()?)
        } else if Tuple::peek(input) {
            Pattern::Tuple(input.parse()?)
        } else if Enumeration::peek(input) {
            Pattern::Enumeration(input.parse()?)
        } else if input.fork().peek(Token![_]) {
            let _: Token![_] = input.parse()?;
            Pattern::Default
        } else {
            let ident: syn::Ident = input.parse()?;
            Pattern::Identifier(ident.to_string())
        };

        if Typed::peek(input) {
            pattern = Pattern::Typed(Typed::parse(Box::new(pattern), input)?);
        }

        Ok(pattern)
    }
}

#[cfg(test)]
mod parse_pattern {
    use super::*;

    #[test]
    fn should_parse_constant() {
        let pattern: Pattern = syn::parse_quote! {1};
        let control = Pattern::cst(Constant::int(syn::parse_quote! {1}));
        assert_eq!(pattern, control)
    }

    #[test]
    fn should_parse_identifier() {
        let pattern: Pattern = syn::parse_quote! {x};
        let control = Pattern::ident("x");
        assert_eq!(pattern, control)
    }

    #[test]
    fn should_parse_structure() {
        let pattern: Pattern = syn::parse_quote! {
            Point {
                x: 0,
                y: _,
            }
        };
        let control = Pattern::structure(Structure::new(
            "Point",
            vec![
                (
                    "x".into(),
                    Some(Pattern::cst(Constant::int(syn::parse_quote! {0}))),
                ),
                ("y".into(), Some(Pattern::default())),
            ],
            None,
        ));
        assert_eq!(pattern, control)
    }

    #[test]
    fn should_parse_structure_with_unrenamed_field() {
        let pattern: Pattern = syn::parse_quote! {
            Point { x: 0, y, }
        };
        let control = Pattern::structure(Structure::new(
            "Point",
            vec![
                (
                    "x".into(),
                    Some(Pattern::cst(Constant::int(syn::parse_quote! {0}))),
                ),
                ("y".into(), None),
            ],
            None,
        ));
        assert_eq!(pattern, control)
    }

    #[test]
    fn should_parse_structure_with_unspecified_field() {
        let pattern: Pattern = syn::parse_quote! {
            Point { x: 0, .. }
        };
        let control = Pattern::structure(Structure::new(
            "Point",
            vec![(
                "x".into(),
                Some(Pattern::cst(Constant::int(syn::parse_quote! {0}))),
            )],
            Some(syn::parse_quote!(..)),
        ));
        assert_eq!(pattern, control)
    }

    #[test]
    fn should_parse_tuple() {
        let pattern: Pattern = syn::parse_quote! {(x, 0)};
        let control = Pattern::tuple(Tuple::new(vec![
            Pattern::ident("x"),
            Pattern::cst(Constant::int(syn::parse_quote! {0})),
        ]));
        assert_eq!(pattern, control)
    }

    #[test]
    fn should_parse_enumeration() {
        let pattern: Pattern = syn::parse_quote! {Color::Pink};
        let control = Pattern::enumeration(Enumeration::new("Color", "Pink"));
        assert_eq!(pattern, control)
    }

    #[test]
    fn should_parse_default() {
        let pattern: Pattern = syn::parse_quote! {_};
        let control = Pattern::default();
        assert_eq!(pattern, control)
    }
}
