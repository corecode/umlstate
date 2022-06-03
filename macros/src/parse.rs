use quote::ToTokens;
use syn::parse::Parse;
use syn::{Error, Result, Token};

mod kw {
    syn::custom_keyword!(machine);
    syn::custom_keyword!(state);
    syn::custom_keyword!(region);
    syn::custom_keyword!(ctx);
}

#[derive(Clone)]
pub struct UmlState {
    pub items: Vec<Machine>,
}

#[derive(Clone)]
pub struct Machine {
    pub vis: syn::Visibility,
    pub machine_token: kw::machine,
    pub ident: syn::Ident,
    pub brace_token: syn::token::Brace,
    pub items: Vec<MachineItem>,
}

#[derive(Clone)]
pub struct State {
    pub state_token: kw::state,
    pub ident: syn::Ident,
    pub brace_token: Option<syn::token::Brace>,
    pub items: Vec<StateItem>,
    pub semi_token: Option<Token![;]>,
}

#[derive(Clone)]
pub enum MachineItem {
    Context(ItemContext),
    StateItem(StateItem),
}

#[derive(Clone)]
pub enum StateItem {
    State(Box<State>),
    Region(Box<Region>),
    Transition(ItemTransition),
}

#[derive(Clone)]
pub struct ItemContext {
    pub ctx_token: kw::ctx,
    pub ident: syn::Ident,
    pub semi_token: Token![;],
}

#[derive(Clone)]
pub struct Region {
    pub region_token: kw::region,
    pub ident: syn::Ident,
    pub brace_token: syn::token::Brace,
    pub items: Vec<StateItem>,
}

#[derive(Clone)]
pub struct ItemTransition {
    pub source: TransitionSource,
    pub event: Option<(Token![+], Event)>,
    pub target: Option<(Token![=>], syn::Ident)>,
    pub action: Option<(Token![/], Action)>,
    pub guard: Option<(Token![if], Guard)>,
    pub semi_token: Token![;],
}

#[derive(Clone)]
pub enum TransitionSource {
    Initial(SourceInitial),
    State(syn::Pat),
}

#[derive(Clone)]
pub struct SourceInitial {
    pub lt_token: Token![<],
    pub asterisk_token: Token![*],
    pub gt_token: Token![>],
}

#[derive(Clone)]
pub struct Event {
    pub pat: syn::Pat,
}

#[derive(Clone)]
pub struct Action {
    pub expr: Box<syn::Expr>,
}

#[derive(Clone)]
pub struct Guard {
    pub expr: Box<syn::Expr>,
}

impl Parse for UmlState {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(UmlState {
            items: {
                let mut items = Vec::new();
                while !input.is_empty() {
                    items.push(input.parse()?)
                }
                items
            },
        })
    }
}

impl ToTokens for UmlState {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for item in self.items.iter() {
            item.to_tokens(tokens);
        }
    }
}

impl Parse for Machine {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        Ok(Machine {
            vis: input.parse()?,
            machine_token: input.parse()?,
            ident: input.parse()?,
            brace_token: syn::braced!(content in input),
            items: {
                let mut items = Vec::new();
                while !content.is_empty() {
                    items.push(content.parse()?)
                }
                items
            },
        })
    }
}

impl ToTokens for Machine {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.vis.to_tokens(tokens);
        self.machine_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            for item in self.items.iter() {
                item.to_tokens(tokens);
            }
        })
    }
}

impl Parse for State {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        let state_token = input.parse()?;
        let ident = input.parse()?;
        let semi_token;
        let brace_token;
        let mut items = vec![];
        if input.peek(Token![;]) {
            semi_token = input.parse()?;
            brace_token = None;
        } else {
            semi_token = None;
            brace_token = Some(syn::braced!(content in input));
            while !content.is_empty() {
                items.push(content.parse()?)
            }
        }

        Ok(State {
            state_token,
            ident,
            brace_token,
            items,
            semi_token,
        })
    }
}

impl ToTokens for State {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.state_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        if let Some(b) = self.brace_token {
            b.surround(tokens, |tokens| {
                for item in self.items.iter() {
                    item.to_tokens(tokens);
                }
            })
        }
        self.semi_token.to_tokens(tokens);
    }
}

impl Parse for Region {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;

        Ok(Region {
            region_token: input.parse()?,
            ident: input.parse()?,
            brace_token: syn::braced!(content in input),
            items: {
                let mut items = vec![];
                while !content.is_empty() {
                    items.push(content.parse()?)
                }
                items
            },
        })
    }
}

impl ToTokens for Region {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.region_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            for item in self.items.iter() {
                item.to_tokens(tokens);
            }
        })
    }
}

impl Parse for MachineItem {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        if input.peek(kw::ctx) {
            return Ok(MachineItem::Context(input.parse()?));
        }
        Ok(MachineItem::StateItem(input.parse()?))
    }
}

impl ToTokens for MachineItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            MachineItem::Context(c) => c.to_tokens(tokens),
            MachineItem::StateItem(i) => i.to_tokens(tokens),
        }
    }
}

impl Parse for StateItem {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        if input.peek(kw::state) {
            return Ok(StateItem::State(input.parse()?));
        }
        if input.peek(kw::region) {
            return Ok(StateItem::Region(input.parse()?));
        }
        Ok(StateItem::Transition(input.parse()?))
    }
}

impl ToTokens for StateItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            StateItem::State(s) => s.to_tokens(tokens),
            StateItem::Region(r) => r.to_tokens(tokens),
            StateItem::Transition(t) => t.to_tokens(tokens),
        }
    }
}

impl Parse for ItemContext {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(ItemContext {
            ctx_token: input.parse()?,
            ident: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

impl ToTokens for ItemContext {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ctx_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

impl Parse for ItemTransition {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(ItemTransition {
            source: input.parse()?,
            event: if input.peek(Token![+]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
            target: if input.peek(Token![=>]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
            action: if input.peek(Token![/]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
            guard: if input.peek(Token![if]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
            semi_token: input.parse()?,
        })
    }
}

impl ToTokens for ItemTransition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.source.to_tokens(tokens);
        if let Some((plus, event)) = &self.event {
            plus.to_tokens(tokens);
            event.to_tokens(tokens);
        }
        if let Some((arrow, target)) = &self.target {
            arrow.to_tokens(tokens);
            target.to_tokens(tokens);
        }
        if let Some((slash, action)) = &self.action {
            slash.to_tokens(tokens);
            action.to_tokens(tokens);
        }
        if let Some((if_token, guard)) = &self.guard {
            if_token.to_tokens(tokens);
            guard.to_tokens(tokens);
        }
    }
}

impl Parse for TransitionSource {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        if input.peek(Token![<]) {
            return Ok(TransitionSource::Initial(input.parse()?));
        }
        return Ok(TransitionSource::State(input.parse()?));
    }
}

impl ToTokens for TransitionSource {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TransitionSource::Initial(i) => i.to_tokens(tokens),
            TransitionSource::State(s) => s.to_tokens(tokens),
        }
    }
}

impl Parse for SourceInitial {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(SourceInitial {
            lt_token: input.parse()?,
            asterisk_token: input.parse()?,
            gt_token: input.parse()?,
        })
    }
}

impl ToTokens for SourceInitial {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lt_token.to_tokens(tokens);
        self.asterisk_token.to_tokens(tokens);
        self.gt_token.to_tokens(tokens);
    }
}

impl Parse for Event {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let pat = input.parse()?;
        match &pat {
            syn::Pat::Path(_) | syn::Pat::Struct(_) | syn::Pat::TupleStruct(_) => (),
            syn::Pat::Ident(i)
                if i.by_ref.is_none() && i.mutability.is_none() && i.subpat.is_none() => {}
            _ => return Err(Error::new_spanned(pat, "event must name a type")),
        }
        Ok(Event { pat })
    }
}

impl ToTokens for Event {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.pat.to_tokens(tokens);
    }
}

impl Parse for Action {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let expr: Box<syn::Expr> = input.parse()?;
        match expr.as_ref() {
            syn::Expr::Assign(_)
            | syn::Expr::AssignOp(_)
            | syn::Expr::Block(_)
            | syn::Expr::Call(_)
            | syn::Expr::Group(_)
            | syn::Expr::Macro(_)
            | syn::Expr::MethodCall(_)
            | syn::Expr::Path(_) => (),
            _ => return Err(Error::new_spanned(expr, "expected an action expression")),
        }
        Ok(Action { expr })
    }
}

impl ToTokens for Action {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr.to_tokens(tokens);
    }
}

impl Parse for Guard {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Guard {
            expr: input.parse()?,
        })
    }
}

impl ToTokens for Guard {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn parse_umlstate() {
        let _sm: UmlState = parse_quote! {
            machine Foo {
                state S1;

                <*> => S1;
                S1 + E2(n) => M2 / print2;
                M2 + E1 => S1
                    if some_cond();

                state M2 {
                    state A;
                    state B;

                    <*> => A;
                    A + E1 => B;
                }
            }
        };
    }
}
