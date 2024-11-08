//! Parsing.

prelude! {
    syn::{Parse, Punctuated, token, LitInt, LitStr, Error, Res},
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

impl Parse for Item {
    fn parse(input: ParseStream) -> Res<Self> {
        if ComponentImport::peek(input) {
            Ok(Item::ComponentImport(input.parse()?))
        } else if Component::peek(input) {
            Ok(Item::Component(input.parse()?))
        } else if Function::peek(input) {
            Ok(Item::Function(input.parse()?))
        } else if Typedef::peek(input) {
            Ok(Item::Typedef(input.parse()?))
        } else if Service::peek(input) {
            Ok(Item::Service(input.parse()?))
        } else if FlowImport::peek(input) {
            Ok(Item::Import(input.parse()?))
        } else if FlowExport::peek(input) {
            Ok(Item::Export(input.parse()?))
        } else {
            Err(input.error(
                "expected either a flow import/export, a type, a component definition/import, \
                or a function/service definition",
            ))
        }
    }
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Res<Self> {
        let _: Config = input.parse()?;
        let items: Vec<Item> = {
            let mut items = Vec::with_capacity(100);
            while !input.is_empty() {
                items.push(input.parse()?);
            }
            items.shrink_to_fit();
            items
        };
        Ok(Self { items })
    }
}

pub trait ParsePrec
where
    Self: Sized,
{
    fn parse_term(input: ParseStream) -> syn::Res<Self>;
    fn parse_prec1(input: ParseStream) -> syn::Res<Self>;
    fn parse_prec2(input: ParseStream) -> syn::Res<Self>;
    fn parse_prec3(input: ParseStream) -> syn::Res<Self>;
    fn parse_prec4(input: ParseStream) -> syn::Res<Self>;
}

mod parse_conf {
    use super::*;

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
    impl Parse for ConfigItem {
        fn parse(input: ParseStream) -> Res<Self> {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "propag" => {
                    let _: Token![=] = input.parse()?;
                    let val: LitStr = input.parse()?;
                    match val.value().as_str() {
                        "onchange" => conf::set_propagation(conf::Propagation::OnChange),
                        "onevent" => conf::set_propagation(conf::Propagation::EventIsles),
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
}

mod interface {
    use super::*;
    prelude! { just interface::* }

    impl Sample {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::sample)
        }
    }
    impl Parse for Sample {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let sample_token: keyword::sample = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr: FlowExpression = content.parse()?;
            let comma_token: Token![,] = content.parse()?;
            let period_ms: LitInt = content.parse()?;
            if content.is_empty() {
                Ok(Sample::new(
                    sample_token,
                    paren_token,
                    expr,
                    comma_token,
                    period_ms,
                ))
            } else {
                Err(content.error("expected two input expressions"))
            }
        }
    }

    impl Scan {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::scan)
        }
    }
    impl Parse for Scan {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let scan_token: keyword::scan = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr: FlowExpression = content.parse()?;
            let comma_token: Token![,] = content.parse()?;
            let period_ms: LitInt = content.parse()?;
            if content.is_empty() {
                Ok(Scan::new(
                    scan_token,
                    paren_token,
                    expr,
                    comma_token,
                    period_ms,
                ))
            } else {
                Err(content.error("expected two input expressions"))
            }
        }
    }

    impl Function {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::function)
        }
    }
    impl Parse for Function {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let function_token: keyword::function = input.parse()?;
            let ident: Ident = input.parse()?;
            let content;
            let args_paren: token::Paren = parenthesized!(content in input);
            let args: Punctuated<Colon<Ident, Typ>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let arrow_token: Token![->] = input.parse()?;
            let output_type: Typ = input.parse()?;
            let contract: Contract = input.parse()?;
            let content;
            let brace: token::Brace = braced!(content in input);
            let statements: Vec<Stmt> = {
                let mut statements = Vec::new();
                while !content.is_empty() {
                    statements.push(content.parse()?);
                }
                statements
            };
            Ok(Function {
                function_token,
                ident,
                args_paren,
                args,
                arrow_token,
                output_type,
                contract,
                brace,
                statements,
            })
        }
    }
    impl Timeout {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::timeout)
        }
    }
    impl Parse for Timeout {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let timeout_token: keyword::timeout = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr: FlowExpression = content.parse()?;
            let comma_token: Token![,] = content.parse()?;
            let deadline: LitInt = content.parse()?;
            if content.is_empty() {
                Ok(Timeout::new(
                    timeout_token,
                    paren_token,
                    expr,
                    comma_token,
                    deadline,
                ))
            } else {
                Err(content.error("expected two input expressions"))
            }
        }
    }

    impl Throttle {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::throttle)
        }
    }
    impl Parse for Throttle {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let throttle_token: keyword::throttle = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr: FlowExpression = content.parse()?;
            let comma_token: Token![,] = content.parse()?;
            let delta: Constant = content.parse()?;
            if content.is_empty() {
                Ok(Throttle::new(
                    throttle_token,
                    paren_token,
                    expr,
                    comma_token,
                    delta,
                ))
            } else {
                Err(content.error("expected two input expressions"))
            }
        }
    }
    impl OnChange {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::on_change)
        }
    }
    impl Parse for OnChange {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let on_change_token: keyword::on_change = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr: FlowExpression = content.parse()?;
            if content.is_empty() {
                Ok(OnChange::new(on_change_token, paren_token, expr))
            } else {
                Err(content.error("expected one input expression"))
            }
        }
    }

    impl Merge {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::merge)
        }
    }
    impl Parse for Merge {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let merge_token: keyword::merge = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr_1: FlowExpression = content.parse()?;
            let comma_token = content.parse()?;
            let expr_2: FlowExpression = content.parse()?;
            if content.is_empty() {
                Ok(Merge::new(
                    merge_token,
                    paren_token,
                    expr_1,
                    comma_token,
                    expr_2,
                ))
            } else {
                Err(content.error("expected two input expressions"))
            }
        }
    }

    impl Parse for ComponentCall {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let ident_component: Ident = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let inputs: Punctuated<FlowExpression, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            // let ident_signal: Option<(Token![.], Ident)> = {
            //     if input.peek(Token![.]) {
            //         Some((input.parse()?, input.parse()?))
            //     } else {
            //         None
            //     }
            // };
            Ok(ComponentCall {
                ident_component,
                paren_token,
                inputs,
            })
        }
    }

    impl Parse for FlowExpression {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if Sample::peek(input) {
                Ok(Self::sample(input.parse()?))
            } else if Scan::peek(input) {
                Ok(Self::scan(input.parse()?))
            } else if Timeout::peek(input) {
                Ok(Self::timeout(input.parse()?))
            } else if Throttle::peek(input) {
                Ok(Self::throttle(input.parse()?))
            } else if OnChange::peek(input) {
                Ok(Self::on_change(input.parse()?))
            } else if Merge::peek(input) {
                Ok(Self::merge(input.parse()?))
            } else if input.fork().call(ComponentCall::parse).is_ok() {
                Ok(Self::comp_call(input.parse()?))
            } else {
                let ident: Ident = input.parse()?;
                Ok(Self::ident(ident.to_string()))
            }
        }
    }

    impl FlowKind {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::signal) || input.peek(keyword::event)
        }
    }
    impl Parse for FlowKind {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(keyword::signal) {
                Ok(FlowKind::Signal(input.parse()?))
            } else if input.peek(keyword::event) {
                Ok(FlowKind::Event(input.parse()?))
            } else {
                Err(input.error("expected 'signal' or 'event'"))
            }
        }
    }

    impl Parse for FlowPattern {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(token::Paren) {
                let content;
                let paren_token: token::Paren = parenthesized!(content in input);
                let patterns: Punctuated<FlowPattern, Token![,]> =
                    Punctuated::parse_terminated(&content)?;
                Ok(FlowPattern::Tuple {
                    paren_token,
                    patterns,
                })
            } else if FlowKind::peek(input) {
                let kind: FlowKind = input.parse()?;
                let ident: Ident = input.parse()?;
                let colon_token: Token![:] = input.parse()?;
                let ty: Typ = input.parse()?;
                Ok(FlowPattern::SingleTyped {
                    kind,
                    ident,
                    colon_token,
                    ty,
                })
            } else {
                let ident: Ident = input.parse()?;
                Ok(FlowPattern::Single { ident })
            }
        }
    }

    impl FlowDeclaration {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(Token![let])
        }
    }
    impl Parse for FlowDeclaration {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let let_token: Token![let] = input.parse()?;
            let typed_pattern: FlowPattern = input.parse()?;
            let eq_token: Token![=] = input.parse()?;
            let expr: FlowExpression = input.parse()?;
            let semi_token: Token![;] = input.parse()?;
            Ok(FlowDeclaration {
                let_token,
                typed_pattern,
                eq_token,
                expr,
                semi_token,
            })
        }
    }

    impl FlowInstantiation {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(FlowPattern::parse).is_err() {
                return false;
            }
            forked.peek(Token![=])
        }
    }
    impl Parse for FlowInstantiation {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let pattern: FlowPattern = input.parse()?;
            let eq_token: Token![=] = input.parse()?;
            let expr: FlowExpression = input.parse()?;
            let semi_token: Token![;] = input.parse()?;
            Ok(FlowInstantiation {
                pattern,
                eq_token,
                expr,
                semi_token,
            })
        }
    }

    impl FlowImport {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            forked
                .parse::<keyword::import>()
                .and_then(|_| forked.parse::<FlowKind>())
                .is_ok()
        }
    }
    impl Parse for FlowImport {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let import_token: keyword::import = input.parse()?;
            let kind: FlowKind = input.parse()?;
            let typed_path: Colon<syn::Path, Typ> = input.parse()?;
            let semi_token: Token![;] = input.parse()?;
            Ok(FlowImport {
                import_token,
                kind,
                typed_path,
                semi_token,
            })
        }
    }

    impl FlowExport {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::export)
        }
    }
    impl Parse for FlowExport {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let export_token: keyword::export = input.parse()?;
            let kind: FlowKind = input.parse()?;
            let typed_path: Colon<syn::Path, Typ> = input.parse()?;
            let semi_token: Token![;] = input.parse()?;
            Ok(FlowExport {
                export_token,
                kind,
                typed_path,
                semi_token,
            })
        }
    }

    impl FlowStatement {
        pub fn peek(input: ParseStream) -> bool {
            FlowDeclaration::peek(input) || FlowInstantiation::peek(input)
        }
    }
    impl Parse for FlowStatement {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if FlowDeclaration::peek(input) {
                Ok(FlowStatement::Declaration(input.parse()?))
            } else if FlowInstantiation::peek(input) {
                Ok(FlowStatement::Instantiation(input.parse()?))
            } else {
                Err(input.error("expected flow declaration or instantiation"))
            }
        }
    }

    impl TimeRange {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(Token![@])
        }
    }
    impl Parse for TimeRange {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let at_token: token::At = input.parse()?;
            let content;
            let bracket_token: token::Bracket = syn::bracketed!(content in input);
            let min: LitInt = content.parse()?;
            let comma_token: token::Comma = content.parse()?;
            let max: LitInt = content.parse()?;
            if content.is_empty() {
                Ok(TimeRange {
                    at_token,
                    bracket_token,
                    min,
                    comma_token,
                    max,
                })
            } else {
                Err(content.error("expected something like `@ [min, max]`"))
            }
        }
    }

    impl Service {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::service)
        }
    }
    impl Parse for Service {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let service_token: keyword::service = input.parse()?;
            let ident: Ident = input.parse()?;
            let time_range = if TimeRange::peek(input) {
                Some(input.parse()?)
            } else {
                None
            };
            let content;
            let brace: token::Brace = syn::braced!(content in input);
            let flow_statements: Vec<FlowStatement> = {
                let mut flow_statements = vec![];
                while !content.is_empty() {
                    flow_statements.push(content.parse()?)
                }
                flow_statements
            };
            Ok(Service {
                service_token,
                ident,
                time_range,
                brace,
                flow_statements,
            })
        }
    }
}

mod parse_component {
    use super::*;

    impl Component {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::component)
        }
    }

    impl Parse for Component {
        fn parse(input: ParseStream) -> Res<Self> {
            let component_token: keyword::component = input.parse()?;
            let ident: Ident = input.parse()?;
            let content;
            let args_paren: token::Paren = parenthesized!(content in input);
            let args: Punctuated<Colon<Ident, Typ>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let arrow_token: Token![->] = input.parse()?;
            let content;
            let outs_paren: token::Paren = parenthesized!(content in input);
            let outs: Punctuated<Colon<Ident, Typ>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let period: Option<(Token![@], LitInt, keyword::ms)> = {
                if input.peek(Token![@]) {
                    Some((input.parse()?, input.parse()?, input.parse()?))
                } else {
                    None
                }
            };
            let contract: Contract = input.parse()?;
            let content;
            let brace: token::Brace = braced!(content in input);
            let equations: Vec<ReactEq> = {
                let mut equations = vec![];
                while !content.is_empty() {
                    equations.push(content.parse()?)
                }
                equations
            };
            Ok(Component {
                component_token,
                ident,
                args_paren,
                args,
                arrow_token,
                outs_paren,
                outs,
                period,
                contract,
                brace,
                equations,
            })
        }
    }

    impl ComponentImport {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            forked
                .parse::<keyword::import>()
                .and_then(|_| forked.parse::<keyword::component>())
                .is_ok()
        }
    }

    impl Parse for ComponentImport {
        fn parse(input: ParseStream) -> Res<Self> {
            let import_token: keyword::import = input.parse()?;
            let component_token: keyword::component = input.parse()?;
            let path: syn::Path = input.parse()?;
            let colon_token: Token![:] = input.parse()?;
            let content;
            let args_paren: token::Paren = parenthesized!(content in input);
            let args: Punctuated<Colon<Ident, Typ>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let arrow_token: Token![->] = input.parse()?;
            let content;
            let outs_paren: token::Paren = parenthesized!(content in input);
            let outs: Punctuated<Colon<Ident, Typ>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let period: Option<(Token![@], LitInt, keyword::ms)> = {
                if input.peek(Token![@]) {
                    Some((input.parse()?, input.parse()?, input.parse()?))
                } else {
                    None
                }
            };
            let semi_token: Token![;] = input.parse()?;
            Ok(ComponentImport {
                import_token,
                component_token,
                path,
                colon_token,
                args_paren,
                args,
                arrow_token,
                outs_paren,
                outs,
                period,
                semi_token,
            })
        }
    }
}
impl Typedef {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![struct]) || input.peek(Token![enum]) || input.peek(keyword::array)
    }
}
impl Parse for Typedef {
    fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(Token![struct]) {
            let struct_token: Token![struct] = input.parse()?;
            let ident: Ident = input.parse()?;
            let content;
            let brace: token::Brace = braced!(content in input);
            let fields: Punctuated<Colon<Ident, Typ>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Typedef::Structure {
                struct_token,
                ident,
                brace,
                fields,
            })
        } else if input.peek(Token![enum]) {
            let enum_token: Token![enum] = input.parse()?;
            let ident: Ident = input.parse()?;
            let content;
            let brace: token::Brace = braced!(content in input);
            let elements: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Typedef::Enumeration {
                enum_token,
                ident,
                brace,
                elements,
            })
        } else if input.peek(keyword::array) {
            let array_token: keyword::array = input.parse()?;
            let ident: Ident = input.parse()?;
            let content;
            let bracket_token: token::Bracket = bracketed!(content in input);
            let array_type: Typ = content.parse()?;
            let semi_token: Token![;] = content.parse()?;
            let size: syn::LitInt = content.parse()?;
            if content.is_empty() {
                Ok(Typedef::Array {
                    array_token,
                    ident,
                    bracket_token,
                    array_type,
                    semi_token,
                    size,
                })
            } else {
                Err(input.error("expected array alias definition"))
            }
        } else {
            Err(input.error("expected type definition"))
        }
    }
}

mod parse_stream {
    use super::*;
    use stream::*;

    impl Last {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::last)
        }
    }
    impl Parse for Last {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let _: keyword::last = input.parse()?;
            let ident = input.parse()?;
            let constant = if input.peek(keyword::init) {
                let _: keyword::init = input.parse()?;
                let constant = input.parse()?;
                Some(constant)
            } else {
                None
            };
            Ok(Last::new(ident, constant))
        }
    }

    impl When {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::when)
        }
    }
    impl Parse for When {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let _: keyword::when = input.parse()?;
            let pattern: equation::EventPattern = input.parse()?;
            let guard = {
                if input.fork().peek(Token![if]) {
                    let _: Token![if] = input.parse()?;
                    let guard = input.parse()?;
                    Some(guard)
                } else {
                    None
                }
            };
            let then_token: keyword::then = input.parse()?;
            let expression: stream::Expr = input.parse()?;
            Ok(When::new(pattern, guard, then_token, expression))
        }
    }

    impl Emit {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::emit)
        }
    }
    impl Parse for Emit {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let emit_token: keyword::emit = input.parse()?;
            let expr: stream::Expr = input.parse()?;
            Ok(Emit::new(emit_token, expr))
        }
    }

    impl ParsePrec for stream::Expr {
        fn parse_term(input: ParseStream) -> syn::Res<Self> {
            let mut expression = if input.fork().call(Constant::parse).is_ok() {
                Self::Constant(input.parse()?)
            } else if Last::peek(input) {
                Self::Last(input.parse()?)
            } else if expr::UnOp::<Self>::peek(input) {
                Self::UnOp(input.parse()?)
            } else if expr::Zip::<Self>::peek(input) {
                Self::Zip(input.parse()?)
            } else if expr::Match::<Self>::peek(input) {
                Self::Match(input.parse()?)
            } else if expr::Tuple::<Self>::peek(input) {
                let mut tuple: expr::Tuple<Self> = input.parse()?;
                if tuple.elements.len() == 1 {
                    tuple.elements.pop().unwrap()
                } else {
                    Self::Tuple(tuple)
                }
            } else if expr::Array::<Self>::peek(input) {
                Self::Array(input.parse()?)
            } else if expr::Structure::<Self>::peek(input) {
                Self::Structure(input.parse()?)
            } else if expr::Enumeration::<Self>::peek(input) {
                Self::Enumeration(input.parse()?)
            } else if input.fork().call(Ident::parse).is_ok() {
                let ident: Ident = input.parse()?;
                Self::Identifier(ident.to_string())
            } else {
                return Err(input.error("expected expression"));
            };
            loop {
                if expr::Sort::<Self>::peek(input) {
                    expression = Self::Sort(expr::Sort::<Self>::parse(expression, input)?);
                } else if expr::Map::<Self>::peek(input) {
                    expression = Self::Map(expr::Map::<Self>::parse(expression, input)?)
                } else if expr::Fold::<Self>::peek(input) {
                    expression = Self::Fold(expr::Fold::<Self>::parse(expression, input)?)
                } else if expr::TupleElementAccess::<Self>::peek(input) {
                    expression = Self::TupleElementAccess(expr::TupleElementAccess::<Self>::parse(
                        expression, input,
                    )?)
                } else if expr::FieldAccess::<Self>::peek(input) {
                    expression =
                        Self::FieldAccess(expr::FieldAccess::<Self>::parse(expression, input)?)
                } else if expr::Application::<Self>::peek(input) {
                    expression =
                        Self::Application(expr::Application::<Self>::parse(expression, input)?)
                } else {
                    break;
                }
            }
            Ok(expression)
        }

        fn parse_prec1(input: ParseStream) -> syn::Res<Self> {
            let mut expression = Self::parse_term(input)?;

            loop {
                if BOp::peek_prec1(input) {
                    expression = Self::Binop(expr::Binop::<Self>::parse_term(expression, input)?);
                } else {
                    break;
                }
            }
            Ok(expression)
        }
        fn parse_prec2(input: ParseStream) -> syn::Res<Self> {
            let mut expression = Self::parse_prec1(input)?;

            loop {
                if BOp::peek_prec2(input) {
                    expression = Self::Binop(expr::Binop::<Self>::parse_prec1(expression, input)?);
                } else {
                    break;
                }
            }
            Ok(expression)
        }
        fn parse_prec3(input: ParseStream) -> syn::Res<Self> {
            let mut expression = Self::parse_prec2(input)?;

            loop {
                if BOp::peek_prec3(input) {
                    expression = Self::Binop(expr::Binop::<Self>::parse_prec2(expression, input)?);
                } else {
                    break;
                }
            }
            Ok(expression)
        }
        fn parse_prec4(input: ParseStream) -> syn::Res<Self> {
            let mut expression = Self::parse_prec3(input)?;

            loop {
                if BOp::peek_prec4(input) {
                    expression = Self::Binop(expr::Binop::<Self>::parse_prec3(expression, input)?);
                } else {
                    break;
                }
            }
            Ok(expression)
        }
    }
    impl Parse for stream::Expr {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expression = if expr::TypedAbstraction::<Self>::peek(input) {
                Self::TypedAbstraction(input.parse()?)
            } else if expr::IfThenElse::<Self>::peek(input) {
                Self::IfThenElse(input.parse()?)
            } else if stream::Emit::peek(input) {
                Self::Emit(input.parse()?)
            } else if stream::When::peek(input) {
                return Err(input.error("'when' is a root expression"));
            } else {
                Self::parse_prec4(input)?
            };
            Ok(expression)
        }
    }

    impl Parse for ReactExpr {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expression = if When::peek(input) {
                Self::when_match(input.parse()?)
            } else {
                Self::expr(input.parse()?)
            };
            Ok(expression)
        }
    }
}

mod parse_stmt {
    use super::*;
    use stmt::*;

    impl Typed {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(Token![:])
        }

        pub fn parse(ident: Ident, input: ParseStream) -> syn::Res<Self> {
            let colon_token: Token![:] = input.parse()?;
            let typ = input.parse()?;
            Ok(Typed {
                ident,
                colon_token,
                typ,
            })
        }
    }

    impl Tuple {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(token::Paren)
        }
    }
    impl Parse for Tuple {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let content;
            let _ = parenthesized!(content in input);
            let elements: Punctuated<Pattern, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Tuple {
                elements: elements.into_iter().collect(),
            })
        }
    }

    impl Parse for Pattern {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let pattern = if Tuple::peek(input) {
                Pattern::Tuple(input.parse()?)
            } else {
                let ident: Ident = input.parse()?;
                if Typed::peek(input) {
                    Pattern::Typed(Typed::parse(ident, input)?)
                } else {
                    Pattern::ident(ident)
                }
            };

            Ok(pattern)
        }
    }

    impl<E> Parse for LetDecl<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let let_token: Token![let] = input.parse()?;
            let typed_pattern: Pattern = input.parse()?;
            let eq_token: Token![=] = input.parse()?;
            let expr: E = input.parse()?;
            let semi_token: Token![;] = input.parse()?;

            Ok(LetDecl {
                let_token,
                typed_pattern,
                eq_token,
                expr,
                semi_token,
            })
        }
    }
    impl Parse for Return {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let return_token: Token![return] = input.parse()?;
            let expression: Expr = input.parse()?;
            let semi_token: Token![;] = input.parse()?;

            Ok(Return {
                return_token,
                expression,
                semi_token,
            })
        }
    }

    impl Parse for Stmt {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(Token![let]) {
                Ok(Stmt::Declaration(input.parse()?))
            } else {
                Ok(Stmt::Return(input.parse()?))
            }
        }
    }
}

mod parse_expr {
    use super::*;
    use expr::*;

    impl<E> UnOp<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            UOp::peek(input)
        }
    }
    impl<E> Parse for UnOp<E>
    where
        E: ParsePrec,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let expr = Box::new(E::parse_term(input)?);
            Ok(UnOp { op, expr })
        }
    }

    impl<E> Binop<E>
    where
        E: ParsePrec,
    {
        pub fn peek(input: ParseStream) -> bool {
            BOp::peek(input)
        }
        pub fn parse_term(lhs: E, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let rhs = E::parse_term(input)?;
            Ok(Binop::new(op, lhs, rhs))
        }
        pub fn parse_prec1(lhs: E, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let rhs = E::parse_prec1(input)?;
            Ok(Binop::new(op, lhs, rhs))
        }
        pub fn parse_prec2(lhs: E, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let rhs = E::parse_prec2(input)?;
            Ok(Binop::new(op, lhs, rhs))
        }
        pub fn parse_prec3(lhs: E, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let rhs = E::parse_prec3(input)?;
            Ok(Binop::new(op, lhs, rhs))
        }
    }

    impl<E> IfThenElse<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(Token![if])
        }
    }
    impl<E> Parse for IfThenElse<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let _: Token![if] = input.parse()?;
            let cnd = Box::new(input.parse()?);
            let _: keyword::then = input.parse()?;
            let thn = Box::new(input.parse()?);
            let _: Token![else] = input.parse()?;
            let els = Box::new(input.parse()?);
            Ok(IfThenElse { cnd, thn, els })
        }
    }

    impl<E> Application<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(token::Paren)
        }

        pub fn parse(function: E, input: ParseStream) -> syn::Res<Self> {
            let content;
            let _ = syn::parenthesized!(content in input);
            let inputs: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Application::new(function, inputs.into_iter().collect()))
        }
    }
    impl<E> Parse for Application<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let function: E = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let inputs: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Application::new(function, inputs.into_iter().collect()))
        }
    }

    impl<E> TypedAbstraction<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(Token![|])
        }
    }
    impl<E> Parse for TypedAbstraction<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let _: Token![|] = input.parse()?;
            let mut inputs: Punctuated<Colon<Ident, Typ>, Token![,]> = Punctuated::new();
            loop {
                if input.peek(Token![|]) {
                    break;
                }
                let value = input.parse()?;
                inputs.push_value(value);
                if input.peek(Token![|]) {
                    break;
                }
                let comma: Token![,] = input.parse()?;
                inputs.push_punct(comma);
            }
            let _: Token![|] = input.parse()?;
            let expr: E = input.parse()?;
            Ok(TypedAbstraction::new(
                inputs
                    .into_iter()
                    .map(|Colon { left, right, .. }| (left.to_string(), right))
                    .collect(),
                expr,
            ))
        }
    }

    impl<E> Structure<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            forked.call(Structure::<E>::parse).is_ok()
        }
    }
    impl<E> Parse for Structure<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let ident: Ident = input.parse()?;
            let content;
            let _ = braced!(content in input);
            let fields: Punctuated<Colon<Ident, E>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Structure::new(
                ident.to_string(),
                fields
                    .into_iter()
                    .map(|Colon { left, right, .. }| (left.to_string(), right))
                    .collect(),
            ))
        }
    }

    impl<E> Tuple<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(token::Paren)
        }
    }

    impl<E> Parse for Tuple<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let content;
            let _ = parenthesized!(content in input);
            let elements: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Tuple::new(elements.into_iter().collect()))
        }
    }

    impl<E> Enumeration<E> {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(Ident::parse).is_err() {
                return false;
            }
            forked.peek(Token![::])
        }
    }

    impl<E> Parse for Enumeration<E> {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let ident_enum: Ident = input.parse()?;
            let _: Token![::] = input.parse()?;
            let ident_elem: Ident = input.parse()?;
            Ok(Enumeration::new(
                ident_enum.to_string(),
                ident_elem.to_string(),
            ))
        }
    }

    impl<E> Array<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket)
        }
    }

    impl<E> Parse for Array<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let content;
            let _ = bracketed!(content in input);
            let elements: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Array::new(elements.into_iter().collect()))
        }
    }

    impl PatStructure {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(Ident::parse).is_err() {
                return false;
            }
            forked.peek(token::Brace)
        }
    }

    impl Parse for PatStructure {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let ident: Ident = input.parse()?;
            let content;
            let _ = braced!(content in input);
            let mut fields: Punctuated<(Ident, Option<(Token![:], Pattern)>), Token![,]> =
                Punctuated::new();
            let mut rest = None;
            while !content.is_empty() {
                if content.peek(Token![..]) {
                    rest = Some(content.parse()?);
                    break;
                }

                let member: Ident = content.parse()?;
                let optional_pattern = if content.peek(Token![:]) {
                    let colon_token = content.parse()?;
                    let pattern = content.parse()?;
                    Some((colon_token, pattern))
                } else {
                    None
                };
                fields.push_value((member, optional_pattern));

                if content.is_empty() {
                    break;
                }
                fields.push_punct(content.parse()?);
            }

            Ok(PatStructure {
                name: ident.to_string(),
                fields: fields
                    .into_iter()
                    .map(|(ident, optional_pattern)| {
                        (
                            ident.to_string(),
                            optional_pattern.map(|(_, pattern)| pattern),
                        )
                    })
                    .collect(),
                rest,
            })
        }
    }

    impl PatEnumeration {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(Ident::parse).is_err() {
                return false;
            }
            forked.peek(Token![::])
        }
    }

    impl Parse for PatEnumeration {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let ident_enum: Ident = input.parse()?;
            let _: Token![::] = input.parse()?;
            let ident_elem: Ident = input.parse()?;
            Ok(PatEnumeration {
                enum_name: ident_enum.to_string(),
                elem_name: ident_elem.to_string(),
            })
        }
    }

    impl PatTuple {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(token::Paren)
        }
    }

    impl Parse for PatTuple {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let content;
            let _ = parenthesized!(content in input);
            let elements: Punctuated<Pattern, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(PatTuple {
                elements: elements.into_iter().collect(),
            })
        }
    }

    impl Parse for Pattern {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let pattern = if input.fork().call(Constant::parse).is_ok() {
                Pattern::Constant(input.parse()?)
            } else if PatStructure::peek(input) {
                Pattern::Structure(input.parse()?)
            } else if PatTuple::peek(input) {
                Pattern::Tuple(input.parse()?)
            } else if PatEnumeration::peek(input) {
                Pattern::Enumeration(input.parse()?)
            } else if input.fork().peek(Token![_]) {
                let _: Token![_] = input.parse()?;
                Pattern::Default
            } else {
                let ident: Ident = input.parse()?;
                Pattern::Identifier(ident.to_string())
            };

            Ok(pattern)
        }
    }

    impl<E> Parse for Arm<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let pattern = input.parse()?;
            let guard = {
                if input.fork().peek(Token![if]) {
                    let _: Token![if] = input.parse()?;
                    let guard = input.parse()?;
                    Some(guard)
                } else {
                    None
                }
            };
            let _: Token![=>] = input.parse()?;
            let expr = input.parse()?;
            Ok(Arm {
                pattern,
                guard,
                expr,
            })
        }
    }

    impl<E> Match<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(Token![match])
        }
    }
    impl<E> Parse for Match<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let _: Token![match] = input.parse()?;
            let expr: E = input.parse()?;
            let content;
            let _ = braced!(content in input);
            let arms: Punctuated<Arm<E>, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Match::new(expr, arms.into_iter().collect()))
        }
    }

    impl<E> FieldAccess<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(token::Dot::parse).is_err() {
                return false;
            }
            forked.call(Ident::parse).is_ok()
        }

        pub fn parse(expr: E, input: ParseStream) -> syn::Res<Self> {
            let _: Token![.] = input.parse()?;
            let field: Ident = input.parse()?;
            Ok(FieldAccess::new(expr, field.to_string()))
        }
    }
    impl<E> Parse for FieldAccess<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let field: Ident = input.parse()?;
            Ok(FieldAccess::new(expr, field.to_string()))
        }
    }

    impl<E> TupleElementAccess<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(token::Dot::parse).is_err() {
                return false;
            }
            forked.call(syn::LitInt::parse).is_ok()
        }

        pub fn parse(expr: E, input: ParseStream) -> syn::Res<Self> {
            let _: Token![.] = input.parse()?;
            let element_number: syn::LitInt = input.parse()?;
            Ok(TupleElementAccess::new(
                expr,
                element_number.base10_parse().unwrap(),
            ))
        }
    }
    impl<E> Parse for TupleElementAccess<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let element_number: syn::LitInt = input.parse()?;
            Ok(TupleElementAccess::new(
                expr,
                element_number.base10_parse().unwrap(),
            ))
        }
    }

    impl<E> Map<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(token::Dot::parse).is_err() {
                return false;
            }
            forked.peek(keyword::map)
        }

        pub fn parse(expr: E, input: ParseStream) -> syn::Res<Self> {
            let _: Token![.] = input.parse()?;
            let _: keyword::map = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let fun: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr, fun))
            } else {
                Err(input.error("expected only one expression"))
            }
        }
    }
    impl<E> Parse for Map<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let _: keyword::map = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let fun: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr, fun))
            } else {
                Err(input.error("expected only one expression"))
            }
        }
    }

    impl<E> Fold<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(token::Dot::parse).is_err() {
                return false;
            }
            forked.peek(keyword::fold)
        }

        pub fn parse(expr: E, input: ParseStream) -> syn::Res<Self> {
            let _: Token![.] = input.parse()?;
            let _: keyword::fold = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let init: E = content.parse()?;
            let _: Token![,] = content.parse()?;
            let function: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr, init, function))
            } else {
                Err(input.error("expected only two expressions"))
            }
        }
    }
    impl<E> Parse for Fold<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let _: keyword::fold = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let init: E = content.parse()?;
            let _: Token![,] = content.parse()?;
            let function: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr, init, function))
            } else {
                Err(input.error("expected only two expressions"))
            }
        }
    }

    impl<E> Sort<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(token::Dot::parse).is_err() {
                return false;
            }
            forked.peek(keyword::sort)
        }

        pub fn parse(expr: E, input: ParseStream) -> syn::Res<Self> {
            let _: Token![.] = input.parse()?;
            let _: keyword::sort = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let fun: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr, fun))
            } else {
                Err(input.error("expected only one expression"))
            }
        }
    }
    impl<E> Parse for Sort<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let _: keyword::sort = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let fun: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr, fun))
            } else {
                Err(input.error("expected only one expression"))
            }
        }
    }

    impl<E> Zip<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::zip)
        }
    }
    impl<E> Parse for Zip<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let _: keyword::zip = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let arrays: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Zip::new(arrays.into_iter().collect()))
        }
    }

    impl ParsePrec for Expr {
        fn parse_term(input: ParseStream) -> syn::Res<Self> {
            let mut expr = if input.fork().call(Constant::parse).is_ok() {
                Self::cst(input.parse()?)
            } else if UnOp::<Self>::peek(input) {
                Self::unop(input.parse()?)
            } else if Zip::<Self>::peek(input) {
                Self::zip(input.parse()?)
            } else if Match::<Self>::peek(input) {
                Self::pat_match(input.parse()?)
            } else if Tuple::<Self>::peek(input) {
                let mut tuple: Tuple<Self> = input.parse()?;
                if tuple.elements.len() == 1 {
                    tuple.elements.pop().unwrap()
                } else {
                    Self::tuple(tuple)
                }
            } else if Array::<Self>::peek(input) {
                Self::array(input.parse()?)
            } else if Structure::<Self>::peek(input) {
                Self::structure(input.parse()?)
            } else if Enumeration::<Self>::peek(input) {
                Self::enumeration(input.parse()?)
            } else if input.fork().call(Ident::parse).is_ok() {
                let ident: Ident = input.parse()?;
                Self::ident(ident.to_string())
            } else {
                return Err(input.error("expected expression"));
            };

            loop {
                if Sort::<Self>::peek(input) {
                    expr = Self::sort(Sort::parse(expr, input)?);
                } else if Map::<Self>::peek(input) {
                    expr = Self::map(Map::parse(expr, input)?)
                } else if Fold::<Self>::peek(input) {
                    expr = Self::fold(Fold::parse(expr, input)?)
                } else if TupleElementAccess::<Self>::peek(input) {
                    expr = Self::tuple_access(TupleElementAccess::parse(expr, input)?)
                } else if FieldAccess::<Self>::peek(input) {
                    expr = Self::field_access(FieldAccess::parse(expr, input)?)
                } else if Application::<Self>::peek(input) {
                    expr = Self::app(Application::parse(expr, input)?)
                } else {
                    break;
                }
            }
            Ok(expr)
        }
        fn parse_prec1(input: ParseStream) -> syn::Res<Self> {
            let mut expr = Expr::parse_term(input)?;

            loop {
                if BOp::peek_prec1(input) {
                    expr = Expr::binop(Binop::parse_term(expr, input)?);
                } else {
                    break;
                }
            }
            Ok(expr)
        }
        fn parse_prec2(input: ParseStream) -> syn::Res<Self> {
            let mut expr = Expr::parse_prec1(input)?;

            loop {
                if BOp::peek_prec2(input) {
                    expr = Expr::Binop(Binop::parse_prec1(expr, input)?);
                } else {
                    break;
                }
            }
            Ok(expr)
        }
        fn parse_prec3(input: ParseStream) -> syn::Res<Self> {
            let mut expr = Expr::parse_prec2(input)?;

            loop {
                if BOp::peek_prec3(input) {
                    expr = Expr::binop(Binop::parse_prec2(expr, input)?);
                } else {
                    break;
                }
            }
            Ok(expr)
        }
        fn parse_prec4(input: ParseStream) -> syn::Res<Self> {
            let mut expr = Expr::parse_prec3(input)?;

            loop {
                if BOp::peek_prec4(input) {
                    expr = Expr::binop(Binop::parse_prec3(expr, input)?);
                } else {
                    break;
                }
            }
            Ok(expr)
        }
    }
    impl Parse for Expr {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr = if TypedAbstraction::<Self>::peek(input) {
                Self::typed_abstraction(input.parse()?)
            } else if IfThenElse::<Self>::peek(input) {
                Self::ite(input.parse()?)
            } else {
                Self::parse_prec4(input)?
            };

            Ok(expr)
        }
    }
}

mod parse_equation {
    use super::*;
    use equation::*;

    impl<E: Parse> Parse for Instantiation<E> {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let pattern: stmt::Pattern = input.parse()?;
            let eq: Token![=] = input.parse()?;
            let expr: E = input.parse()?;
            let semi_token: Token![;] = input.parse()?;

            Ok(Instantiation::new(pattern, eq, expr, semi_token))
        }
    }

    impl Parse for Arm {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let pattern = input.parse()?;
            let guard = {
                if input.fork().peek(Token![if]) {
                    let token = input.parse()?;
                    let guard = input.parse()?;
                    Some((token, guard))
                } else {
                    None
                }
            };
            let arrow = input.parse()?;
            let content;
            let brace = braced!(content in input);
            let equations = {
                let mut equations = Vec::new();
                while !content.is_empty() {
                    equations.push(content.parse()?);
                }
                equations
            };
            Ok(Arm::new(pattern, guard, arrow, brace, equations))
        }
    }

    impl Parse for Match {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let match_token = input.parse()?;
            let expr = input.parse()?;
            let content;
            let brace = braced!(content in input);
            let arms: Punctuated<Arm, Token![,]> = Punctuated::parse_terminated(&content)?;

            Ok(Match::new(match_token, expr, brace, arms))
        }
    }

    impl Parse for Eq {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(Token![match]) {
                Ok(Eq::pat_match(input.parse()?))
            } else if input.peek(Token![let]) {
                Ok(Eq::local_def(input.parse()?))
            } else {
                Ok(Eq::out_def(input.parse()?))
            }
        }
    }

    impl Parse for TupleEventPattern {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let content;
            let paren_token = parenthesized!(content in input);
            let patterns = Punctuated::parse_terminated(&content)?;
            Ok(TupleEventPattern::new(paren_token, patterns))
        }
    }

    impl Parse for LetEventPattern {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let let_token = input.parse()?;
            let pattern = input.parse()?;
            let eq_token = input.parse()?;
            let event = input.parse()?;
            let question_token = input.parse()?;
            Ok(LetEventPattern::new(
                let_token,
                pattern,
                eq_token,
                event,
                question_token,
            ))
        }
    }

    impl Parse for EventPattern {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(token::Paren) {
                Ok(EventPattern::Tuple(input.parse()?))
            } else if input.peek(Token![let]) {
                Ok(EventPattern::Let(input.parse()?))
            } else {
                let forked = input.fork();
                let is_event = forked
                    .parse::<Ident>()
                    .is_ok_and(|_| forked.parse::<token::Question>().is_ok());
                if is_event {
                    let event: Ident = input.parse()?;
                    let question_token: token::Question = input.parse()?;
                    let span = event.span();
                    let let_token = token::Let { span };
                    let pattern = expr::Pattern::ident(event.to_string());
                    let eq_token = token::Eq { spans: [span] };
                    let pat =
                        LetEventPattern::new(let_token, pattern, eq_token, event, question_token);
                    Ok(EventPattern::Let(pat))
                } else {
                    Ok(EventPattern::RisingEdge(input.parse()?))
                }
            }
        }
    }

    impl Parse for EventArmWhen {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let pat = input.parse()?;
            let guard = {
                if input.fork().peek(Token![if]) {
                    let token = input.parse()?;
                    let guard = input.parse()?;
                    Some((token, guard))
                } else {
                    None
                }
            };
            let arrow = input.parse()?;
            let content;
            let brace = braced!(content in input);
            let equations = {
                let mut equations = Vec::new();
                while !content.is_empty() {
                    equations.push(content.parse()?);
                }
                equations
            };
            Ok(EventArmWhen::new(pat, guard, arrow, brace, equations))
        }
    }

    impl Parse for MatchWhen {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let when_token = input.parse()?;
            let content;
            let brace = braced!(content in input);
            let mut arms: Vec<EventArmWhen> = vec![];
            while !content.is_empty() {
                arms.push(content.parse()?);
            }

            Ok(MatchWhen::new(when_token, brace, arms))
        }
    }

    impl Parse for ReactEq {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(Token![match]) {
                Ok(ReactEq::pat_match(input.parse()?))
            } else if input.peek(keyword::when) {
                Ok(ReactEq::match_when(input.parse()?))
            } else if input.peek(Token![let]) {
                Ok(ReactEq::local_def(input.parse()?))
            } else {
                Ok(ReactEq::out_def(input.parse()?))
            }
        }
    }
}

mod parse_contract {
    use super::*;
    use contract::*;

    impl ForAll {
        fn peek(input: ParseStream) -> bool {
            input.peek(keyword::forall)
        }
    }

    impl Parse for ForAll {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let forall_token: keyword::forall = input.parse()?;
            let ident: Ident = input.parse()?;
            let colon_token: Token![:] = input.parse()?;
            let ty: Typ = input.parse()?;
            let comma_token: Token![,] = input.parse()?;
            let term: Term = input.parse()?;
            Ok(ForAll::new(
                forall_token,
                ident.to_string(),
                colon_token,
                ty,
                comma_token,
                term,
            ))
        }
    }

    impl Implication {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![=>])
        }
        fn parse(input: ParseStream, left: Term) -> syn::Res<Self> {
            let arrow: Token![=>] = input.parse()?;
            let right: Term = input.parse()?;
            Ok(Implication::new(left, arrow, right))
        }
    }

    impl EventImplication {
        fn peek(input: ParseStream) -> bool {
            input.peek(keyword::when)
        }
    }
    impl Parse for EventImplication {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let when_token: keyword::when = input.parse()?;
            let pattern: Ident = input.parse()?;
            let eq_token: token::Eq = input.parse()?;
            let event: Ident = input.parse()?;
            let question_token: token::Question = input.parse()?;
            let arrow: Token![=>] = input.parse()?;
            let term: Term = Term::parse_prec4(input)?;
            Ok(EventImplication::new(
                when_token,
                pattern.to_string(),
                eq_token,
                event.to_string(),
                question_token,
                arrow,
                term,
            ))
        }
    }

    impl Enumeration {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            if forked.call(Ident::parse).is_err() {
                return false;
            }
            forked.peek(Token![::])
        }
    }
    impl Parse for Enumeration {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let ident_enum: Ident = input.parse()?;
            let _: Token![::] = input.parse()?;
            let ident_elem: Ident = input.parse()?;
            Ok(Enumeration::new(
                ident_enum.to_string(),
                ident_elem.to_string(),
            ))
        }
    }

    impl Unary {
        fn peek(input: ParseStream) -> bool {
            UOp::peek(input)
        }
    }
    impl Parse for Unary {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let op: UOp = input.parse()?;
            let term: Term = Term::parse_term(input)?;
            Ok(Unary::new(op, term))
        }
    }

    impl Binary {
        fn parse_term(left: Box<Term>, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let right = Box::new(Term::parse_term(input)?);
            Ok(Binary { op, left, right })
        }
        fn parse_prec1(left: Box<Term>, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let right = Box::new(Term::parse_prec1(input)?);
            Ok(Binary { op, left, right })
        }
        fn parse_prec2(left: Box<Term>, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let right = Box::new(Term::parse_prec2(input)?);
            Ok(Binary { op, left, right })
        }
        fn parse_prec3(left: Box<Term>, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let right = Box::new(Term::parse_prec3(input)?);
            Ok(Binary { op, left, right })
        }
    }
    impl Parse for Binary {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let left: Box<Term> = Box::new(input.parse()?);
            let op: BOp = input.parse()?;
            let right: Box<Term> = Box::new(input.parse()?);
            Ok(Binary { left, op, right })
        }
    }

    impl ParsePrec for Term {
        fn parse_term(input: ParseStream) -> syn::Res<Self> {
            let term = if input.peek(keyword::result) {
                Term::result(input.parse()?)
            } else if input.fork().call(Constant::parse).is_ok() {
                Term::constant(input.parse()?)
            } else if Enumeration::peek(input) {
                Term::enumeration(input.parse()?)
            } else if Unary::peek(input) {
                Term::unary(input.parse()?)
            } else if input.fork().call(Ident::parse).is_ok() {
                let ident: Ident = input.parse()?;
                Term::ident(ident.to_string())
            } else {
                return Err(input.error("expected expression"));
            };

            Ok(term)
        }

        fn parse_prec1(input: ParseStream) -> syn::Res<Self> {
            let mut term = Term::parse_term(input)?;

            loop {
                if BOp::peek_prec1(input) {
                    term = Term::binary(Binary::parse_term(Box::new(term), input)?);
                } else {
                    break;
                }
            }
            Ok(term)
        }

        fn parse_prec2(input: ParseStream) -> syn::Res<Self> {
            let mut term = Term::parse_prec1(input)?;

            loop {
                if BOp::peek_prec2(input) {
                    term = Term::binary(Binary::parse_prec1(Box::new(term), input)?);
                } else {
                    break;
                }
            }
            Ok(term)
        }

        fn parse_prec3(input: ParseStream) -> syn::Res<Self> {
            let mut term = Term::parse_prec2(input)?;

            loop {
                if BOp::peek_prec3(input) {
                    term = Term::binary(Binary::parse_prec2(Box::new(term), input)?);
                } else {
                    break;
                }
            }
            Ok(term)
        }

        fn parse_prec4(input: ParseStream) -> syn::Res<Self> {
            let mut term = Term::parse_prec3(input)?;

            loop {
                if BOp::peek_prec4(input) {
                    term = Term::binary(Binary::parse_prec3(Box::new(term), input)?);
                } else {
                    break;
                }
            }

            Ok(term)
        }
    }
    impl Parse for Term {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let mut term = if ForAll::peek(input) {
                Self::forall(input.parse()?)
            } else if EventImplication::peek(input) {
                Self::event(input.parse()?)
            } else {
                Self::parse_prec4(input)?
            };

            loop {
                if Implication::peek(input) {
                    term = Term::implication(Implication::parse(input, term)?);
                } else {
                    break;
                }
            }

            Ok(term)
        }
    }

    impl ClauseKind {
        pub(crate) fn peek(input: ParseStream) -> bool {
            input.peek(keyword::requires)
                || input.peek(keyword::ensures)
                || input.peek(keyword::invariant)
                || input.peek(keyword::assert)
        }
    }

    impl Parse for Clause {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let kind = {
                if input.peek(keyword::requires) {
                    Ok(ClauseKind::Requires(input.parse()?))
                } else if input.peek(keyword::ensures) {
                    Ok(ClauseKind::Ensures(input.parse()?))
                } else if input.peek(keyword::invariant) {
                    Ok(ClauseKind::Invariant(input.parse()?))
                } else if input.peek(keyword::assert) {
                    Ok(ClauseKind::Assert(input.parse()?))
                } else {
                    Err(input.error("expected 'requires', 'ensures', 'invariant', or 'assert'"))
                }
            }?;
            let content;
            let brace = braced!(content in input);
            let term = content.parse()?;

            if content.is_empty() {
                Ok(Clause::new(kind, brace, term))
            } else {
                Err(content.error("expected term"))
            }
        }
    }

    impl Parse for Contract {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let clauses = {
                let mut clauses = Vec::new();
                while ClauseKind::peek(input) {
                    clauses.push(input.parse()?);
                }
                clauses
            };
            Ok(Contract::new(clauses))
        }
    }
}

#[cfg(test)]
mod parsing_tests {
    use super::*;

    #[test]
    fn service() {
        let _: Service = parse_quote! {
            service aeb {
                let event pedestrian: float = merge(pedestrian_l, pedestrian_r);
                let event timeout_pedestrian: unit = timeout(pedestrian, 2000);
                brakes = braking_state(pedestrian, timeout_pedestrian, speed_km_h);
            }
        };
    }

    #[test]
    fn component() {
        let _: Component = parse_quote! {
            component counter(res: bool, tick: bool) -> (o: int) {
                o = if res then 0 else (last o init 0) + inc;
                let inc: int = if tick then 1 else 0;
            }
        };
    }

    #[test]
    fn component_import() {
        let _: ComponentImport = parse_quote! {
            import component grust::grust_std::rising_edge: (test: bool) -> (res: bool);
        };
    }

    #[cfg(test)]
    mod parse_stream {
        prelude! {
            stream::{Expr, ReactExpr, Last, Emit, When},
            expr::*,
        }

        #[test]
        fn should_parse_last() {
            let expression: ReactExpr = syn::parse_quote! {last x};
            let control = ReactExpr::expr(Expr::last(Last::new(syn::parse_quote! {x}, None)));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_initialized_last() {
            let expression: ReactExpr = syn::parse_quote! {last x init 0};
            let control = ReactExpr::expr(Expr::last(Last::new(
                syn::parse_quote! {x},
                Some(Expr::cst(Constant::int(syn::parse_quote! {0}))),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_constant() {
            let expression: ReactExpr = syn::parse_quote! {1};
            let control = ReactExpr::expr(Expr::cst(Constant::int(syn::parse_quote! {1})));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_identifier() {
            let expression: ReactExpr = syn::parse_quote! {x};
            let control = ReactExpr::expr(Expr::ident("x"));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_application() {
            let expression: ReactExpr = syn::parse_quote! {f(x)};
            let control = ReactExpr::expr(Expr::app(Application::new(
                Expr::ident("f"),
                vec![Expr::ident("x")],
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_binop() {
            let expression: ReactExpr = syn::parse_quote! {a+b};
            let control = ReactExpr::expr(Expr::binop(Binop::new(
                BOp::Add,
                Expr::ident("a"),
                Expr::ident("b"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_binop_with_precedence() {
            let expression: ReactExpr = syn::parse_quote! {a+b*c};
            let control = ReactExpr::expr(Expr::binop(Binop::new(
                BOp::Add,
                Expr::ident("a"),
                Expr::Binop(Binop::new(BOp::Mul, Expr::ident("b"), Expr::ident("c"))),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_typed_abstraction() {
            let expression: ReactExpr = syn::parse_quote! {|x: int| f(x)};
            let control = ReactExpr::expr(Expr::type_abstraction(TypedAbstraction::new(
                vec![("x".into(), Typ::int())],
                Expr::app(Application::new(Expr::ident("f"), vec![Expr::ident("x")])),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_structure() {
            let expression: ReactExpr = syn::parse_quote! {Point {x: 0, y: 1}};
            let control = ReactExpr::expr(Expr::structure(Structure::new(
                "Point",
                vec![
                    ("x".into(), Expr::cst(Constant::int(syn::parse_quote! {0}))),
                    ("y".into(), Expr::cst(Constant::int(syn::parse_quote! {1}))),
                ],
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_tuple() {
            let expression: ReactExpr = syn::parse_quote! {(x, 0)};
            let control = ReactExpr::expr(Expr::tuple(Tuple::new(vec![
                Expr::ident("x"),
                Expr::cst(Constant::int(syn::parse_quote! {0})),
            ])));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_enumeration() {
            let expression: ReactExpr = syn::parse_quote! {Color::Pink};
            let control = ReactExpr::expr(Expr::enumeration(Enumeration::new("Color", "Pink")));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_array() {
            let expression: ReactExpr = syn::parse_quote! {[1, 2, 3]};
            let control = ReactExpr::expr(Expr::array(Array::new(vec![
                Expr::cst(Constant::int(syn::parse_quote! {1})),
                Expr::cst(Constant::int(syn::parse_quote! {2})),
                Expr::cst(Constant::int(syn::parse_quote! {3})),
            ])));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_match() {
            let expression: ReactExpr = syn::parse_quote! {
                match a {
                    Point {x: 0, y: _} => 0,
                    Point {x: x, y: _} if f(x) => -1,
                    _ => 1,
                }
            };
            let control = ReactExpr::expr(Expr::pat_match(Match::new(
                Expr::ident("a"),
                vec![
                    Arm::new(
                        Pattern::Structure(PatStructure::new(
                            "Point",
                            vec![
                                (
                                    "x".into(),
                                    Some(Pattern::Constant(Constant::int(syn::parse_quote! {0}))),
                                ),
                                ("y".into(), Some(Pattern::Default)),
                            ],
                            None,
                        )),
                        Expr::cst(Constant::int(syn::parse_quote! {0})),
                    ),
                    Arm {
                        pattern: Pattern::Structure(PatStructure::new(
                            "Point",
                            vec![
                                ("x".into(), Some(Pattern::ident("x"))),
                                ("y".into(), Some(Pattern::Default)),
                            ],
                            None,
                        )),
                        guard: Some(Expr::app(Application::new(
                            Expr::ident("f"),
                            vec![Expr::ident("x")],
                        ))),
                        expr: Expr::cst(Constant::int(syn::parse_quote! {-1})),
                    },
                    Arm::new(
                        Pattern::Default,
                        Expr::cst(Constant::int(syn::parse_quote! {1})),
                    ),
                ],
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_field_access() {
            let expression: ReactExpr = syn::parse_quote! {p.x};
            let control =
                ReactExpr::expr(Expr::field_access(FieldAccess::new(Expr::ident("p"), "x")));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_tuple_element_access() {
            let expression: ReactExpr = syn::parse_quote! {t.0};
            let control = ReactExpr::expr(Expr::tuple_access(TupleElementAccess::new(
                Expr::ident("t"),
                0,
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_map() {
            let expression: ReactExpr = syn::parse_quote! {a.map(f)};
            let control = ReactExpr::expr(Expr::map(Map::new(Expr::ident("a"), Expr::ident("f"))));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_fold() {
            let expression: ReactExpr = syn::parse_quote! {a.fold(0, sum)};
            let control = ReactExpr::expr(Expr::fold(Fold::new(
                Expr::ident("a"),
                Expr::cst(Constant::int(syn::parse_quote! {0})),
                Expr::ident("sum"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_sort() {
            let expression: ReactExpr = syn::parse_quote! {a.sort(order)};
            let control = ReactExpr::expr(Expr::sort(Sort::new(
                Expr::ident("a"),
                Expr::ident("order"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_zip() {
            let expression: ReactExpr = syn::parse_quote! {zip(a, b, c)};
            let control = ReactExpr::expr(Expr::zip(Zip::new(vec![
                Expr::ident("a"),
                Expr::ident("b"),
                Expr::ident("c"),
            ])));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_emit() {
            let expression: ReactExpr = syn::parse_quote! {emit 0};
            let control = ReactExpr::expr(Expr::emit(Emit::new(
                Default::default(),
                Expr::cst(Constant::int(syn::parse_quote! {0})),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_when() {
            let expression: ReactExpr = syn::parse_quote! {when let d = p? then emit x};
            let control = ReactExpr::when_match(When::new(
                equation::EventPattern::Let(equation::LetEventPattern::new(
                    Default::default(),
                    expr::Pattern::ident("d"),
                    Default::default(),
                    format_ident!("p"),
                    Default::default(),
                )),
                None,
                Default::default(),
                Expr::emit(Emit::new(Default::default(), Expr::ident("x"))),
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_when_with_guard() {
            let expression: ReactExpr = syn::parse_quote! {when p? if p > 0 then emit x};
            let control = ReactExpr::when_match(When::new(
                equation::EventPattern::Let(equation::LetEventPattern::new(
                    Default::default(),
                    expr::Pattern::ident("p"),
                    Default::default(),
                    format_ident!("p"),
                    Default::default(),
                )),
                Some(Expr::binop(Binop::new(
                    BOp::Grt,
                    Expr::ident("p"),
                    Expr::cst(Constant::Integer(syn::parse_quote! {0})),
                ))),
                Default::default(),
                Expr::emit(Emit::new(Default::default(), Expr::ident("x"))),
            ));
            assert_eq!(expression, control)
        }
    }

    mod parse_expr {
        use super::*;
        use expr::*;

        #[test]
        fn parse_constant_pat() {
            let pattern: Pattern = parse_quote! {1};
            let control = Pattern::cst(Constant::int(parse_quote! {1}));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_identifier_pat() {
            let pattern: Pattern = parse_quote! {x};
            let control = Pattern::ident("x");
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_structure_pat() {
            let pattern: Pattern = parse_quote! {
                Point {
                    x: 0,
                    y: _,
                }
            };
            let control = Pattern::structure(PatStructure::new(
                "Point",
                vec![
                    (
                        "x".into(),
                        Some(Pattern::cst(Constant::int(parse_quote! {0}))),
                    ),
                    ("y".into(), Some(Pattern::default())),
                ],
                None,
            ));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_structure_with_not_renamed_field_pat() {
            let pattern: Pattern = parse_quote! {
                Point { x: 0, y, }
            };
            let control = Pattern::structure(PatStructure::new(
                "Point",
                vec![
                    (
                        "x".into(),
                        Some(Pattern::cst(Constant::int(parse_quote! {0}))),
                    ),
                    ("y".into(), None),
                ],
                None,
            ));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_structure_with_unspecified_field_pat() {
            let pattern: Pattern = parse_quote! {
                Point { x: 0, .. }
            };
            let control = Pattern::structure(PatStructure::new(
                "Point",
                vec![(
                    "x".into(),
                    Some(Pattern::cst(Constant::int(parse_quote! {0}))),
                )],
                Some(parse_quote!(..)),
            ));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_tuple_pat() {
            let pattern: Pattern = parse_quote! {(x, 0)};
            let control = Pattern::tuple(PatTuple::new(vec![
                Pattern::ident("x"),
                Pattern::cst(Constant::int(parse_quote! {0})),
            ]));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_enumeration_pat() {
            let pattern: Pattern = parse_quote! {Color::Pink};
            let control = Pattern::enumeration(PatEnumeration::new("Color", "Pink"));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_default_pat() {
            let pattern: Pattern = parse_quote! {_};
            let control = Pattern::default();
            assert_eq!(pattern, control)
        }

        #[test]
        fn should_parse_constant() {
            let expr: Expr = parse_quote! {1};
            let control = Expr::cst(Constant::int(parse_quote! {1}));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_identifier() {
            let expr: Expr = parse_quote! {x};
            let control = Expr::ident("x");
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_application() {
            let expr: Expr = parse_quote! {f(x)};
            let control = Expr::app(Application::new(Expr::ident("f"), vec![Expr::ident("x")]));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_binop() {
            let expr: Expr = parse_quote! {a+b};
            let control = Expr::binop(Binop::new(BOp::Add, Expr::ident("a"), Expr::ident("b")));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_binop_with_precedence() {
            let expr: Expr = parse_quote! {a+b*c};
            let control = Expr::binop(Binop::new(
                BOp::Add,
                Expr::ident("a"),
                Expr::binop(Binop::new(BOp::Mul, Expr::ident("b"), Expr::ident("c"))),
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_binop_with_unop() {
            let term: Expr = parse_quote! {-x + 1};
            let control = Expr::binop(Binop::new(
                BOp::Add,
                Expr::unop(UnOp::new(UOp::Neg, Expr::ident("x"))),
                Expr::constant(Constant::int(parse_quote! {1})),
            ));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_typed_abstraction() {
            let expr: Expr = parse_quote! {|x: int| f(x)};
            let control = Expr::typed_abstraction(TypedAbstraction::new(
                vec![("x".into(), Typ::int())],
                Expr::app(Application::new(Expr::ident("f"), vec![Expr::ident("x")])),
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_structure() {
            let expr: Expr = parse_quote! {Point {x: 0, y: 1}};
            let control = Expr::structure(Structure::new(
                "Point",
                vec![
                    ("x".into(), Expr::cst(Constant::int(parse_quote! {0}))),
                    ("y".into(), Expr::cst(Constant::int(parse_quote! {1}))),
                ],
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_tuple() {
            let expr: Expr = parse_quote! {(x, 0)};
            let control = Expr::tuple(Tuple::new(vec![
                Expr::ident("x"),
                Expr::cst(Constant::int(parse_quote! {0})),
            ]));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_enumeration() {
            let expr: Expr = parse_quote! {Color::Pink};
            let control = Expr::enumeration(Enumeration::new("Color", "Pink"));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_array() {
            let expr: Expr = parse_quote! {[1, 2, 3]};
            let control = Expr::array(Array::new(vec![
                Expr::cst(Constant::int(parse_quote! {1})),
                Expr::cst(Constant::int(parse_quote! {2})),
                Expr::cst(Constant::int(parse_quote! {3})),
            ]));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_match() {
            let expr: Expr = parse_quote! {
                match a {
                    Point {x: 0, y: _} => 0,
                    Point {x: x, y: _} if f(x) => -1,
                    _ => 1,
                }
            };
            let control = Expr::pat_match(Match::new(
                Expr::ident("a"),
                vec![
                    Arm::new(
                        Pattern::structure(PatStructure::new(
                            "Point",
                            vec![
                                (
                                    "x".into(),
                                    Some(Pattern::cst(Constant::int(parse_quote! {0}))),
                                ),
                                ("y".into(), Some(Pattern::default())),
                            ],
                            None,
                        )),
                        Expr::Constant(Constant::Integer(parse_quote! {0})),
                    ),
                    Arm::new_with_guard(
                        Pattern::Structure(PatStructure::new(
                            "Point",
                            vec![
                                ("x".into(), Some(Pattern::ident("x"))),
                                ("y".into(), Some(Pattern::default())),
                            ],
                            None,
                        )),
                        Expr::cst(Constant::int(parse_quote! {-1})),
                        Some(Expr::app(Application::new(
                            Expr::ident("f"),
                            vec![Expr::ident("x")],
                        ))),
                    ),
                    Arm::new(Pattern::Default, Expr::cst(Constant::int(parse_quote! {1}))),
                ],
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_field_access() {
            let expression: Expr = parse_quote! {p.x};
            let control = Expr::field_access(FieldAccess::new(Expr::ident("p"), "x"));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_tuple_element_access() {
            let expression: Expr = parse_quote! {t.0};
            let control = Expr::tuple_access(TupleElementAccess::new(Expr::ident("t"), 0));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_map() {
            let expression: Expr = parse_quote! {a.map(f)};
            let control = Expr::map(Map::new(Expr::ident("a"), Expr::ident("f")));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_fold() {
            let expression: Expr = parse_quote! {a.fold(0, sum)};
            let control = Expr::fold(Fold::new(
                Expr::ident("a"),
                Expr::cst(Constant::int(parse_quote! {0})),
                Expr::ident("sum"),
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_sort() {
            let expression: Expr = parse_quote! {a.sort(order)};
            let control = Expr::sort(Sort::new(Expr::ident("a"), Expr::ident("order")));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_zip() {
            let expression: Expr = parse_quote! {zip(a, b, c)};
            let control = Expr::zip(Zip::new(vec![
                Expr::ident("a"),
                Expr::ident("b"),
                Expr::ident("c"),
            ]));
            assert_eq!(expression, control)
        }
    }

    mod equations {
        use super::*;
        use equation::*;

        #[test]
        fn should_parse_output_definition() {
            let equation: ReactEq = parse_quote! {o = if res then 0 else (last o init 0) + inc;};
            let control = ReactEq::out_def(Instantiation {
                pattern: parse_quote! {o},
                eq_token: parse_quote! {=},
                expr: stream::ReactExpr::expr(stream::Expr::ite(expr::IfThenElse::new(
                    stream::Expr::ident("res"),
                    stream::Expr::cst(Constant::int(parse_quote! {0})),
                    stream::Expr::binop(expr::Binop::new(
                        BOp::Add,
                        stream::Expr::last(stream::Last::new(
                            parse_quote! {o},
                            Some(stream::Expr::cst(Constant::int(parse_quote! {0}))),
                        )),
                        stream::Expr::ident("inc"),
                    )),
                ))),
                semi_token: parse_quote! {;},
            });
            assert_eq!(equation, control)
        }

        #[test]
        fn should_parse_tuple_instantiation() {
            let equation: ReactEq = parse_quote! {
                (o1, o2) = if res then (0, 0) else ((last o1 init 0) + inc1, last o2 + inc2);
            };
            let control = ReactEq::out_def(Instantiation {
                pattern: stmt::Pattern::tuple(stmt::Tuple::new(vec![
                    parse_quote! {o1},
                    parse_quote! {o2},
                ])),
                eq_token: parse_quote! {=},
                expr: stream::ReactExpr::expr(stream::Expr::ite(expr::IfThenElse::new(
                    stream::Expr::ident("res"),
                    stream::Expr::tuple(expr::Tuple::new(vec![
                        stream::Expr::cst(Constant::int(parse_quote! {0})),
                        stream::Expr::cst(Constant::int(parse_quote! {0})),
                    ])),
                    stream::Expr::tuple(expr::Tuple::new(vec![
                        stream::Expr::binop(expr::Binop::new(
                            BOp::Add,
                            stream::Expr::last(stream::Last::new(
                                parse_quote! {o1},
                                Some(stream::Expr::cst(Constant::int(parse_quote! {0}))),
                            )),
                            stream::Expr::ident("inc1"),
                        )),
                        stream::Expr::binop(expr::Binop::new(
                            BOp::Add,
                            stream::Expr::last(stream::Last::new(parse_quote! {o2}, None)),
                            stream::Expr::ident("inc2"),
                        )),
                    ])),
                ))),
                semi_token: parse_quote! {;},
            });
            assert_eq!(equation, control)
        }

        #[test]
        fn should_parse_local_definition() {
            let equation: ReactEq = parse_quote! {
                let o: int = if res then 0 else last o + inc;
            };
            let control = ReactEq::local_def(stmt::LetDecl::new(
                parse_quote!(let),
                stmt::Pattern::typed(stmt::Typed {
                    ident: parse_quote!(o),
                    colon_token: parse_quote!(:),
                    typ: Typ::int(),
                }),
                parse_quote!(=),
                stream::ReactExpr::expr(stream::Expr::ite(expr::IfThenElse::new(
                    stream::Expr::ident("res"),
                    stream::Expr::cst(Constant::int(parse_quote! {0})),
                    stream::Expr::binop(expr::Binop::new(
                        BOp::Add,
                        stream::Expr::last(stream::Last::new(parse_quote! {o}, None)),
                        stream::Expr::ident("inc"),
                    )),
                ))),
                parse_quote! {;},
            ));
            assert_eq!(equation, control)
        }

        #[test]
        fn should_parse_multiple_definitions() {
            let equation: ReactEq = parse_quote! {
                let (o1: int, o2: int) =
                    if res then (0, 0) else ((last o1 init 0) + inc1, last o2 + inc2);
            };
            let control = ReactEq::local_def(stmt::LetDecl::new(
                parse_quote!(let),
                stmt::Pattern::tuple(stmt::Tuple::new(vec![
                    stmt::Pattern::Typed(stmt::Typed {
                        ident: parse_quote!(o1),
                        colon_token: parse_quote!(:),
                        typ: Typ::int(),
                    }),
                    stmt::Pattern::Typed(stmt::Typed {
                        ident: parse_quote!(o2),
                        colon_token: parse_quote!(:),
                        typ: Typ::int(),
                    }),
                ])),
                parse_quote!(=),
                stream::ReactExpr::expr(stream::Expr::ite(expr::IfThenElse::new(
                    stream::Expr::ident("res"),
                    stream::Expr::tuple(expr::Tuple::new(vec![
                        stream::Expr::cst(Constant::int(parse_quote! {0})),
                        stream::Expr::cst(Constant::int(parse_quote! {0})),
                    ])),
                    stream::Expr::tuple(expr::Tuple::new(vec![
                        stream::Expr::binop(expr::Binop::new(
                            BOp::Add,
                            stream::Expr::last(stream::Last::new(
                                parse_quote! {o1},
                                Some(stream::Expr::cst(Constant::int(parse_quote! {0}))),
                            )),
                            stream::Expr::ident("inc1"),
                        )),
                        stream::Expr::binop(expr::Binop::new(
                            BOp::Add,
                            stream::Expr::last(stream::Last::new(parse_quote! {o2}, None)),
                            stream::Expr::ident("inc2"),
                        )),
                    ])),
                ))),
                parse_quote! {;},
            ));
            assert_eq!(equation, control)
        }
    }

    mod contracts {
        use super::*;
        use contract::*;

        #[test]
        fn should_parse_constant() {
            let term: Term = parse_quote! {1};
            let control = Term::constant(Constant::int(parse_quote! {1}));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_identifier() {
            let term: Term = parse_quote! {x};
            let control = Term::ident("x");
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_unary_operation() {
            let term: Term = parse_quote! {!x};
            let control = Term::unary(Unary::new(UOp::Not, Term::ident("x")));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_binary_operation() {
            let term: Term = parse_quote! {-x + 1};
            let control = Term::binary(Binary::new(
                Term::unary(Unary::new(UOp::Neg, Term::ident("x"))),
                BOp::Add,
                Term::constant(Constant::int(parse_quote! {1})),
            ));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_implication() {
            let term: Term = parse_quote! { !x && y => z};
            let control = Term::implication(Implication::new(
                Term::binary(Binary::new(
                    Term::unary(Unary::new(UOp::Not, Term::ident("x"))),
                    BOp::And,
                    Term::ident("y"),
                )),
                Default::default(),
                Term::ident("z"),
            ));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_event_implication() {
            let term: Term = parse_quote! { when d = p? => d > x+y};
            let control = Term::event(EventImplication::new(
                Default::default(),
                "d",
                Default::default(),
                "p",
                Default::default(),
                Default::default(),
                Term::binary(Binary::new(
                    Term::ident("d"),
                    BOp::Grt,
                    Term::binary(Binary::new(Term::ident("x"), BOp::Add, Term::ident("y"))),
                )),
            ));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_forall() {
            let term: Term = parse_quote! { forall d: int, d > x+y};
            let control = Term::forall(ForAll::new(
                Default::default(),
                "d",
                Default::default(),
                Typ::int(),
                Default::default(),
                Term::binary(Binary::new(
                    Term::ident("d"),
                    BOp::Grt,
                    Term::binary(Binary::new(Term::ident("x"), BOp::Add, Term::ident("y"))),
                )),
            ));
            assert_eq!(term, control)
        }
    }
}
