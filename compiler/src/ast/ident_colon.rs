use syn::{parse::Parse, Token};

pub struct IdentColon<T: Parse> {
    pub ident: syn::Ident,
    pub colon: Token![:],
    pub elem: T,
}
impl<T: Parse> Parse for IdentColon<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            colon: input.parse()?,
            elem: input.parse()?,
        })
    }
}

pub struct PathColon<T: Parse> {
    pub path: syn::Path,
    pub colon: Token![:],
    pub elem: T,
}
impl<T: Parse> Parse for PathColon<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            colon: input.parse()?,
            elem: input.parse()?,
        })
    }
}
