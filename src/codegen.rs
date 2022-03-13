use proc_macro2::Ident;
use quote::quote;

use crate::analyze;

pub fn generate(model: analyze::Model) -> proc_macro::TokenStream {
    let mut tt = proc_macro::TokenStream::default();

    for m in model.items {
        tt.extend(generate_machine(&m));
    }

    tt
}

fn generate_machine(machine: &analyze::Machine) -> proc_macro::TokenStream {
    let ident = &machine.ident;
    let context = Ident::new(format!("{}Context", ident).as_str(), ident.span());
    let modname = Ident::new(format!("{}_mod", ident).as_str(), ident.span());

    quote! {
        mod #modname {
            use super::*;

            pub(crate) struct Machine {
                context: #context
            }

            impl Machine {
                pub fn new(context: #context) -> Self {
                    Machine { context }
                }
            }
        }

        use #modname::Machine as #ident;
    }
    .into()
}
