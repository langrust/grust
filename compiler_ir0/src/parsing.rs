//! Parsing.

prelude! {
    syn::{Parse, Punctuated, token, LitInt, Res},
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
        // parse attributes if any
        let attrs = input.call(syn::Attribute::parse_outer)?;
        macro_rules! no_attrs {
            {} => {
                if let Some(next) = attrs.into_iter().next() {
                    return Err(syn::Error::new_spanned(next, "unexpected attribute"))
                }
            };
        }
        if ComponentImport::peek(input) {
            no_attributes!();
            Ok(Item::ComponentImport(input.parse()?))
        } else if Component::peek(input) {
            Ok(Item::Component(Component::parse_item(input, attrs)?))
        } else if Function::peek(input) {
            Ok(Item::Function(Function::parse_item(input, attrs)?))
        } else if Typedef::peek(input) {
            no_attrs!();
            Ok(Item::Typedef(input.parse()?))
        } else if Service::peek(input) {
            no_attrs!();
            Ok(Item::Service(input.parse()?))
        } else if FlowImport::peek(input) {
            no_attrs!();
            Ok(Item::Import(input.parse()?))
        } else if FlowExport::peek(input) {
            no_attrs!();
            Ok(Item::Export(input.parse()?))
        } else if ExtFunDecl::peek(input) {
            Ok(Item::ExtFun(ExtFunDecl::parse_item(input, attrs)?))
        } else if ExtCompDecl::peek(input) {
            Ok(Item::ExtComp(ExtCompDecl::parse_item(input, attrs)?))
        } else {
            Err(input.error(
                "expected either a flow import/export, a type, a component or function \
                definition/import, or a service definition",
            ))
        }
    }
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Res<Self> {
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

impl Parse for Top {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let conf: Conf = input.parse()?;
        let ast: Ast = input.parse()?;
        Ok(Self { conf, ast })
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

mod interface {
    use super::*;
    prelude! { just
        interface::*, ParseItem
    }

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
            let sample = if content.peek(LitInt) {
                let period_ms: LitInt = content.parse()?;
                Sample::new_lit(sample_token, paren_token, expr, comma_token, period_ms)
            } else {
                let period_ms: Ident = content.parse()?;
                Sample::new_id(sample_token, paren_token, expr, comma_token, period_ms)
            };
            if content.is_empty() {
                Ok(sample)
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
            let scan = if content.peek(LitInt) {
                let period_ms: LitInt = content.parse()?;
                Scan::new_lit(scan_token, paren_token, expr, comma_token, period_ms)
            } else {
                let period_ms: Ident = content.parse()?;
                Scan::new_id(scan_token, paren_token, expr, comma_token, period_ms)
            };
            if content.is_empty() {
                Ok(scan)
            } else {
                Err(content.error("expected two input expressions"))
            }
        }
    }

    impl Period {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::period)
        }
    }
    impl Parse for Period {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let period_token: keyword::period = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let period = if content.peek(LitInt) {
                let period_ms: LitInt = content.parse()?;
                Period::new_lit(period_token, paren_token, period_ms)
            } else {
                let period_ms: Ident = content.parse()?;
                Period::new_id(period_token, paren_token, period_ms)
            };
            if content.is_empty() {
                Ok(period)
            } else {
                Err(content.error("expected two input expressions"))
            }
        }
    }

    impl SampleOn {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::sample_on)
        }
    }
    impl Parse for SampleOn {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let sample_on_token: keyword::sample_on = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr: FlowExpression = content.parse()?;
            let comma_token: Token![,] = content.parse()?;
            let event: FlowExpression = content.parse()?;
            if content.is_empty() {
                Ok(SampleOn::new(
                    sample_on_token,
                    paren_token,
                    expr,
                    comma_token,
                    event,
                ))
            } else {
                Err(content.error("expected two input expressions"))
            }
        }
    }

    impl ScanOn {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::scan_on)
        }
    }
    impl Parse for ScanOn {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let scan_on_token: keyword::scan_on = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr: FlowExpression = content.parse()?;
            let comma_token: Token![,] = content.parse()?;
            let event: FlowExpression = content.parse()?;
            if content.is_empty() {
                Ok(ScanOn::new(
                    scan_on_token,
                    paren_token,
                    expr,
                    comma_token,
                    event,
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
                weight: None,
            })
        }
    }
    impl ParseItem for Function {
        const DESC: &str = "function";

        fn parse_attributes(mut self, attrs: Vec<syn::Attribute>) -> syn::Res<Self> {
            prelude! {}
            for attr in attrs {
                let span = attr.bracket_token.span.join();
                if let Some(w) = attr.meta.parse_weight_percent_hint()? {
                    if self.weight.is_some() {
                        let msg = format!("this {} already has a weight percent hint", Self::DESC);
                        return Err(syn::Error::new(span, msg));
                    } else {
                        self.weight = Some(w)
                    }
                } else {
                    return Err(syn::Error::new(span, "unexpected attribute name"));
                }
            }
            Ok(self)
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
            let timeout = if content.peek(LitInt) {
                let deadline: LitInt = content.parse()?;
                Timeout::new_lit(timeout_token, paren_token, expr, comma_token, deadline)
            } else {
                let deadline: Ident = content.parse()?;
                Timeout::new_id(timeout_token, paren_token, expr, comma_token, deadline)
            };
            if content.is_empty() {
                Ok(timeout)
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

    impl Persist {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::persist)
        }
    }
    impl Parse for Persist {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let persist_token: keyword::persist = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let expr: FlowExpression = content.parse()?;
            if content.is_empty() {
                Ok(Persist::new(persist_token, paren_token, expr))
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

    impl Time {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::time)
        }
    }
    impl Parse for Time {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let time_token: keyword::time = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            if content.is_empty() {
                Ok(Time::new(time_token, paren_token))
            } else {
                Err(content.error("no input expected"))
            }
        }
    }

    impl Parse for Call {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let ident: Ident = input.parse()?;
            let content;
            let paren_token: token::Paren = parenthesized!(content in input);
            let inputs: Punctuated<FlowExpression, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Call {
                ident,
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
            } else if Persist::peek(input) {
                Ok(Self::persist(input.parse()?))
            } else if Merge::peek(input) {
                Ok(Self::merge(input.parse()?))
            } else if Time::peek(input) {
                Ok(Self::time(input.parse()?))
            } else if Period::peek(input) {
                Ok(Self::period(input.parse()?))
            } else if SampleOn::peek(input) {
                Ok(Self::sample_on(input.parse()?))
            } else if ScanOn::peek(input) {
                Ok(Self::scan_on(input.parse()?))
            } else if input.fork().call(Call::parse).is_ok() {
                Ok(Self::comp_call(input.parse()?))
            } else {
                let ident: Ident = input.parse()?;
                Ok(Self::ident(ident))
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

    impl ExtFunDecl {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            let res = forked.parse::<Token![use]>().is_ok();
            res && forked.parse::<keyword::function>().is_ok()
        }
    }
    impl Parse for ExtFunDecl {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let use_token: Token![use] = input.parse()?;
            let function_token: keyword::function = input.parse()?;
            let path: syn::Path = input.parse()?;
            let ident = if let Some(last) = path.segments.last() {
                last.ident.clone()
            } else {
                return Err(syn::Error::new_spanned(path, "illegal function path"));
            };
            let content;
            let args_paren = parenthesized!(content in input);
            let args: Punctuated<Colon<Ident, Typ>, syn::Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let arrow_token = input.parse()?;
            let output_typ: Typ = input.parse()?;
            let semi_token = input.parse()?;
            let full_typ = Typ::Fn {
                paren_token: Some(args_paren),
                inputs: args.iter().map(|pair| pair.right.clone()).collect(),
                arrow_token,
                output: Box::new(output_typ.clone()),
            };
            Ok(ExtFunDecl {
                use_token,
                function_token,
                path,
                ident,
                args_paren,
                args,
                arrow_token,
                output_typ,
                semi_token,
                full_typ,
                weight: None,
            })
        }
    }
    impl ParseItem for ExtFunDecl {
        const DESC: &'static str = "external function";

        fn parse_attributes(mut self, attrs: Vec<syn::Attribute>) -> syn::Res<Self> {
            prelude! {}
            for attr in attrs {
                let span = attr.bracket_token.span.join();
                if let Some(w) = attr.meta.parse_weight_percent_hint()? {
                    if self.weight.is_some() {
                        let msg = format!("this {} already has a weight percent hint", Self::DESC);
                        return Err(syn::Error::new(span, msg));
                    } else {
                        self.weight = Some(w)
                    }
                } else {
                    return Err(syn::Error::new(span, "unexpected attribute name"));
                }
            }
            Ok(self)
        }
    }

    impl ExtCompDecl {
        pub fn peek(input: ParseStream) -> bool {
            let forked = input.fork();
            let res = forked.parse::<Token![use]>().is_ok();
            res && forked.parse::<keyword::component>().is_ok()
        }
    }
    impl Parse for ExtCompDecl {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let use_token: Token![use] = input.parse()?;
            let component_token: keyword::component = input.parse()?;
            let path: syn::Path = input.parse()?;
            let ident = if let Some(last) = path.segments.last() {
                last.ident.clone()
            } else {
                return Err(syn::Error::new_spanned(path, "illegal function path"));
            };
            let content;
            let args_paren = parenthesized!(content in input);
            let args: Punctuated<Colon<Ident, Typ>, syn::Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let arrow_token = input.parse()?;
            let content;
            let outs_paren = parenthesized!(content in input);
            let outs: Punctuated<Colon<Ident, Typ>, syn::Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let semi_token = input.parse()?;
            Ok(ExtCompDecl {
                use_token,
                component_token,
                path,
                ident,
                args_paren,
                args,
                arrow_token,
                outs_paren,
                outs,
                semi_token,
                weight: None,
            })
        }
    }
    impl ParseItem for ExtCompDecl {
        const DESC: &str = "external component";

        fn parse_attributes(mut self, attrs: Vec<syn::Attribute>) -> syn::Res<Self> {
            prelude! {}
            for attr in attrs {
                let span = attr.bracket_token.span.join();
                if let Some(w) = attr.meta.parse_weight_percent_hint()? {
                    if self.weight.is_some() {
                        let msg = format!("this {} already has a weight percent hint", Self::DESC);
                        return Err(syn::Error::new(span, msg));
                    } else {
                        self.weight = Some(w)
                    }
                } else {
                    return Err(syn::Error::new(span, "unexpected attribute name"));
                }
            }
            Ok(self)
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
            let min: Either<LitInt, Ident> = if content.peek(LitInt) {
                Either::Left(content.parse()?)
            } else {
                Either::Right(content.parse()?)
            };
            let comma_token: token::Comma = content.parse()?;
            let max: Either<LitInt, Ident> = if content.peek(LitInt) {
                Either::Left(content.parse()?)
            } else {
                Either::Right(content.parse()?)
            };
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

impl Component {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(keyword::component)
    }
}
impl syn::Parse for Component {
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
            contract,
            brace,
            equations,
            weight: None,
        })
    }
}
impl ParseItem for Component {
    const DESC: &str = "component";

    fn parse_attributes(mut self, attrs: Vec<syn::Attribute>) -> syn::Res<Self> {
        prelude! {}
        for attr in attrs {
            let span = attr.bracket_token.span.join();
            if let Some(w) = attr.meta.parse_weight_percent_hint()? {
                if self.weight.is_some() {
                    let msg = format!("this {} already has a weight percent hint", Self::DESC);
                    return Err(syn::Error::new(span, msg));
                } else {
                    self.weight = Some(w)
                }
            } else {
                return Err(syn::Error::new(span, "unexpected attribute name"));
            }
        }
        Ok(self)
    }
}

impl ConstDecl {
    pub fn peek(input: ParseStream) -> bool {
        input.peek(Token![const])
    }
}
impl Parse for ConstDecl {
    fn parse(input: ParseStream) -> Res<Self> {
        let const_token: Token![const] = input.parse()?;
        let ident: Ident = input.parse()?;
        let colon_token: Token![:] = input.parse()?;
        let ty: Typ = input.parse()?;
        let eq_token: Token![=] = input.parse()?;
        let value: Constant = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(ConstDecl {
            const_token,
            ident,
            colon_token,
            ty,
            eq_token,
            value,
            semi_token,
        })
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
                Err(content.error("expected array alias definition"))
            }
        } else {
            Err(input.error("expected type definition"))
        }
    }
}

mod parse_stream {
    use super::*;
    use expr::BinOp;
    use stream::*;

    impl Last {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::last)
        }
    }
    impl Parse for Last {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let kw: keyword::last = input.parse()?;
            let mut loc = Loc::from(kw.span);
            let ident: Ident = input.parse()?;
            loc = loc.join(ident.loc());
            Ok(Last::new(loc, ident))
        }
    }

    impl Parse for EventArmWhen {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let pattern = input.parse()?;
            let guard = {
                if input.peek(Token![if]) {
                    let _if_token: Token![if] = input.parse()?;
                    let guard = input.parse()?;
                    Some(guard)
                } else {
                    None
                }
            };
            let _arrow_token: Token![=>] = input.parse()?;
            let expr = input.parse()?;
            Ok(EventArmWhen::new(pattern, guard, expr))
        }
    }

    impl InitArmWhen {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::init)
        }
    }
    impl Parse for InitArmWhen {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let init_token = input.parse()?;
            let arrow_token = input.parse()?;
            let expr = input.parse()?;
            let _coma_token: Token![,] = input.parse()?;
            Ok(InitArmWhen::new(init_token, arrow_token, expr))
        }
    }

    impl WhenExpr {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::when)
        }
    }
    impl Parse for WhenExpr {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let when_token: keyword::when = input.parse()?;
            let content;
            let _ = braced!(content in input);
            let init = {
                if InitArmWhen::peek(&content) {
                    let init = content.parse()?;
                    Some(init)
                } else {
                    None
                }
            };
            let arms: Punctuated<EventArmWhen, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(WhenExpr::new(when_token, init, arms.into_iter().collect()))
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
            Ok(Emit::new(
                Loc::from(emit_token.span).join(expr.loc()),
                emit_token,
                expr,
            ))
        }
    }

    impl ParsePrec for stream::Expr {
        fn parse_term(input: ParseStream) -> syn::Res<Self> {
            // #TODO: have a cheap peeking for complex expressions
            let mut expression = if input.fork().call(Constant::parse).is_ok() {
                Self::Constant(input.parse()?)
            } else if Last::peek(input) {
                Self::Last(input.parse()?)
            } else if expr::UnOp::<Self>::peek(input) {
                Self::UnOp(input.parse()?)
            } else if expr::Zip::<Self>::peek(input) {
                Self::Zip(input.parse()?)
            } else if expr::MatchExpr::<Self>::peek(input) {
                Self::MatchExpr(input.parse()?)
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
                Self::Identifier(input.parse()?)
            } else {
                return Err(input.error("expected expression"));
            };
            loop {
                if input.peek(Token![^]) {
                    let op_loc = input.span().into();
                    let _power_token: Token![^] = input.parse()?;
                    let power: LitInt = input.parse()?;
                    let right = expression.clone();
                    for _ in 0..power.base10_parse::<u64>().unwrap() - 1 {
                        let op = BOp::Mul;
                        expression = Self::BinOp(BinOp::new(op, op_loc, expression, right.clone()));
                    }
                } else if expr::Sort::<Self>::peek(input) {
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
                    expression = Self::BinOp(expr::BinOp::<Self>::parse_term(expression, input)?);
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
                    expression = Self::BinOp(expr::BinOp::<Self>::parse_prec1(expression, input)?);
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
                    expression = Self::BinOp(expr::BinOp::<Self>::parse_prec2(expression, input)?);
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
                    expression = Self::BinOp(expr::BinOp::<Self>::parse_prec3(expression, input)?);
                } else {
                    break;
                }
            }
            Ok(expression)
        }
    }
    impl Parse for stream::Expr {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expression = if expr::Lambda::<Self>::peek(input) {
                Self::Lambda(input.parse()?)
            } else if expr::IfThenElse::<Self>::peek(input) {
                Self::IfThenElse(input.parse()?)
            } else if stream::Emit::peek(input) {
                Self::Emit(input.parse()?)
            } else if stream::WhenExpr::peek(input) {
                return Err(input.error("'when' should be a root expression"));
            } else {
                Self::parse_prec4(input)?
            };
            Ok(expression)
        }
    }

    impl Parse for ReactExpr {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expression = if WhenExpr::peek(input) {
                Self::when_expr(input.parse()?)
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
            let parens = parenthesized!(content in input);
            let elements: Punctuated<Pattern, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Tuple::new(
                parens.span.join(),
                elements.into_iter().collect(),
            ))
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

    impl Parse for LogStmt {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let log_token: keyword::log = input.parse()?;
            let pattern: Pattern = input.parse()?;
            let semi_token: Token![;] = input.parse()?;

            Ok(LogStmt {
                log_token,
                pattern,
                semi_token,
            })
        }
    }

    impl Parse for Stmt {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(Token![let]) {
                Ok(Stmt::Declaration(input.parse()?))
            } else if input.peek(keyword::log) {
                Ok(Stmt::Log(input.parse()?))
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
            let op_loc = input.span().into();
            let op = input.parse()?;
            let expr = Box::new(E::parse_term(input)?);
            Ok(UnOp { op, expr, op_loc })
        }
    }

    impl<E> BinOp<E>
    where
        E: ParsePrec,
    {
        pub fn peek(input: ParseStream) -> bool {
            BOp::peek(input)
        }
        pub fn parse_term(lhs: E, input: ParseStream) -> syn::Res<Self> {
            let op_loc = input.span().into();
            let op = input.parse()?;
            let rhs = E::parse_term(input)?;
            Ok(BinOp::new(op, op_loc, lhs, rhs))
        }
        pub fn parse_prec1(lhs: E, input: ParseStream) -> syn::Res<Self> {
            let op = input.parse()?;
            let op_loc = input.span().into();
            let rhs = E::parse_prec1(input)?;
            Ok(BinOp::new(op, op_loc, lhs, rhs))
        }
        pub fn parse_prec2(lhs: E, input: ParseStream) -> syn::Res<Self> {
            let op_loc = input.span().into();
            let op = input.parse()?;
            let rhs = E::parse_prec2(input)?;
            Ok(BinOp::new(op, op_loc, lhs, rhs))
        }
        pub fn parse_prec3(lhs: E, input: ParseStream) -> syn::Res<Self> {
            let op_loc = input.span().into();
            let op = input.parse()?;
            let rhs = E::parse_prec3(input)?;
            Ok(BinOp::new(op, op_loc, lhs, rhs))
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
        E: Parse + HasLoc,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let iff: Token![if] = input.parse()?;
            let cnd: E = input.parse()?;
            let _: keyword::then = input.parse()?;
            let thn: E = input.parse()?;
            let _: Token![else] = input.parse()?;
            let els: E = input.parse()?;
            Ok(IfThenElse::new(
                Loc::from(iff.span).join(els.loc()),
                cnd,
                thn,
                els,
            ))
        }
    }

    impl<E> Application<E>
    where
        E: Parse + HasLoc,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(token::Paren)
        }

        pub fn parse(function: E, input: ParseStream) -> syn::Res<Self> {
            let content;
            let parens = syn::parenthesized!(content in input);
            let inputs: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Application::new(
                function.loc().join(parens.span.join()),
                function,
                inputs.into_iter().collect(),
            ))
        }
    }
    impl<E> Parse for Application<E>
    where
        E: Parse + HasLoc,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let function: E = input.parse()?;
            let content;
            let parens = parenthesized!(content in input);
            let inputs: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Application::new(
                function.loc().join(parens.span.join()),
                function,
                inputs.into_iter().collect(),
            ))
        }
    }

    impl<E> Lambda<E> {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(Token![|])
        }
    }
    impl<E> Parse for Lambda<E> {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let open_pipe: Token![|] = input.parse()?;
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
            let expr: Expr = input.parse()?;
            Ok(Lambda::new(
                Loc::from(open_pipe.span).join(expr.loc()),
                inputs
                    .into_iter()
                    .map(|Colon { left, right, .. }| (left, right))
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
            let braces = braced!(content in input);
            let fields: Punctuated<Colon<Ident, E>, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            Ok(Structure::new(
                ident.loc().join(braces.span.join()),
                ident,
                fields
                    .into_iter()
                    .map(|Colon { left, right, .. }| (left, right))
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
            let parens = parenthesized!(content in input);
            let elements: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Tuple::new(
                parens.span.join(),
                elements.into_iter().collect(),
            ))
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
                ident_enum.loc().join(ident_elem.loc()),
                ident_enum,
                ident_elem,
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
        E: Parse + Clone,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let content;
            let brackets = bracketed!(content in input);

            if content.is_empty() {
                Ok(Array::new(brackets.span.join(), vec![]))
            } else {
                let first: E = content.parse()?;
                let elements = if content.peek(Token![;]) {
                    let _: Token![;] = content.parse()?;
                    let size: syn::LitInt = content.parse()?;
                    if content.is_empty() {
                        let n: usize = size.base10_parse()?;
                        vec![first; n]
                    } else {
                        return Err(content.error("expected closed bracket"));
                    }
                } else if content.peek(Token![,]) {
                    let _: Token![,] = content.parse()?;
                    let others: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
                    let mut elements = Vec::with_capacity(others.len() + 1);
                    elements.push(first);
                    elements.extend(others);
                    elements
                } else {
                    vec![first]
                };
                Ok(Array::new(brackets.span.join(), elements))
            }
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
            let braces = braced!(content in input);
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
                braces,
                name: ident,
                fields: fields
                    .into_iter()
                    .map(|(ident, optional_pattern)| {
                        (ident, optional_pattern.map(|(_, pattern)| pattern))
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
                enum_name: ident_enum,
                elem_name: ident_elem,
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
            let parens = parenthesized!(content in input);
            let elements: Punctuated<Pattern, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(PatTuple {
                parens,
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
                let token: Token![_] = input.parse()?;
                Pattern::Default(token.span.into())
            } else {
                let ident: Ident = input.parse()?;
                Pattern::Identifier(ident)
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
                if input.peek(Token![if]) {
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

    impl<E> MatchExpr<E>
    where
        E: Parse,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(Token![match])
        }
    }
    impl<E> Parse for MatchExpr<E>
    where
        E: Parse,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let match_token: Token![match] = input.parse()?;
            let expr: E = input.parse()?;
            let content;
            let braces = braced!(content in input);
            let arms: Punctuated<Arm<E>, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(MatchExpr::new(
                Loc::from(match_token.span).join(braces.span.join()),
                expr,
                arms.into_iter().collect(),
            ))
        }
    }

    impl<E> FieldAccess<E>
    where
        E: Parse + HasLoc,
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
            Ok(FieldAccess::new(expr.loc().join(field.loc()), expr, field))
        }
    }
    impl<E> Parse for FieldAccess<E>
    where
        E: Parse + HasLoc,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let field: Ident = input.parse()?;
            Ok(FieldAccess::new(expr.loc().join(field.loc()), expr, field))
        }
    }

    impl<E> TupleElementAccess<E>
    where
        E: Parse + HasLoc,
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
                expr.loc().join(element_number.span()),
                expr,
                element_number.base10_parse().unwrap(),
            ))
        }
    }
    impl<E> Parse for TupleElementAccess<E>
    where
        E: Parse + HasLoc,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            Self::parse(expr, input)
        }
    }

    impl<E> ArrayAccess<E>
    where
        E: Parse + HasLoc,
    {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket)
        }

        pub fn parse(expr: E, input: ParseStream) -> syn::Res<Self> {
            let content;
            let braces = bracketed!(content in input);
            let index: syn::LitInt = content.parse()?;
            let loc = expr.loc().join(braces.span.close()).join(index.span());
            Ok(ArrayAccess::new(loc, expr, index))
        }
    }
    impl<E> Parse for ArrayAccess<E>
    where
        E: Parse + HasLoc,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            Self::parse(expr, input)
        }
    }

    impl<E> Map<E>
    where
        E: Parse + HasLoc,
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
            let parens = parenthesized!(content in input);
            let fun: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr.loc().join(parens.span.join()), expr, fun))
            } else {
                Err(input.error("expects one argument"))
            }
        }
    }
    impl<E> Parse for Map<E>
    where
        E: Parse + HasLoc,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let _: keyword::map = input.parse()?;
            let content;
            let parens = parenthesized!(content in input);
            let fun: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr.loc().join(parens.span.join()), expr, fun))
            } else {
                Err(input.error("expects one argument"))
            }
        }
    }

    impl<E> Fold<E>
    where
        E: Parse + HasLoc,
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
            let parens = parenthesized!(content in input);
            let init: E = content.parse()?;
            let _: Token![,] = content.parse()?;
            let function: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(
                    expr.loc().join(parens.span.join()),
                    expr,
                    init,
                    function,
                ))
            } else {
                Err(input.error("expects two arguments"))
            }
        }
    }
    impl<E> Parse for Fold<E>
    where
        E: Parse + HasLoc,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let _: keyword::fold = input.parse()?;
            let content;
            let parens = parenthesized!(content in input);
            let init: E = content.parse()?;
            let _: Token![,] = content.parse()?;
            let function: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(
                    expr.loc().join(parens.span.join()),
                    expr,
                    init,
                    function,
                ))
            } else {
                Err(input.error("expects two arguments"))
            }
        }
    }

    impl<E> Sort<E>
    where
        E: Parse + HasLoc,
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
            let parens = parenthesized!(content in input);
            let fun: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr.loc().join(parens.span.join()), expr, fun))
            } else {
                Err(input.error("expects one argument"))
            }
        }
    }
    impl<E> Parse for Sort<E>
    where
        E: Parse + HasLoc,
    {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr: E = input.parse()?;
            let _: Token![.] = input.parse()?;
            let _: keyword::sort = input.parse()?;
            let content;
            let parens = parenthesized!(content in input);
            let fun: E = content.parse()?;
            if content.is_empty() {
                Ok(Self::new(expr.loc().join(parens.span.join()), expr, fun))
            } else {
                Err(input.error("expects one argument"))
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
            let kw: keyword::zip = input.parse()?;
            let content;
            let parens = parenthesized!(content in input);
            let arrays: Punctuated<E, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Zip::new(
                Loc::from(kw.span).join(parens.span.join()),
                arrays.into_iter().collect(),
            ))
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
            } else if MatchExpr::<Self>::peek(input) {
                Self::match_expr(input.parse()?)
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
                Self::ident(ident)
            } else {
                return Err(input.error("expected expression"));
            };

            loop {
                if input.peek(Token![^]) {
                    let op_loc = input.span().into();
                    let _power_token: Token![^] = input.parse()?;
                    let power: LitInt = input.parse()?;
                    let right = expr.clone();
                    for _ in 0..power.base10_parse::<u64>().unwrap() - 1 {
                        let op = BOp::Mul;
                        expr = Self::BinOp(BinOp::new(op, op_loc, expr, right.clone()));
                    }
                } else if Sort::<Self>::peek(input) {
                    expr = Self::sort(Sort::parse(expr, input)?);
                } else if Map::<Self>::peek(input) {
                    expr = Self::map(Map::parse(expr, input)?)
                } else if Fold::<Self>::peek(input) {
                    expr = Self::fold(Fold::parse(expr, input)?)
                } else if TupleElementAccess::<Self>::peek(input) {
                    expr = Self::tuple_access(TupleElementAccess::parse(expr, input)?)
                } else if ArrayAccess::<Self>::peek(input) {
                    expr = Self::array_access(ArrayAccess::parse(expr, input)?)
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
                    expr = Expr::binop(BinOp::parse_term(expr, input)?);
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
                    expr = Expr::BinOp(BinOp::parse_prec1(expr, input)?);
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
                    expr = Expr::binop(BinOp::parse_prec2(expr, input)?);
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
                    expr = Expr::binop(BinOp::parse_prec3(expr, input)?);
                } else {
                    break;
                }
            }
            Ok(expr)
        }
    }
    impl Parse for Expr {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let expr = if Lambda::<Self>::peek(input) {
                Self::typed_lambda(input.parse()?)
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

    impl Parse for MatchEq {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let match_token = input.parse()?;
            let expr = input.parse()?;
            let content;
            let brace = braced!(content in input);
            let arms: Punctuated<Arm, Option<Token![,]>> = Punctuated::parse_terminated(&content)?;

            Ok(MatchEq::new(match_token, expr, brace, arms))
        }
    }

    impl Parse for Eq {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(Token![match]) {
                Ok(Eq::match_eq(input.parse()?))
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
                    let pattern = expr::Pattern::ident(event.clone());
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

    impl InitArmWhen {
        pub fn peek(input: ParseStream) -> bool {
            input.peek(keyword::init)
        }
    }
    impl Parse for InitArmWhen {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let init_token = input.parse()?;
            let arrow_token = input.parse()?;
            let content;
            let brace = braced!(content in input);
            let equations = {
                let mut equations = Vec::new();
                while !content.is_empty() {
                    equations.push(content.parse()?);
                }
                equations
            };
            Ok(InitArmWhen::new(init_token, arrow_token, brace, equations))
        }
    }

    impl Parse for WhenEq {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let when_token = input.parse()?;
            let content;
            let brace = braced!(content in input);
            let init = {
                if InitArmWhen::peek(&content) {
                    let init = content.parse()?;
                    Some(init)
                } else {
                    None
                }
            };
            if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
            }
            let arms: Punctuated<EventArmWhen, Option<Token![,]>> =
                Punctuated::parse_terminated(&content)?;
            Ok(WhenEq::new(when_token, brace, init, arms))
        }
    }

    impl Parse for InitSignal {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let init_token: keyword::init = input.parse()?;
            let pattern: stmt::Pattern = input.parse()?;
            let eq_token: token::Eq = input.parse()?;
            let expr: stream::Expr = input.parse()?;
            let semi_token: token::Semi = input.parse()?;
            Ok(InitSignal::new(
                init_token, pattern, eq_token, expr, semi_token,
            ))
        }
    }

    impl Parse for ReactEq {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            if input.peek(Token![match]) {
                Ok(ReactEq::match_eq(input.parse()?))
            } else if input.peek(keyword::when) {
                Ok(ReactEq::when_eq(input.parse()?))
            } else if input.peek(Token![let]) {
                Ok(ReactEq::local_def(input.parse()?))
            } else if input.peek(keyword::init) {
                Ok(ReactEq::init(input.parse()?))
            } else if input.peek(keyword::log) {
                Ok(ReactEq::log(input.parse()?))
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
                ident,
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
                pattern,
                eq_token,
                event,
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
            Ok(Enumeration::new(ident_enum, ident_elem))
        }
    }

    impl Unary {
        fn peek(input: ParseStream) -> bool {
            UOp::peek(input)
        }
    }
    impl Parse for Unary {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let op_loc = input.span();
            let op: UOp = input.parse()?;
            let term: Term = Term::parse_term(input)?;
            Ok(Unary::new(op_loc, op, term))
        }
    }

    impl Binary {
        fn parse_term(left: Term, input: ParseStream) -> syn::Res<Self> {
            let op_loc = input.span();
            let op = input.parse()?;
            let right = Term::parse_term(input)?;
            Ok(Binary::new(op_loc, left, op, right))
        }
        fn parse_prec1(left: Term, input: ParseStream) -> syn::Res<Self> {
            let op_loc = input.span();
            let op = input.parse()?;
            let right = Term::parse_prec1(input)?;
            Ok(Binary::new(op_loc, left, op, right))
        }
        fn parse_prec2(left: Term, input: ParseStream) -> syn::Res<Self> {
            let op_loc = input.span();
            let op = input.parse()?;
            let right = Term::parse_prec2(input)?;
            Ok(Binary::new(op_loc, left, op, right))
        }
        fn parse_prec3(left: Term, input: ParseStream) -> syn::Res<Self> {
            let op_loc = input.span();
            let op = input.parse()?;
            let right = Term::parse_prec3(input)?;
            Ok(Binary::new(op_loc, left, op, right))
        }
    }
    impl Parse for Binary {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let left = input.parse()?;
            let op_loc = input.span();
            let op = input.parse()?;
            let right = input.parse()?;
            Ok(Binary::new(op_loc, left, op, right))
        }
    }

    impl Application {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Paren)
        }
        fn parse(function: Ident, input: ParseStream) -> syn::Result<Self> {
            let content;
            let parens = parenthesized!(content in input);
            let inputs: Punctuated<Term, Token![,]> = Punctuated::parse_terminated(&content)?;
            Ok(Application::new(
                function.loc().join(parens.span.join()),
                function,
                inputs.into_iter().collect(),
            ))
        }
    }

    impl ParsePrec for Term {
        fn parse_term(input: ParseStream) -> syn::Res<Self> {
            let mut term = if input.peek(token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                Term::Paren(Box::new(content.parse()?))
            } else if input.peek(keyword::result) {
                Term::result(input.parse()?)
            } else if input.peek(keyword::last) {
                let _: keyword::last = input.parse()?;
                let ident: Ident = input.parse()?;
                Term::last(ident)
            } else if input.fork().call(Constant::parse).is_ok() {
                Term::constant(input.parse()?)
            } else if Enumeration::peek(input) {
                Term::enumeration(input.parse()?)
            } else if Unary::peek(input) {
                Term::unary(input.parse()?)
            } else if input.fork().call(Ident::parse).is_ok() {
                let ident: Ident = input.parse()?;
                if Application::peek(input) {
                    Term::app(Application::parse(ident, input)?)
                } else {
                    Term::ident(ident)
                }
            } else {
                return Err(input.error("expected expression"));
            };

            loop {
                if input.peek(Token![^]) {
                    let op_loc = input.span();
                    let _power_token: Token![^] = input.parse()?;
                    let power: LitInt = input.parse()?;
                    let right = term.clone();
                    for _ in 0..power.base10_parse::<u64>().unwrap() - 1 {
                        let op = BOp::Mul;
                        term = Term::binary(Binary::new(op_loc, term, op, right.clone()));
                    }
                } else {
                    break;
                }
            }
            Ok(term)
        }

        fn parse_prec1(input: ParseStream) -> syn::Res<Self> {
            let mut term = Term::parse_term(input)?;

            loop {
                if BOp::peek_prec1(input) {
                    term = Term::binary(Binary::parse_term(term, input)?);
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
                    term = Term::binary(Binary::parse_prec1(term, input)?);
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
                    term = Term::binary(Binary::parse_prec2(term, input)?);
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
                    term = Term::binary(Binary::parse_prec3(term, input)?);
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
                let signal t: float = time();
                let event timeout_pedestrian: unit = timeout(pedestrian, 2000);
                brakes = braking_state(pedestrian, timeout_pedestrian, speed_km_h);
            }
        };
    }

    #[test]
    fn component() {
        let _: Component = parse_quote! {
            component counter(res: bool, tick: bool) -> (o: int) {
                init o = 0;
                o = if res then 0 else (last o) + inc;
                let inc: int = if tick then 1 else 0;
            }
        };
    }

    #[test]
    fn const_decl() {
        let _: ConstDecl = parse_quote! {
            const TEST: int = 3;
        };
    }

    #[cfg(test)]
    mod parse_stream {
        prelude! {
            stream::{Expr, ReactExpr, Last, Emit, WhenExpr},
            expr::*,
        }

        #[test]
        fn should_parse_last() {
            let expression: ReactExpr = syn::parse_quote! {last x};
            let control = ReactExpr::expr(Expr::last(Last::new(
                Loc::test_dummy(),
                syn::parse_quote! {x},
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
            let control = ReactExpr::expr(Expr::test_ident("x"));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_application() {
            let expression: ReactExpr = syn::parse_quote! {f(x)};
            let control = ReactExpr::expr(Expr::app(Application::new(
                Loc::test_dummy(),
                Expr::test_ident("f"),
                vec![Expr::test_ident("x")],
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_binop() {
            let expression: ReactExpr = syn::parse_quote! {a+b};
            let control = ReactExpr::expr(Expr::binop(BinOp::new(
                BOp::Add,
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::test_ident("b"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_binop_with_precedence() {
            let expression: ReactExpr = syn::parse_quote! {a+b*c};
            let control = ReactExpr::expr(Expr::binop(BinOp::new(
                BOp::Add,
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::BinOp(BinOp::new(
                    BOp::Mul,
                    Loc::test_dummy(),
                    Expr::test_ident("b"),
                    Expr::test_ident("c"),
                )),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_typed_lambda() {
            let expression: ReactExpr = syn::parse_quote! {|x: int| f(x)};
            let control = ReactExpr::expr(Expr::type_lambda(Lambda::new(
                Loc::test_dummy(),
                vec![(Loc::test_id("x"), Typ::int())],
                expr::Expr::app(Application::new(
                    Loc::test_dummy(),
                    expr::Expr::test_ident("f"),
                    vec![expr::Expr::test_ident("x")],
                )),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_structure() {
            let expression: ReactExpr = syn::parse_quote! {Point {x: 0, y: 1}};
            let control = ReactExpr::expr(Expr::structure(Structure::new(
                Loc::test_dummy(),
                Loc::test_id("Point"),
                vec![
                    (
                        Loc::test_id("x"),
                        Expr::cst(Constant::int(syn::parse_quote! {0})),
                    ),
                    (
                        Loc::test_id("y"),
                        Expr::cst(Constant::int(syn::parse_quote! {1})),
                    ),
                ],
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_tuple() {
            let expression: ReactExpr = syn::parse_quote! {(x, 0)};
            let control = ReactExpr::expr(Expr::tuple(Tuple::new(
                Loc::test_dummy(),
                vec![
                    Expr::test_ident("x"),
                    Expr::cst(Constant::int(syn::parse_quote! {0})),
                ],
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_enumeration() {
            let expression: ReactExpr = syn::parse_quote! {Color::Pink};
            let control = ReactExpr::expr(Expr::enumeration(Enumeration::new(
                Loc::test_dummy(),
                Loc::test_id("Color"),
                Loc::test_id("Pink"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_array() {
            let expression: ReactExpr = syn::parse_quote! {[1, 2, 3]};
            let control = ReactExpr::expr(Expr::array(Array::new(
                Loc::test_dummy(),
                vec![
                    Expr::cst(Constant::int(syn::parse_quote! {1})),
                    Expr::cst(Constant::int(syn::parse_quote! {2})),
                    Expr::cst(Constant::int(syn::parse_quote! {3})),
                ],
            )));
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
            let control = ReactExpr::expr(Expr::match_expr(MatchExpr::new(
                Loc::test_dummy(),
                Expr::test_ident("a"),
                vec![
                    Arm::new(
                        Pattern::Structure(PatStructure::new(
                            Default::default(),
                            Loc::test_id("Point"),
                            vec![
                                (
                                    Loc::test_id("x"),
                                    Some(Pattern::Constant(Constant::int(syn::parse_quote! {0}))),
                                ),
                                (Loc::test_id("y"), Some(Pattern::Default(Loc::test_dummy()))),
                            ],
                            None,
                        )),
                        Expr::cst(Constant::int(syn::parse_quote! {0})),
                    ),
                    Arm {
                        pattern: Pattern::Structure(PatStructure::new(
                            Default::default(),
                            Loc::test_id("Point"),
                            vec![
                                (Loc::test_id("x"), Some(Pattern::test_ident("x"))),
                                (Loc::test_id("y"), Some(Pattern::Default(Loc::test_dummy()))),
                            ],
                            None,
                        )),
                        guard: Some(Expr::app(Application::new(
                            Loc::test_dummy(),
                            Expr::test_ident("f"),
                            vec![Expr::test_ident("x")],
                        ))),
                        expr: Expr::cst(Constant::int(syn::parse_quote! {-1})),
                    },
                    Arm::new(
                        Pattern::Default(Loc::test_dummy()),
                        Expr::cst(Constant::int(syn::parse_quote! {1})),
                    ),
                ],
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_field_access() {
            let expression: ReactExpr = syn::parse_quote! {p.x};
            let control = ReactExpr::expr(Expr::field_access(FieldAccess::new(
                Loc::test_dummy(),
                Expr::test_ident("p"),
                Loc::test_id("x"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_tuple_element_access() {
            let expression: ReactExpr = syn::parse_quote! {t.0};
            let control = ReactExpr::expr(Expr::tuple_access(TupleElementAccess::new(
                Loc::test_dummy(),
                Expr::test_ident("t"),
                0,
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_map() {
            let expression: ReactExpr = syn::parse_quote! {a.map(f)};
            let control = ReactExpr::expr(Expr::map(Map::new(
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::test_ident("f"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_fold() {
            let expression: ReactExpr = syn::parse_quote! {a.fold(0, sum)};
            let control = ReactExpr::expr(Expr::fold(Fold::new(
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::cst(Constant::int(syn::parse_quote! {0})),
                Expr::test_ident("sum"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_sort() {
            let expression: ReactExpr = syn::parse_quote! {a.sort(order)};
            let control = ReactExpr::expr(Expr::sort(Sort::new(
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::test_ident("order"),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_zip() {
            let expression: ReactExpr = syn::parse_quote! {zip(a, b, c)};
            let control = ReactExpr::expr(Expr::zip(Zip::new(
                Loc::test_dummy(),
                vec![
                    Expr::test_ident("a"),
                    Expr::test_ident("b"),
                    Expr::test_ident("c"),
                ],
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_emit() {
            let expression: ReactExpr = syn::parse_quote! {emit 0};
            let control = ReactExpr::expr(Expr::emit(Emit::new(
                Loc::test_dummy(),
                Default::default(),
                Expr::cst(Constant::int(syn::parse_quote! {0})),
            )));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_when() {
            let expression: ReactExpr = syn::parse_quote! { when {let d = p? => emit x} };
            let control = ReactExpr::when_expr(WhenExpr::new(
                Default::default(),
                None,
                vec![stream::EventArmWhen::new(
                    equation::EventPattern::Let(equation::LetEventPattern::new(
                        Default::default(),
                        expr::Pattern::test_ident("d"),
                        Default::default(),
                        format_ident!("p"),
                        Default::default(),
                    )),
                    None,
                    Expr::emit(Emit::new(
                        Loc::test_dummy(),
                        Default::default(),
                        Expr::test_ident("x"),
                    )),
                )],
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_when_with_guard() {
            let expression: ReactExpr = syn::parse_quote! { when {p? if p > 0 => emit x} };
            let control = ReactExpr::when_expr(WhenExpr::new(
                Default::default(),
                None,
                vec![stream::EventArmWhen::new(
                    equation::EventPattern::Let(equation::LetEventPattern::new(
                        Default::default(),
                        expr::Pattern::test_ident("p"),
                        Default::default(),
                        format_ident!("p"),
                        Default::default(),
                    )),
                    Some(Box::new(Expr::binop(BinOp::new(
                        BOp::Gt,
                        Loc::test_dummy(),
                        Expr::test_ident("p"),
                        Expr::cst(Constant::Integer(syn::parse_quote! {0})),
                    )))),
                    Expr::emit(Emit::new(
                        Loc::test_dummy(),
                        Default::default(),
                        Expr::test_ident("x"),
                    )),
                )],
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_when_with_init() {
            let expression: ReactExpr = syn::parse_quote! {
                when {
                    init        => 0,
                    p? if p > 0 => p
                }
            };
            let control = ReactExpr::when_expr(WhenExpr::new(
                Default::default(),
                Some(stream::InitArmWhen::new(
                    Default::default(),
                    Default::default(),
                    Expr::cst(Constant::Integer(syn::parse_quote! {0})),
                )),
                vec![stream::EventArmWhen::new(
                    equation::EventPattern::Let(equation::LetEventPattern::new(
                        Default::default(),
                        expr::Pattern::test_ident("p"),
                        Default::default(),
                        format_ident!("p"),
                        Default::default(),
                    )),
                    Some(Box::new(Expr::binop(BinOp::new(
                        BOp::Gt,
                        Loc::test_dummy(),
                        Expr::test_ident("p"),
                        Expr::cst(Constant::Integer(syn::parse_quote! {0})),
                    )))),
                    Expr::ident("p"),
                )],
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
            let control = Pattern::test_ident("x");
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
                Default::default(),
                Loc::test_id("Point"),
                vec![
                    (
                        Loc::test_id("x"),
                        Some(Pattern::cst(Constant::int(parse_quote! {0}))),
                    ),
                    (Loc::test_id("y"), Some(Pattern::default(Loc::test_dummy()))),
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
                Default::default(),
                Loc::test_id("Point"),
                vec![
                    (
                        Loc::test_id("x"),
                        Some(Pattern::cst(Constant::int(parse_quote! {0}))),
                    ),
                    (Loc::test_id("y"), None),
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
                Default::default(),
                Loc::test_id("Point"),
                vec![(
                    Loc::test_id("x"),
                    Some(Pattern::cst(Constant::int(parse_quote! {0}))),
                )],
                Some(parse_quote!(..)),
            ));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_tuple_pat() {
            let pattern: Pattern = parse_quote! {(x, 0)};
            let control = Pattern::tuple(PatTuple::new(
                Default::default(),
                vec![
                    Pattern::test_ident("x"),
                    Pattern::cst(Constant::int(parse_quote! {0})),
                ],
            ));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_enumeration_pat() {
            let pattern: Pattern = parse_quote! {Color::Pink};
            let control = Pattern::enumeration(PatEnumeration::new(
                Loc::test_id("Color"),
                Loc::test_id("Pink"),
            ));
            assert_eq!(pattern, control)
        }

        #[test]
        fn parse_default_pat() {
            let pattern: Pattern = parse_quote! {_};
            let control = Pattern::default(Loc::test_dummy());
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
            let control = Expr::test_ident("x");
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_application() {
            let expr: Expr = parse_quote! {f(x)};
            let control = Expr::app(Application::new(
                Loc::test_dummy(),
                Expr::test_ident("f"),
                vec![Expr::test_ident("x")],
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_binop() {
            let expr: Expr = parse_quote! {a+b};
            let control = Expr::binop(BinOp::new(
                BOp::Add,
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::test_ident("b"),
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_binop_with_precedence() {
            let expr: Expr = parse_quote! {a+b*c};
            let control = Expr::binop(BinOp::new(
                BOp::Add,
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::binop(BinOp::new(
                    BOp::Mul,
                    Loc::test_dummy(),
                    Expr::test_ident("b"),
                    Expr::test_ident("c"),
                )),
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_binop_with_unop() {
            let term: Expr = parse_quote! {-x + 1};
            let control = Expr::binop(BinOp::new(
                BOp::Add,
                Loc::test_dummy(),
                Expr::unop(UnOp::new(
                    UOp::Neg,
                    Loc::test_dummy(),
                    Expr::test_ident("x"),
                )),
                Expr::constant(Constant::int(parse_quote! {1})),
            ));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_typed_lambda() {
            let expr: Expr = parse_quote! {|x: int| f(x)};
            let control = Expr::typed_lambda(Lambda::new(
                Loc::test_dummy(),
                vec![(Loc::test_id("x"), Typ::int())],
                Expr::app(Application::new(
                    Loc::test_dummy(),
                    Expr::test_ident("f"),
                    vec![Expr::test_ident("x")],
                )),
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_structure() {
            let expr: Expr = parse_quote! {Point {x: 0, y: 1}};
            let control = Expr::structure(Structure::new(
                Loc::test_dummy(),
                Loc::test_id("Point"),
                vec![
                    (
                        Loc::test_id("x"),
                        Expr::cst(Constant::int(parse_quote! {0})),
                    ),
                    (
                        Loc::test_id("y"),
                        Expr::cst(Constant::int(parse_quote! {1})),
                    ),
                ],
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_tuple() {
            let expr: Expr = parse_quote! {(x, 0)};
            let control = Expr::tuple(Tuple::new(
                Loc::test_dummy(),
                vec![
                    Expr::test_ident("x"),
                    Expr::cst(Constant::int(parse_quote! {0})),
                ],
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_enumeration() {
            let expr: Expr = parse_quote! {Color::Pink};
            let control = Expr::enumeration(Enumeration::new(
                Loc::test_dummy(),
                Loc::test_id("Color"),
                Loc::test_id("Pink"),
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_array() {
            let expr: Expr = parse_quote! {[1, 2, 3]};
            let control = Expr::array(Array::new(
                Loc::test_dummy(),
                vec![
                    Expr::cst(Constant::int(parse_quote! {1})),
                    Expr::cst(Constant::int(parse_quote! {2})),
                    Expr::cst(Constant::int(parse_quote! {3})),
                ],
            ));
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
            let control = Expr::match_expr(MatchExpr::new(
                Loc::test_dummy(),
                Expr::test_ident("a"),
                vec![
                    Arm::new(
                        Pattern::structure(PatStructure::new(
                            Default::default(),
                            Loc::test_id("Point"),
                            vec![
                                (
                                    Loc::test_id("x"),
                                    Some(Pattern::cst(Constant::int(parse_quote! {0}))),
                                ),
                                (Loc::test_id("y"), Some(Pattern::default(Loc::test_dummy()))),
                            ],
                            None,
                        )),
                        Expr::Constant(Constant::Integer(parse_quote! {0})),
                    ),
                    Arm::new_with_guard(
                        Pattern::Structure(PatStructure::new(
                            Default::default(),
                            Loc::test_id("Point"),
                            vec![
                                (Loc::test_id("x"), Some(Pattern::test_ident("x"))),
                                (Loc::test_id("y"), Some(Pattern::default(Loc::test_dummy()))),
                            ],
                            None,
                        )),
                        Expr::cst(Constant::int(parse_quote! {-1})),
                        Some(Expr::app(Application::new(
                            Loc::test_dummy(),
                            Expr::test_ident("f"),
                            vec![Expr::test_ident("x")],
                        ))),
                    ),
                    Arm::new(
                        Pattern::Default(Loc::test_dummy()),
                        Expr::cst(Constant::int(parse_quote! {1})),
                    ),
                ],
            ));
            assert_eq!(expr, control)
        }

        #[test]
        fn should_parse_field_access() {
            let expression: Expr = parse_quote! {p.x};
            let control = Expr::field_access(FieldAccess::new(
                Loc::test_dummy(),
                Expr::test_ident("p"),
                Loc::test_id("x"),
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_tuple_element_access() {
            let expression: Expr = parse_quote! {t.0};
            let control = Expr::tuple_access(TupleElementAccess::new(
                Loc::test_dummy(),
                Expr::test_ident("t"),
                0,
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_map() {
            let expression: Expr = parse_quote! {a.map(f)};
            let control = Expr::map(Map::new(
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::test_ident("f"),
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_fold() {
            let expression: Expr = parse_quote! {a.fold(0, sum)};
            let control = Expr::fold(Fold::new(
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::cst(Constant::int(parse_quote! {0})),
                Expr::test_ident("sum"),
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_sort() {
            let expression: Expr = parse_quote! {a.sort(order)};
            let control = Expr::sort(Sort::new(
                Loc::test_dummy(),
                Expr::test_ident("a"),
                Expr::test_ident("order"),
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_zip() {
            let expression: Expr = parse_quote! {zip(a, b, c)};
            let control = Expr::zip(Zip::new(
                Loc::test_dummy(),
                vec![
                    Expr::test_ident("a"),
                    Expr::test_ident("b"),
                    Expr::test_ident("c"),
                ],
            ));
            assert_eq!(expression, control)
        }
    }

    mod equations {
        use super::*;
        use equation::*;

        #[test]
        fn should_parse_output_definition() {
            let equation: ReactEq = parse_quote! {o = if res then 0 else (last o) + inc;};
            let control = ReactEq::out_def(Instantiation {
                pattern: parse_quote! {o},
                eq_token: parse_quote! {=},
                expr: stream::ReactExpr::expr(stream::Expr::ite(expr::IfThenElse::new(
                    Loc::test_dummy(),
                    stream::Expr::test_ident("res"),
                    stream::Expr::cst(Constant::int(parse_quote! {0})),
                    stream::Expr::binop(expr::BinOp::new(
                        BOp::Add,
                        Loc::test_dummy(),
                        stream::Expr::last(stream::Last::new(Loc::test_dummy(), parse_quote! {o})),
                        stream::Expr::test_ident("inc"),
                    )),
                ))),
                semi_token: parse_quote! {;},
            });
            assert_eq!(equation, control)
        }

        #[test]
        fn should_parse_log() {
            let equation: ReactEq = parse_quote! {log (a, b);};
            let control = ReactEq::log(LogStmt {
                log_token: parse_quote! {log},
                pattern: parse_quote! {(a, b)},
                semi_token: parse_quote! {;},
            });
            assert_eq!(equation, control)
        }

        #[test]
        fn should_parse_when_equation() {
            let expression: ReactEq = syn::parse_quote! {
                when {
                    e1? if e1 > 0 => {
                        y = emit e1;
                    }
                    e2? => {
                        y = emit e2;
                    },
                    e3? => {
                        y = emit e3;
                    }
                }
            };
            let mut arms = Punctuated::new();
            arms.push_value(equation::EventArmWhen::new(
                equation::EventPattern::Let(equation::LetEventPattern::new(
                    Default::default(),
                    expr::Pattern::test_ident("e1"),
                    Default::default(),
                    format_ident!("e1"),
                    Default::default(),
                )),
                Some((
                    Default::default(),
                    stream::Expr::binop(expr::BinOp::new(
                        BOp::Gt,
                        Loc::test_dummy(),
                        stream::Expr::test_ident("e1"),
                        stream::Expr::cst(Constant::Integer(syn::parse_quote! {0})),
                    )),
                )),
                Default::default(),
                Default::default(),
                vec![Eq::OutputDef(Instantiation::new(
                    stmt::Pattern::Identifier(format_ident!("y")),
                    Default::default(),
                    stream::Expr::emit(stream::Emit::new(
                        Loc::test_dummy(),
                        Default::default(),
                        stream::Expr::test_ident("e1"),
                    )),
                    Default::default(),
                ))],
            ));
            arms.push_punct(None);
            arms.push_value(equation::EventArmWhen::new(
                equation::EventPattern::Let(equation::LetEventPattern::new(
                    Default::default(),
                    expr::Pattern::test_ident("e2"),
                    Default::default(),
                    format_ident!("e2"),
                    Default::default(),
                )),
                None,
                Default::default(),
                Default::default(),
                vec![Eq::OutputDef(Instantiation::new(
                    stmt::Pattern::Identifier(format_ident!("y")),
                    Default::default(),
                    stream::Expr::emit(stream::Emit::new(
                        Loc::test_dummy(),
                        Default::default(),
                        stream::Expr::test_ident("e2"),
                    )),
                    Default::default(),
                ))],
            ));
            arms.push_punct(Some(Default::default()));
            arms.push_value(equation::EventArmWhen::new(
                equation::EventPattern::Let(equation::LetEventPattern::new(
                    Default::default(),
                    expr::Pattern::test_ident("e3"),
                    Default::default(),
                    format_ident!("e3"),
                    Default::default(),
                )),
                None,
                Default::default(),
                Default::default(),
                vec![Eq::OutputDef(Instantiation::new(
                    stmt::Pattern::Identifier(format_ident!("y")),
                    Default::default(),
                    stream::Expr::emit(stream::Emit::new(
                        Loc::test_dummy(),
                        Default::default(),
                        stream::Expr::test_ident("e3"),
                    )),
                    Default::default(),
                ))],
            ));
            let control = ReactEq::when_eq(WhenEq::new(
                Default::default(),
                Default::default(),
                None,
                arms,
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_match_equation() {
            let expression: ReactEq = syn::parse_quote! {
                match e {
                    x if x > 0 => {
                        y = x;
                    }
                    x if x < 0 => {
                        y = -x;
                    },
                    x => {
                        y = x;
                    }
                }
            };
            let mut arms = Punctuated::new();
            arms.push_value(equation::Arm::new(
                expr::Pattern::Identifier(format_ident!("x")),
                Some((
                    Default::default(),
                    stream::Expr::binop(expr::BinOp::new(
                        BOp::Gt,
                        Loc::test_dummy(),
                        stream::Expr::test_ident("x"),
                        stream::Expr::cst(Constant::Integer(syn::parse_quote! {0})),
                    )),
                )),
                Default::default(),
                Default::default(),
                vec![Eq::OutputDef(Instantiation::new(
                    stmt::Pattern::Identifier(format_ident!("y")),
                    Default::default(),
                    stream::Expr::test_ident("x"),
                    Default::default(),
                ))],
            ));
            arms.push_punct(None);
            arms.push_value(equation::Arm::new(
                expr::Pattern::Identifier(format_ident!("x")),
                Some((
                    Default::default(),
                    stream::Expr::binop(expr::BinOp::new(
                        BOp::Lt,
                        Loc::test_dummy(),
                        stream::Expr::test_ident("x"),
                        stream::Expr::cst(Constant::Integer(syn::parse_quote! {0})),
                    )),
                )),
                Default::default(),
                Default::default(),
                vec![Eq::OutputDef(Instantiation::new(
                    stmt::Pattern::Identifier(format_ident!("y")),
                    Default::default(),
                    stream::Expr::unop(expr::UnOp::new(
                        UOp::Neg,
                        Loc::test_dummy(),
                        stream::Expr::test_ident("x"),
                    )),
                    Default::default(),
                ))],
            ));
            arms.push_punct(Some(Default::default()));
            arms.push_value(equation::Arm::new(
                expr::Pattern::Identifier(format_ident!("x")),
                None,
                Default::default(),
                Default::default(),
                vec![Eq::OutputDef(Instantiation::new(
                    stmt::Pattern::Identifier(format_ident!("y")),
                    Default::default(),
                    stream::Expr::test_ident("x"),
                    Default::default(),
                ))],
            ));
            let control = ReactEq::match_eq(MatchEq::new(
                Default::default(),
                stream::Expr::test_ident("e"),
                Default::default(),
                arms,
            ));
            assert_eq!(expression, control)
        }

        #[test]
        fn should_parse_tuple_instantiation() {
            let equation: ReactEq = parse_quote! {
                (o1, o2) = if res then (0, 0) else ((last o1) + inc1, last o2 + inc2);
            };
            let control = ReactEq::out_def(Instantiation {
                pattern: stmt::Pattern::tuple(stmt::Tuple::new(
                    Loc::test_dummy(),
                    vec![parse_quote! {o1}, parse_quote! {o2}],
                )),
                eq_token: parse_quote! {=},
                expr: stream::ReactExpr::expr(stream::Expr::ite(expr::IfThenElse::new(
                    Loc::test_dummy(),
                    stream::Expr::test_ident("res"),
                    stream::Expr::tuple(expr::Tuple::new(
                        Loc::test_dummy(),
                        vec![
                            stream::Expr::cst(Constant::int(parse_quote! {0})),
                            stream::Expr::cst(Constant::int(parse_quote! {0})),
                        ],
                    )),
                    stream::Expr::tuple(expr::Tuple::new(
                        Loc::test_dummy(),
                        vec![
                            stream::Expr::binop(expr::BinOp::new(
                                BOp::Add,
                                Loc::test_dummy(),
                                stream::Expr::last(stream::Last::new(
                                    Loc::test_dummy(),
                                    parse_quote! {o1},
                                )),
                                stream::Expr::test_ident("inc1"),
                            )),
                            stream::Expr::binop(expr::BinOp::new(
                                BOp::Add,
                                Loc::test_dummy(),
                                stream::Expr::last(stream::Last::new(
                                    Loc::test_dummy(),
                                    parse_quote! {o2},
                                )),
                                stream::Expr::test_ident("inc2"),
                            )),
                        ],
                    )),
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
                    Loc::test_dummy(),
                    stream::Expr::test_ident("res"),
                    stream::Expr::cst(Constant::int(parse_quote! {0})),
                    stream::Expr::binop(expr::BinOp::new(
                        BOp::Add,
                        Loc::test_dummy(),
                        stream::Expr::last(stream::Last::new(Loc::test_dummy(), parse_quote! {o})),
                        stream::Expr::test_ident("inc"),
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
                    if res then (0, 0) else ((last o1) + inc1, last o2 + inc2);
            };
            let control = ReactEq::local_def(stmt::LetDecl::new(
                parse_quote!(let),
                stmt::Pattern::tuple(stmt::Tuple::new(
                    Loc::test_dummy(),
                    vec![
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
                    ],
                )),
                parse_quote!(=),
                stream::ReactExpr::expr(stream::Expr::ite(expr::IfThenElse::new(
                    Loc::test_dummy(),
                    stream::Expr::test_ident("res"),
                    stream::Expr::tuple(expr::Tuple::new(
                        Loc::test_dummy(),
                        vec![
                            stream::Expr::cst(Constant::int(parse_quote! {0})),
                            stream::Expr::cst(Constant::int(parse_quote! {0})),
                        ],
                    )),
                    stream::Expr::tuple(expr::Tuple::new(
                        Loc::test_dummy(),
                        vec![
                            stream::Expr::binop(expr::BinOp::new(
                                BOp::Add,
                                Loc::test_dummy(),
                                stream::Expr::last(stream::Last::new(
                                    Loc::test_dummy(),
                                    parse_quote! {o1},
                                )),
                                stream::Expr::test_ident("inc1"),
                            )),
                            stream::Expr::binop(expr::BinOp::new(
                                BOp::Add,
                                Loc::test_dummy(),
                                stream::Expr::last(stream::Last::new(
                                    Loc::test_dummy(),
                                    parse_quote! {o2},
                                )),
                                stream::Expr::test_ident("inc2"),
                            )),
                        ],
                    )),
                ))),
                parse_quote! {;},
            ));
            assert_eq!(equation, control)
        }

        #[test]
        fn should_parse_initialization() {
            let equation: ReactEq = parse_quote! {
                init (o1: int, o2: int) = (0, 0);
            };
            let control = ReactEq::init(equation::InitSignal::new(
                parse_quote!(init),
                stmt::Pattern::tuple(stmt::Tuple::new(
                    Loc::test_dummy(),
                    vec![
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
                    ],
                )),
                parse_quote!(=),
                stream::Expr::tuple(expr::Tuple::new(
                    Loc::test_dummy(),
                    vec![
                        stream::Expr::cst(Constant::int(parse_quote! {0})),
                        stream::Expr::cst(Constant::int(parse_quote! {0})),
                    ],
                )),
                parse_quote! {;},
            ));
            assert_eq!(equation, control);
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
            let control = Term::test_ident("x");
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_unary_operation() {
            let term: Term = parse_quote! {!x};
            let control = Term::unary(Unary::new(
                Loc::test_dummy(),
                UOp::Not,
                Term::test_ident("x"),
            ));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_binary_operation() {
            let term: Term = parse_quote! {-x + 1};
            let control = Term::binary(Binary::new(
                Loc::test_dummy(),
                Term::unary(Unary::new(
                    Loc::test_dummy(),
                    UOp::Neg,
                    Term::test_ident("x"),
                )),
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
                    Loc::test_dummy(),
                    Term::unary(Unary::new(
                        Loc::test_dummy(),
                        UOp::Not,
                        Term::test_ident("x"),
                    )),
                    BOp::And,
                    Term::test_ident("y"),
                )),
                Default::default(),
                Term::test_ident("z"),
            ));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_event_implication() {
            let term: Term = parse_quote! { when d = p? => d > x+y};
            let control = Term::event(EventImplication::new(
                Default::default(),
                Loc::test_id("d"),
                Default::default(),
                Loc::test_id("p"),
                Default::default(),
                Default::default(),
                Term::binary(Binary::new(
                    Loc::test_dummy(),
                    Term::test_ident("d"),
                    BOp::Gt,
                    Term::binary(Binary::new(
                        Loc::test_dummy(),
                        Term::test_ident("x"),
                        BOp::Add,
                        Term::test_ident("y"),
                    )),
                )),
            ));
            assert_eq!(term, control)
        }

        #[test]
        fn should_parse_forall() {
            let term: Term = parse_quote! { forall d: int, d > x+y};
            let control = Term::forall(ForAll::new(
                Default::default(),
                Loc::test_id("d"),
                Default::default(),
                Typ::int(),
                Default::default(),
                Term::binary(Binary::new(
                    Loc::test_dummy(),
                    Term::test_ident("d"),
                    BOp::Gt,
                    Term::binary(Binary::new(
                        Loc::test_dummy(),
                        Term::test_ident("x"),
                        BOp::Add,
                        Term::test_ident("y"),
                    )),
                )),
            ));
            assert_eq!(term, control)
        }
    }
}
