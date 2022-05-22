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
    let vis = &machine.machine.vis;
    let ident = &machine.machine.type_ident;
    let generics = &machine.machine.generics;
    let modname = format_ident!("{}_mod", ident.to_string().to_lowercase());

    let event_decl = machine.events.iter().map(|(path, ident)| {
        quote! {
            #ident(#path)
        }
    });

    let process_impls = machine.events.iter().map(|(path, event_ident)| {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        quote! {
            impl #impl_generics ::umlstate::EventProcessor<#path> for #ident #ty_generics #where_clause {
                fn process(&mut self, event: #path) -> ::umlstate::ProcessResult {
                    self.process_event(Event::#event_ident(event))
                }
            }
        }
    });

    let topmachine_state = &machine.machine.state_type;
    let machine_decl = generate_submachine(&machine.machine);

    quote! {
        mod #modname {
            use super::*;
            use std::cell::RefCell;
            use std::rc::Rc;

            #[derive(Clone)]
            enum Event {
                #(#event_decl),*
            }

            #(#process_impls)*

            #machine_decl
        }

        #vis use #modname::#topmachine_state;
        #vis use #modname::#ident;
    }
}

fn generate_submachine(machine: &lower::SubMachine) -> proc_macro2::TokenStream {
    let machine_name = &machine.type_ident;
    let state_type = &machine.state_type;

    let initial_state = &machine.initial_state;

    let invalid_event_state_str = format!("{} received event while in invalid state", machine_name);
    let invalid_enter_state_str = format!("{}.enter() while in active state", machine_name);
    let invalid_exit_state_str = format!("{}.exit() while in not in active state", machine_name);

    let state_decl = machine.states.iter().map(|s| s.ident.clone());

    let submachine_fields = machine.machines.iter().map(|m| {
        let type_ident = &m.type_ident;
        let field_ident = &m.field_ident;
        let (_impl_generics, ty_generics, _where_clause) = m.generics.split_for_impl();
        quote! {
            #field_ident: #type_ident #ty_generics
        }
    });

    let submachines = machine.machines.iter().map(|m| generate_submachine(m));

    let submachine_init = machine.machines.iter().map(|m| {
        let type_ident = &m.type_ident;
        let field_ident = &m.field_ident;
        let context_arg = m.context_type.as_ref().map(|_| quote! { context.clone() });
        quote! {
            #field_ident: #type_ident::new(#context_arg)
        }
    });

    let process_states = machine.states.iter().map(|state| {
        let state_name = &state.ident;

        let exit_action = state.submachine_field.as_ref().map(|field_ident| {
            quote! {
                self.#field_ident.exit();
            }
        });

        let get_ctx = machine.context_type.as_ref().map(|_| {
            quote! { let ctx = self.context.borrow(); }
        });
        let drop_ctx = machine.context_type.as_ref().map(|_| quote! { drop(ctx); });
        let mut_ctx = machine.context_type.as_ref().map(|_| {
            quote! { let mut ctx = self.context.borrow_mut(); }
        });

        let transitions = state.out_transitions.iter().map(|t| {
            let event = &t.event;
            let event_pat = &t.event_pat.as_ref().map(|p| quote! { @ #p });
            let target = &t.target;
            let action = &t.action;
            let guard = t.guard.as_ref().map(|g| quote! { if #g });

            let entry_action = t.target_machine.as_ref().map(|field_ident| {
                quote! {
                    self.#field_ident.enter();
                }
            });

            quote! {
                Event::#event(event #event_pat) #guard => {
                    #drop_ctx
                    #exit_action
                    {
                        #mut_ctx
                        #action;
                    }
                    self.state = #state_type::#target;
                    #entry_action
                    ::umlstate::ProcessResult::Handled
                }
            }
        });
        let event_handlers = quote! {
            #get_ctx
            match event {
                #(#transitions),*
                _ => ::umlstate::ProcessResult::Unhandled,
            }
        };

        let state_handler = match &state.submachine_field {
            None => quote! {
                #state_type::#state_name => {
                    #event_handlers
                }
            },
            Some(field_ident) => quote! {
                #state_type::#state_name =>
                match self.#field_ident.process_event(event.clone()) {
                    ::umlstate::ProcessResult::Handled => ::umlstate::ProcessResult::Handled,
                    ::umlstate::ProcessResult::Unhandled => {
                        #event_handlers
                    }
                }
            },
        };

        state_handler
    });

    let generics = &machine.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let context_field = machine.context_type.as_ref().map(|ident| {
        quote! {
            context: Rc<RefCell<#ident>>,
        }
    });
    let context_init = machine.context_type.as_ref().map(|_ident| {
        quote! {
            context: context.clone(),
        }
    });

    quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub(super) enum #state_type {
            __NotStarted,
            __Exited,
            #(#state_decl),*
        }

        pub(super) struct #machine_name #generics {
            #context_field
            state: #state_type,
            #(#submachine_fields),*
        }

        impl #impl_generics #machine_name #ty_generics #where_clause {
            pub fn new(#context_field) -> Self {
                Self {
                    #context_init
                    state: #state_type::__NotStarted,
                    #(#submachine_init),*
                }
            }

            pub fn state(&self) -> &#state_type {
                &self.state
            }

            fn process_event(&mut self, event: Event) -> ::umlstate::ProcessResult {
                match self.state {
                    #(#process_states),*
                    #state_type::__NotStarted | #state_type::__Exited => {
                        panic!(#invalid_event_state_str);
                    }
                }
            }

            pub fn enter(&mut self) {
                match self.state {
                    #state_type::__NotStarted | #state_type::__Exited => (),
                    _ => {
                        panic!(#invalid_enter_state_str);
                    }
                }
                self.state = #state_type::#initial_state;
            }

            pub fn exit(&mut self) {
                match self.state {
                    #state_type::__NotStarted | #state_type::__Exited => {
                        panic!(#invalid_exit_state_str);
                    }
                    _ => ()
                }
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
            pub(crate) machine Foo {
                state A;
                A + E(_) => A;
            }
        };

        let model = analyze::analyze(ast).unwrap();
        let lower_model = lower::lower(model);
        let _code = generate(&lower_model);
    }
}
