use quote::{format_ident, quote};

use crate::analyze;

pub fn generate(model: analyze::Model) -> proc_macro2::TokenStream {
    let mut tt = proc_macro2::TokenStream::default();

    for m in model.items {
        tt.extend(generate_machine(&m));
    }

    tt
}

fn generate_machine(machine: &analyze::Machine) -> proc_macro2::TokenStream {
    let ident = &machine.ident;
    let generics = &machine.generics;
    let context = format_ident!("{}Context", ident);
    let statename = format_ident!("{}State", ident);
    let modname = format_ident!("{}_mod", ident.to_string().to_lowercase());

    let state_decl = machine.states.keys();

    let event_decl = machine.events.iter().map(|(path, ident)| {
        quote! {
            #ident(#path)
        }
    });

    let process_impls = machine.events.iter().map(|(path, ident)| {
        quote! {
            impl #generics ::umlstate::EventProcessor<#path> for Machine #generics {
                fn process(&mut self, event: #path) -> ::umlstate::ProcessResult {
                    self.process_internal(Event::#ident(event))
                }
            }
        }
    });

    let initial_state = &machine.initial_state;

    let process_states = machine.states.iter().map(|(statename, state)| {
        let transitions = state.out_transitions.iter().map(|t| {
            let event = &t.event;
            let event_pat = &t.event_pat.as_ref().map(|p| quote! { @ #p });
            let target = &t.target;
            let action = &t.action;
            let guard = t.guard.as_ref().map(|g| quote! { if #g });
            quote! {
                Event::#event(event #event_pat) #guard => {
                    let ctx = &mut self.context;
                    #action;
                    self.state = State::#target;
                    ::umlstate::ProcessResult::Handled
                }
            }
        });
        quote! {
            State::#statename => match event {
                #(#transitions),*
                _ => ::umlstate::ProcessResult::Unhandled,
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

            pub(crate) struct Machine #generics {
                pub context: #context #generics,
                state: State,
            }

            impl #generics Machine #generics {
                pub fn new(context: #context #generics) -> Self {
                    Machine {
                        context,
                        state: State::#initial_state,
                    }
                }

                pub fn state_config(&self) -> std::vec::IntoIter<&State> {
                    vec![&self.state].into_iter()
                }

                fn process_internal(&mut self, event: Event) -> ::umlstate::ProcessResult {
                    match self.state {
                        #(#process_states),*
                    }
                }
            }

            #(#process_impls)*
        }

        use #modname::Machine as #ident;
        use #modname::State as #statename;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parse;

    #[test]
    fn basic() {
        let ast: parse::UmlState = syn::parse_quote! {
            machine Foo {
                state A;
                A + E(_) => A;
            }
        };

        let model = analyze::analyze(ast).unwrap();
        let _code = generate(model);
    }
}