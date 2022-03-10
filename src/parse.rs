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
    pub slash_token: Option<Token![/]>,
    pub action: Option<Action>,
    pub semi_token: Token![;],
}

pub struct Action {
    pub expr: syn::Expr,
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
        let source = input.parse()?;
        let plus_token = input.parse()?;
        let event = input.parse()?;
        let arrow_token = input.parse()?;
        let target = input.parse()?;
        let slash_token = input.parse()?;
        let action = match &slash_token {
            Some(_) => Some(input.parse()?),
            _ => None,
        };
        let semi_token = input.parse()?;
        Ok(ItemTransition {
            source,
            plus_token,
            event,
            arrow_token,
            target,
            slash_token,
            action,
            semi_token,
        })
    }
}

impl Parse for Action {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let expr = input.parse()?;
        match expr {
            syn::Expr::Assign(_)
            | syn::Expr::AssignOp(_)
            | syn::Expr::Block(_)
            | syn::Expr::Call(_)
            | syn::Expr::Group(_)
            | syn::Expr::Macro(_)
            | syn::Expr::MethodCall(_)
            | syn::Expr::Path(_) => Ok(Action { expr }),
            _ => Err(Error::new_spanned(expr, "expected an action expression")),
        }
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
                S2 + E1 => S1;
            }
        };
    }
}
