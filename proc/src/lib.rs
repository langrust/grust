use compiler::{handle_tokens, TokenStream};

#[proc_macro]
pub fn grust(tokens: TokenStream) -> TokenStream {
    handle_tokens(tokens)
}
