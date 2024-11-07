//! LIR [Pattern] module.

prelude! {}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust matching pattern LIR (resemble to the AST).
pub enum Pattern {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier.
        name: String,
    },
    /// Literal pattern, matches the given literal (constant).
    Literal {
        /// The matching literal (constant).
        literal: Constant,
    },
    /// Typed pattern.
    Typed {
        /// The pattern.
        pattern: Box<Pattern>,
        /// The type.
        typ: Typ,
    },
    /// Structure pattern that matches the structure and its fields.
    Structure {
        /// The structure id.
        name: String,
        /// The structure fields with the corresponding patterns to match.
        fields: Vec<(String, Pattern)>,
    },
    /// Enumeration pattern.
    Enumeration {
        /// The enumeration type name.
        enum_name: String,
        /// The element name.
        elem_name: String,
        /// The optional element of the enumeration.
        element: Option<Box<Pattern>>,
    },
    /// Tuple pattern that matches tuples.
    Tuple {
        /// The elements of the tuple.
        elements: Vec<Pattern>,
    },
    /// Ok pattern that matches when a result has a value which match the pattern.
    Ok {
        /// The pattern matching the value.
        pattern: Box<Pattern>,
    },
    /// Err pattern, matches when the result does not have a value.
    Err,
    /// Some pattern that matches when an optional has a value which match the pattern.
    Some {
        /// The pattern matching the value.
        pattern: Box<Pattern>,
    },
    /// None pattern, matches when the optional does not have a value.
    None,
    /// The default pattern that matches anything.
    Default,
}

mk_new! { impl Pattern =>
    Identifier: ident { name: impl Into<String> = name.into() }
    Literal: literal {literal: Constant }
    Typed: typed {
        pattern: Self = Box::new(pattern),
        typ: Typ
    }
    Structure: structure {
        name: impl Into<String> = name.into(),
        fields: Vec<(String, Self)>
    }
    Enumeration: enumeration {
        enum_name: impl Into<String> = enum_name.into(),
        elem_name: impl Into<String> = elem_name.into(),
        element: Option<Self> = element.map(Box::new),
    }
    Tuple: tuple { elements: Vec<Self> }
    Ok: ok { pattern: Self = Box::new(pattern) }
    Err: err()
    Some: some { pattern: Self = Box::new(pattern) }
    None: none()
    Default: default()
}

impl Pattern {
    pub fn into_syn(self) -> syn::Pat {
        use syn::*;
        match self {
            Pattern::Literal { literal } => match literal {
                Constant::Integer(i) => Pat::Lit(parse_quote! { #i }),
                Constant::Float(f) => Pat::Lit(parse_quote! { #f }),
                Constant::Boolean(b) => Pat::Lit(parse_quote! { #b }),
                Constant::Unit(paren_token) => Pat::Tuple(PatTuple {
                    attrs: vec![],
                    paren_token,
                    elems: Default::default(),
                }),
                Constant::Default => parse_quote! { Default::default() },
            },
            Pattern::Identifier { name } => Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: Ident::new(&name, Span::call_site()),
                subpat: None,
            }),
            Pattern::Default => Pat::Wild(PatWild {
                attrs: vec![],
                underscore_token: Default::default(),
            }),
            Pattern::Ok { pattern } => Pat::TupleStruct(PatTupleStruct {
                attrs: vec![],
                path: parse_quote! { Ok },
                elems: vec![pattern.into_syn()].into_iter().collect(),
                paren_token: Default::default(),
                qself: None,
            }),
            Pattern::Err => parse_quote! { Err(()) },
            Pattern::Some { pattern } => Pat::TupleStruct(PatTupleStruct {
                attrs: vec![],
                path: parse_quote! { Some },
                elems: vec![pattern.into_syn()].into_iter().collect(),
                paren_token: Default::default(),
                qself: None,
            }),
            Pattern::None => parse_quote! { None },
            Pattern::Typed { pattern, .. } => pattern.into_syn(),
            Pattern::Structure { name, fields } => Pat::Struct(PatStruct {
                attrs: vec![],
                path: format_ident!("{name}").into(),
                brace_token: Default::default(),
                fields: fields
                    .into_iter()
                    .map(|(name, pattern)| FieldPat {
                        attrs: vec![],
                        member: Member::Named(Ident::new(&name, Span::call_site())),
                        colon_token: Some(Default::default()),
                        pat: Box::new(pattern.into_syn()),
                    })
                    .collect(),
                qself: None,
                rest: None,
            }),
            Pattern::Enumeration {
                enum_name,
                elem_name,
                element,
            } => {
                let ty = Ident::new(&enum_name, Span::call_site());
                let cons = Ident::new(&elem_name, Span::call_site());
                if let Some(pattern) = element {
                    let inner = pattern.into_syn();
                    parse_quote! { #ty::#cons(#inner) }
                } else {
                    parse_quote! { #ty::#cons }
                }
            }
            Pattern::Tuple { elements } => {
                let elements = elements
                    .into_iter()
                    .map(|element| -> Pat { element.into_syn() });
                parse_quote! { (#(#elements),*) }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_a_rust_ast_default_pattern_from_a_lir_default_pattern() {
        let pattern = Pattern::Default;
        let control = parse_quote! { _ };
        assert_eq!(pattern.into_syn(), control)
    }

    #[test]
    fn should_create_a_rust_ast_default_pattern_from_a_lir_none_pattern() {
        let pattern = Pattern::None;
        let control = parse_quote! { None };
        assert_eq!(pattern.into_syn(), control)
    }

    #[test]
    fn should_create_a_rust_ast_tuple_structure_pattern_from_a_lir_some_pattern() {
        let pattern = Pattern::Some {
            pattern: Box::new(Pattern::Default),
        };

        let control = parse_quote! { Some(_) };
        assert_eq!(pattern.into_syn(), control)
    }

    #[test]
    fn should_create_a_rust_ast_literal_pattern_from_a_lir_constant_pattern() {
        let pattern = Pattern::Literal {
            literal: Constant::Integer(parse_quote!(1i64)),
        };

        let control = parse_quote! { 1i64 };
        assert_eq!(pattern.into_syn(), control)
    }

    #[test]
    fn should_create_a_rust_ast_identifier_pattern_owned_and_immutable_from_a_lir_identifier_pattern(
    ) {
        let pattern = Pattern::ident("x");

        let control = parse_quote! { x };
        assert_eq!(pattern.into_syn(), control)
    }

    #[test]
    fn should_create_a_rust_ast_structure_pattern_from_a_lir_structure_pattern() {
        let pattern = Pattern::Structure {
            name: "Point".into(),
            fields: vec![
                ("x".into(), Pattern::Default),
                ("y".into(), Pattern::ident("y")),
            ],
        };

        let control = parse_quote! { Point { x: _, y : y } };
        assert_eq!(pattern.into_syn(), control)
    }
}
