use quote::{format_ident, quote};

use crate::lower;

pub fn generate(model: &lower::Model) -> proc_macro2::TokenStream {
    let mut tt = proc_macro2::TokenStream::default();

    for m in &model.machines {
        tt.extend(generate_machine(m));
    }

    tt
}

fn generate_machine(machine: &lower::TopMachine) -> proc_macro2::TokenStream {
    let ident = &machine.machine.type_ident;
    let generics = &machine.generics;
    let context = format_ident!("{}Context", ident);
    let modname = format_ident!("{}_mod", ident.to_string().to_lowercase());

    let event_decl = machine.events.iter().map(|(path, ident)| {
        quote! {
            #ident(#path)
        }
    });

    let process_impls = machine.events.iter().map(|(path, ident)| {
        quote! {
            impl #generics ::umlstate::EventProcessor<#path> for Machine #generics {
                fn process(&mut self, event: #path) -> ::umlstate::ProcessResult {
                    self.machine.process_internal(&mut self.context, Event::#ident(event))
                }
            }
        }
    });

    let topmachine_name = &machine.machine.type_ident;
    let topmachine_state = &machine.machine.state_type;
    let machine_decl = generate_submachine(&machine.machine, &context);

    quote! {
        mod #modname {
            use super::*;

            #[derive(Clone)]
            enum Event {
                #(#event_decl),*
            }

            pub(crate) struct Machine #generics {
                pub context: #context #generics,
                machine: #topmachine_name,
            }

            impl #generics Machine #generics {
                pub fn new(context: #context #generics) -> Self {
                    Machine {
                        context,
                        machine: #topmachine_name ::new()
                    }
                }

                pub fn start(&mut self) {
                    self.machine.enter(&mut self.context);
                }

                pub fn state_config(&self) -> std::vec::IntoIter<&#topmachine_state> {
                    self.machine.state_config()
                }
            }

            #(#process_impls)*

            #machine_decl
        }

        use #modname::#topmachine_state;
        use #modname::Machine as #ident;
    }
}

fn generate_submachine(
    machine: &lower::SubMachine,
    context: &syn::Ident,
) -> proc_macro2::TokenStream {
    let machine_name = &machine.type_ident;
    let state_type = &machine.state_type;

    let initial_state = &machine.initial_state;

    let invalid_state_str = format!("{} received event while in invalid state", machine_name);

    let state_decl = machine.states.iter().map(|s| s.ident.clone());

    let submachine_fields = machine.machines.iter().map(|m| {
        let type_ident = &m.type_ident;
        let field_ident = &m.field_ident;
        quote! {
            #field_ident: #type_ident
        }
    });

    let submachines = machine
        .machines
        .iter()
        .map(|m| generate_submachine(m, context));

    let submachine_init = machine.machines.iter().map(|m| {
        let type_ident = &m.type_ident;
        let field_ident = &m.field_ident;
        quote! {
            #field_ident: #type_ident::new()
        }
    });

    let process_states = machine.states.iter().map(|state| {
        let state_name = &state.ident;

        let exit_action = state.submachine_field.as_ref().map(|field_ident| {
            quote! {
                self.#field_ident.exit(ctx);
            }
        });

        let transitions = state.out_transitions.iter().map(|t| {
            let event = &t.event;
            let event_pat = &t.event_pat.as_ref().map(|p| quote! { @ #p });
            let target = &t.target;
            let action = &t.action;
            let guard = t.guard.as_ref().map(|g| quote! { if #g });

            let entry_action = t.target_machine.as_ref().map(|field_ident| {
                quote! {
                    self.#field_ident.enter(ctx);
                }
            });

            quote! {
                Event::#event(event #event_pat) #guard => {
                    let ctx = mut_ctx;
                    #exit_action
                    #action;
                    self.state = #state_type::#target;
                    #entry_action
                    ::umlstate::ProcessResult::Handled
                }
            }
        });
        let event_handlers = quote! {
            match event {
                #(#transitions),*
                _ => ::umlstate::ProcessResult::Unhandled,
            }
        };

        let state_handler = match &state.submachine_field {
            None => quote! {
                #state_type::#state_name => #event_handlers
            },
            Some(field_ident) => quote! {
                #state_type::#state_name =>
                match self.#field_ident.process_internal(mut_ctx, event.clone()) {
                    ::umlstate::ProcessResult::Handled => ::umlstate::ProcessResult::Handled,
                    ::umlstate::ProcessResult::Unhandled => #event_handlers,
                }
            },
        };

        state_handler
    });

    quote! {
        #[derive(Debug)]
        pub enum #state_type {
            __NotStarted,
            __Exited,
            #(#state_decl),*
        }

        struct #machine_name {
            state: #state_type,
            #(#submachine_fields),*
        }

        impl #machine_name {
            pub fn new() -> Self {
                Self {
                    state: #state_type::__NotStarted,
                    #(#submachine_init),*
                }
            }

            pub fn state_config(&self) -> ::std::vec::IntoIter<&#state_type> {
                vec![&self.state].into_iter()
            }

            fn process_internal(&mut self, mut_ctx: &mut #context, event: Event) -> ::umlstate::ProcessResult {
                let ctx = &mut_ctx;
                match self.state {
                    #(#process_states),*
                    #state_type::__NotStarted | #state_type::__Exited => {
                        panic!(#invalid_state_str);
                    }
                }
            }

            fn enter(&mut self, ctx: &mut #context) {
                self.state = #state_type::#initial_state;
            }

            fn exit(&mut self, ctx: &mut #context) {
                self.state = #state_type::__Exited;
            }
        }

        #(#submachines)*
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{analyze, parse};

    #[test]
    fn basic() {
        let ast: parse::UmlState = syn::parse_quote! {
            machine Foo {
                state A;
                A + E(_) => A;
            }
        };

        let model = analyze::analyze(ast).unwrap();
        let lower_model = lower::lower(model);
        let _code = generate(&lower_model);
    }
}
