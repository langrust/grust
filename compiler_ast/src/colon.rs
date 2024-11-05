prelude! {
    syn::Parse,
}

pub struct Colon<U: Parse, V: Parse> {
    pub left: U,
    pub colon: Token![:],
    pub right: V,
}
impl<U: Parse, V: Parse> Parse for Colon<U, V> {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        Ok(Self {
            left: input.parse()?,
            colon: input.parse()?,
            right: input.parse()?,
        })
    }
}
