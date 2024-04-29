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
