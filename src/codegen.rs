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
    let statename = format_ident!("{}State", ident);
    let modname = format_ident!("{}_mod", ident.to_string().to_lowercase());

    let state_decl = machine.states.keys();

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

    let initial_state = &machine.initial_state;

    let process_states = machine.states.iter().map(|(statename, state)| {
        let transitions = state.out_transitions.iter().map(|t| {
            let event = &t.event;
            let target = &t.target;
            quote! {
                Event::#event(event) => {
                    self.state = State::#target;
                }
            }
        });
        quote! {
            State::#statename => match &event {
                #(#transitions),*
                _ => (),
            }
        }
    });

    quote! {
        mod #modname {
            use super::*;

            #[derive(Debug)]
            pub enum State {
                #(#state_decl),*
            }

            enum Event {
                #(#event_decl),*
            }

            pub(crate) struct Machine {
                context: #context,
                state: State,
            }

            impl Machine {
                pub fn new(context: #context) -> Self {
                    Machine {
                        context,
                        state: State::#initial_state,
                    }
                }

                pub fn state_config(&self) -> std::vec::IntoIter<&State> {
                    vec![&self.state].into_iter()
                }

                fn process_internal(&mut self, event: Event) {
                    match self.state {
                        #(#process_states),*
                    }
                }
            }

            pub(crate) trait EventProcessor<E> {
                fn process(&mut self, event: E);
            }

            #(#process_impls)*
        }

        use #modname::Machine as #ident;
        use #modname::State as #statename;
        use #modname::EventProcessor;
    }
    .into()
}
