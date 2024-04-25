use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result, Token,
};

use crate::conf;

pub struct Config;
pub struct ConfigItem;
impl Parse for ConfigItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        match ident.to_string().as_str() {
            "dump" => {
                let val: syn::LitStr = input.parse()?;
                conf::set_dump_code(Some(val.value()))
            }
            "pub_fields" => {
                let val: syn::LitBool = input.parse()?;
                conf::set_pub_nodes(val.value)
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unexpected configuration key",
                ))
            }
        }
        Ok(ConfigItem)
    }
}
impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(true) = input
            .fork()
            .call(syn::Attribute::parse_inner)
            .map(|attrs| !attrs.is_empty())
        {
            let _: Token![#] = input.parse()?;
            let _: Token![!] = input.parse()?;
            let content;
            let _ = syn::bracketed!(content in input);
            let _: Punctuated<ConfigItem, Token![,]> = Punctuated::parse_terminated(&content)?;
        }
        Ok(Self)
    }
}
