extern crate proc_macro;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, Result, Type};

enum StreamType {
    Push,
    PushTimeout,
    None,
}

/// # Proc macro.
///
/// ## Usage
///
/// ```rust
/// #[stream(push, item = T)]
/// struct MyPush<T, U, Push>
///     where T: Clone,
/// {
///     first: T,
///     second: U,
///     third: Push,
/// }
/// ```
#[proc_macro_attribute]
pub fn stream(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // get "item" param in args
    let mut ty: StreamType = StreamType::None;
    let mut item: Option<Type> = None;
    let item_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("item") {
            item = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("push") {
            if let StreamType::None = ty {
                ty = StreamType::Push;
                Ok(())
            } else {
                Err(meta.error("only one argument [push] or [timeout]"))
            }
        } else if meta.path.is_ident("timeout") {
            if let StreamType::None = ty {
                ty = StreamType::PushTimeout;
                Ok(())
            } else {
                Err(meta.error("only one argument [push] or [timeout]"))
            }
        } else {
            Err(meta.error("`stream` arguments can only be [item = T], [push] or [timeout]"))
        }
    });
    parse_macro_input!(args with item_parser);

    parse_push_stream_impl(item.expect("[item = T] should be set"), ty, input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn parse_push_stream_impl(
    item_type: Type,
    ty: StreamType,
    input: TokenStream,
) -> Result<TokenStream> {
    let push_struct: ItemStruct = syn::parse2(input)?;
    let generics = &push_struct.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let ident = &push_struct.ident;

    let poll_fn = match ty {
        StreamType::Push => quote!(poll_update),
        StreamType::PushTimeout => quote!(poll_timeout),
        StreamType::None => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "[push] or [timeout] should be set",
            ))
        }
    };

    let stream_impl: TokenStream = quote! {
        impl #impl_generics futures::Stream for #ident #ty_generics #where_clause {
            type Item = #item_type;

            fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
                match self.#poll_fn(cx) {
                    std::task::Poll::Ready(option) => std::task::Poll::Ready(Some(option)),
                    std::task::Poll::Pending => std::task::Poll::Pending,
                }
            }
        }
    };

    Ok(quote! {
        #push_struct
        #stream_impl
    })
}
