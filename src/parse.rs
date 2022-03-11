use quote::ToTokens;
use syn::parse::Parse;
use syn::{Error, Result, Token};

mod kw {
    syn::custom_keyword!(machine);
    syn::custom_keyword!(state);
}

pub struct UmlState {
    pub items: Vec<Machine>,
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

pub struct Machine {
    pub machine_token: kw::machine,
    pub ident: syn::Ident,
    pub brace_token: syn::token::Brace,
    pub items: Vec<MachineItem>,
}

impl Parse for Machine {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let content;
        Ok(Machine {
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

pub enum MachineItem {
    State(ItemState),
    Transition(ItemTransition),
}

pub struct ItemState {
    pub state_token: kw::state,
    pub ident: syn::Ident,
    pub semi_token: Option<Token![;]>,
}

pub struct ItemTransition {
    pub source: syn::Ident,
    pub plus_token: Token![+],
    pub event: syn::Ident,
    pub arrow_token: Token![=>],
    pub target: syn::Ident,
    pub action: Option<(Token![/], Action)>,
    pub guard: Option<(Token![if], Guard)>,
    pub semi_token: Token![;],
}

pub struct Action {
    pub expr: Box<syn::Expr>,
}

pub struct Guard {
    pub expr: Box<syn::Expr>,
}

impl Parse for MachineItem {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        if input.peek(kw::state) {
            return Ok(MachineItem::State(input.parse()?));
        }
        Ok(MachineItem::Transition(input.parse()?))
    }
}

impl Parse for ItemState {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(ItemState {
            state_token: input.parse()?,
            ident: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

impl Parse for ItemTransition {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(ItemTransition {
            source: input.parse()?,
            plus_token: input.parse()?,
            event: input.parse()?,
            arrow_token: input.parse()?,
            target: input.parse()?,
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

impl Parse for Guard {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Guard {
            expr: input.parse()?,
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

impl ToTokens for Machine {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.machine_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            for item in self.items.iter() {
                item.to_tokens(tokens);
            }
        })
    }
}

impl ToTokens for MachineItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            MachineItem::State(s) => s.to_tokens(tokens),
            MachineItem::Transition(t) => t.to_tokens(tokens),
        }
    }
}

impl ToTokens for ItemState {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.state_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

impl ToTokens for ItemTransition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.source.to_tokens(tokens);
        self.plus_token.to_tokens(tokens);
        self.event.to_tokens(tokens);
        self.arrow_token.to_tokens(tokens);
        self.target.to_tokens(tokens);
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

impl ToTokens for Action {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr.to_tokens(tokens);
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
                state S2;

                S1 + E2 => S2 / print2;
                S2 + E1 => S1
                    if some_cond();
            }
        };
    }
}
