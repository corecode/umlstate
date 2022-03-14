use quote::{format_ident, quote};

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
    let context = format_ident!("{}Context", ident);
    let modname = format_ident!("{}_mod", ident);

    let event_decl = machine.events.iter().map(|ident| {
        quote! {
            #ident(#ident)
        }
    });

    let process_impls = machine.events.iter().map(|ident| {
        quote! {
            impl EventProcessor<#ident> for Machine {
                fn process(&mut self, event: #ident) {
                    self.process_internal(Event::#ident(event));
                }
            }
        }
    });

    quote! {
        mod #modname {
            use super::*;

            enum Event {
                #(#event_decl),*
            }

            pub(crate) struct Machine {
                context: #context
            }

            impl Machine {
                pub fn new(context: #context) -> Self {
                    Machine { context }
                }

                fn process_internal(&mut self, _event: Event) {
                }
            }

            pub(crate) trait EventProcessor<E> {
                fn process(&mut self, event: E);
            }

            #(#process_impls)*
        }

        use #modname::Machine as #ident;
        use #modname::EventProcessor;
    }
    .into()
}
