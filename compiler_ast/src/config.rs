prelude! {
    syn::{
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        Result, Token,
    },
}

/// Configuration items in the AST.
///
/// They set the static [Conf](conf::Conf).
pub struct ConfigItem;
impl Parse for ConfigItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_str() {
            "dump" => {
                if conf::dump_code().is_some() {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "dump code only once",
                    ));
                }
                let _: Token![=] = input.parse()?;
                let val: syn::LitStr = input.parse()?;
                conf::set_dump_code(Some(val.value()));
                return Ok(ConfigItem);
            }
            "pub" => {
                conf::set_pub_components(true);
                return Ok(ConfigItem);
            }
            "greusot" => conf::set_greusot(true),
            "test" => conf::set_test(true),
            "demo" => conf::set_demo(true),
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unexpected configuration key",
                ))
            }
        }
        if conf::greusot() && (conf::test() || conf::demo()) {
            return Err(syn::Error::new_spanned(
                ident,
                "greusot can not be used with test/demo modes",
            ));
        }
        if conf::test() && conf::demo() {
            return Err(syn::Error::new_spanned(
                ident,
                "test and demo modes are incompatible",
            ));
        }
        Ok(ConfigItem)
    }
}

/// Configuration structure in the AST.
///
/// It sets the static [Conf](conf::Conf).
pub struct Config;
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
