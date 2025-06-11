//! [ArrayAlias] module.

prelude! {}

/// An array alias definition.
#[derive(Debug, PartialEq)]
pub struct ArrayAlias {
    /// The array's name.
    pub name: Ident,
    /// The array's type.
    pub array_type: Typ,
    /// The array's size.
    pub size: usize,
}
mk_new! { impl ArrayAlias =>
    new {
        name: impl Into<Ident> = name.into(),
        array_type: Typ,
        size: usize,
    }
}

pub struct ArrayAliasTokens<'a> {
    aa: &'a ArrayAlias,
    public: bool,
}
impl ArrayAlias {
    pub fn prepare_tokens(&self, public: bool) -> ArrayAliasTokens<'_> {
        ArrayAliasTokens { aa: self, public }
    }
}

impl<'a> ToTokens for ArrayAliasTokens<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let size = self.aa.size;
        let name = &self.aa.name;
        let inner_typ = &self.aa.array_type;
        let pub_token = if self.public {
            quote! {pub}
        } else {
            quote! {}
        };
        quote! {
            #pub_token type #name = [#inner_typ; #size];
        }
        .to_tokens(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_type_alias_from_ir2_array_alias() {
        let array_alias = ArrayAlias {
            name: Loc::test_id("Matrix5x5"),
            array_type: Typ::array(Typ::int(), 5),
            size: 5,
        }
        .prepare_tokens(true)
        .to_token_stream();
        println!("{}", array_alias.to_token_stream());

        let control = parse_quote! { pub type Matrix5x5 = [[i64; 5usize]; 5usize];};
        let array_alias: syn::ItemType = parse_quote!(#array_alias);
        assert_eq!(array_alias, control)
    }
}
