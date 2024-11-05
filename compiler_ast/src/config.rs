prelude! {
    syn::{Parse, Punctuated, Res, Error, LitStr},
}

/// Configuration items in the AST.
///
/// They set the static [Conf](conf::Conf).
pub struct ConfigItem;
impl Parse for ConfigItem {
    fn parse(input: ParseStream) -> Res<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "propag" => {
                let _: Token![=] = input.parse()?;
                let val: LitStr = input.parse()?;
                match val.value().as_str() {
                    "onchange" => conf::set_propag(conf::PropagOption::OnChange),
                    "onevent" => conf::set_propag(conf::PropagOption::EventIsles),
                    _ => {
                        bail!(Error::new_spanned(
                            val,
                            "unexpected propagation configuration",
                        ));
                    }
                }
                return Ok(ConfigItem);
            }
            "dump" => {
                if let Some(prev) = conf::dump_code() {
                    let msg = format!("code-dump target already set to `{prev}`");
                    bail!(Error::new_spanned(ident, msg));
                }
                let _: Token![=] = input.parse()?;
                let val: LitStr = input.parse()?;
                conf::set_dump_code(Some(val.value()));
                return Ok(ConfigItem);
            }
            "para" => {
                conf::set_para(true);
                return Ok(ConfigItem);
            }
            "pub" => {
                conf::set_pub_components(true);
                return Ok(ConfigItem);
            }
            "greusot" => conf::set_greusot(true),
            "test" => conf::set_test(true),
            "demo" => conf::set_demo(true),
            _ => bail!(Error::new_spanned(ident, "unexpected configuration key")),
        }
        if conf::greusot() && (conf::test() || conf::demo()) {
            bail!(Error::new_spanned(
                ident,
                "greusot can not be used with test/demo modes",
            ));
        }
        if conf::test() && conf::demo() {
            bail!(Error::new_spanned(
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
    fn parse(input: ParseStream) -> Res<Self> {
        if let Ok(true) = input
            .fork()
            .call(syn::Attribute::parse_inner)
            .map(|attrs| !attrs.is_empty())
        {
            // reset config before parsing items
            conf::reset();

            let _: Token![#] = input.parse()?;
            let _: Token![!] = input.parse()?;
            let content;
            let _ = bracketed!(content in input);
            let _: Punctuated<ConfigItem, Token![,]> = Punctuated::parse_terminated(&content)?;
        }
        Ok(Self)
    }
}
