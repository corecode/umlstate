use quote::quote;
use syn::parse::Parse;
use syn::{Result, Token};

mod kw {
    syn::custom_keyword!(machine);
    syn::custom_keyword!(state);
}

struct UmlState {
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

struct Machine {
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

enum MachineItem {
    State(ItemState),
    Transition(ItemTransition),
}

struct ItemState {
    pub state_token: kw::state,
    pub ident: syn::Ident,
    pub semi_token: Option<Token![;]>,
}

struct ItemTransition {
    pub source: syn::Ident,
    pub plus_token: Token![+],
    pub event: syn::Ident,
    pub arrow_token: Token![=>],
    pub target: syn::Ident,
    pub semi_token: Option<Token![;]>,
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
            semi_token: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn umlstate(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {}.into()
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

                S1 + E2 => S2;
                S2 + E1 => S1;
            }
        };
    }
}
