use quote::quote;

use crate::lower;

pub fn generate(model: &lower::Model) -> proc_macro2::TokenStream {
    let mut tt = proc_macro2::TokenStream::default();

    for m in &model.machines {
        tt.extend(generate_machine(m));
    }

    tt
}

fn generate_machine(machine: &lower::TopMachine) -> proc_macro2::TokenStream {
    let vis = &machine.vis;
    let ident = &machine.ident;
    let state_ident = &machine.state.ident;
    let generics = &machine.state.generics;
    let mod_name = &machine.state.mod_name;

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

    let state_decl = generate_state(&machine.state);
    let topmachine_state = &machine.state.state_type;

    quote! {
        mod #mod_name {
            use super::*;

            #[derive(Clone)]
            enum Event {
                #(#event_decl),*
            }

            #(#process_impls)*

            #state_decl
        }

        #vis use #mod_name::#mod_name::#topmachine_state;
        #vis use #mod_name::#mod_name::#state_ident as #ident;
    }
}

fn generate_state(state: &lower::State) -> proc_macro2::TokenStream {
    let state_name = &state.ident;
    let root_path = &state.root_path;
    let mod_name = &state.mod_name;
    let state_type = &state.state_type;

    let invalid_event_state_str = format!("{} received event while in invalid state", state_name);
    let invalid_enter_state_str = format!("{}.enter() while in active state", state_name);
    let invalid_exit_state_str = format!("{}.exit() while in not in active state", state_name);

    let state_decl = state.states.iter().map(|s| s.ident.clone());

    let state_fields = state.states.iter().map(|s| {
        let state_mod = &s.mod_name;
        let state_ident = &s.ident;
        let field_ident = &s.field_ident;
        let (_impl_generics, ty_generics, _where_clause) = s.generics.split_for_impl();
        quote! {
            #field_ident: #state_mod::#state_ident #ty_generics
        }
    });

    let states = state.states.iter().map(|m| generate_state(m));

    let states_init = state.states.iter().map(|s| {
        let type_ident = &s.ident;
        let mod_name = &s.mod_name;
        let field_ident = &s.field_ident;
        let context_arg = s.context_type.as_ref().map(|_| quote! { context.clone() });
        quote! {
            #field_ident: #mod_name::#type_ident::new(#context_arg)
        }
    });

    let process_states = state.states.iter().map(|sub_state| {
        let state_name = &sub_state.ident;
        let field_ident = &sub_state.field_ident;
        let exit_action = quote! {
            self.#field_ident.exit();
        };

        let get_ctx = sub_state.context_type.as_ref().map(|_| {
            quote! { let ctx = self.context.borrow(); }
        });
        let drop_ctx = sub_state
            .context_type
            .as_ref()
            .map(|_| quote! { drop(ctx); });

        let transitions = sub_state.out_transitions.iter().map(|t| {
            let event = &t.event;
            let event_pat = &t.event_pat.as_ref().map(|p| quote! { @ #p });
            let guard = t.guard.as_ref().map(|g| quote! { if #g });
            let entry = generate_entry(&state, Some(t));

            quote! {
                Event::#event(event #event_pat) #guard => {
                    #drop_ctx
                    #exit_action
                    #entry
                    ::umlstate::ProcessResult::Handled
                }
            }
        });

        quote! {
            #state_type::#state_name => {
                match self.#field_ident.process_event(event.clone()) {
                    ::umlstate::ProcessResult::Handled => ::umlstate::ProcessResult::Handled,
                    ::umlstate::ProcessResult::Unhandled => {
                        #get_ctx
                        match event {
                            #(#transitions),*
                            _ => ::umlstate::ProcessResult::Unhandled,
                        }
                    }
                }
            }
        }
    });

    let generics = &state.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let context_field = state.context_type.as_ref().map(|ident| {
        quote! {
            context: Rc<RefCell<#ident>>,
        }
    });
    let context_init = state.context_type.as_ref().map(|_ident| {
        quote! {
            context: context.clone(),
        }
    });
    let enter_action = generate_entry(state, state.initial_transition.as_ref());

    let entered_state_decl = if state.states.is_empty() {
        Some(quote! { __Entered, })
    } else {
        None
    };

    let entered_arm = if state.states.is_empty() {
        quote! {
            #state_type::__Entered => ::umlstate::ProcessResult::Unhandled,
        }
    } else {
        quote! {}
    };

    quote! {
        pub mod #mod_name {
            use super::*;
            use std::cell::RefCell;
            use std::rc::Rc;

            #[derive(Clone, Debug, PartialEq)]
            pub enum #state_type {
                __NotStarted,
                __Exited,
                #entered_state_decl
                #(#state_decl),*
            }

            pub(in #root_path::super) struct #state_name #generics {
                #context_field
                state: #state_type,
                #(#state_fields),*
            }

            impl #impl_generics #state_name #ty_generics #where_clause {
                pub fn new(#context_field) -> Self {
                    Self {
                        #context_init
                        state: #state_type::__NotStarted,
                        #(#states_init),*
                    }
                }

                pub fn state(&self) -> &#state_type {
                    &self.state
                }

                pub(super) fn process_event(&mut self, event: Event) -> ::umlstate::ProcessResult {
                    match self.state {
                        #(#process_states),*
                        #entered_arm
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
                    #enter_action
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

            #(#states)*
        }
    }
}

fn generate_entry(
    state: &lower::State,
    transition: Option<&lower::Transition>,
) -> proc_macro2::TokenStream {
    let mut_ctx = state.context_type.as_ref().map(|_| {
        quote! { let mut ctx = self.context.borrow_mut(); }
    });
    let state_type = &state.state_type;

    if let Some(t) = transition {
        let action = &t.action;
        let target = &t.target;
        let field_ident = &t.target_state_field;
        let entry_action = quote! {
            self.#field_ident.enter();
        };

        quote! {
            {
                #mut_ctx
                #action;
            }
            self.state = #state_type::#target;
            #entry_action
        }
    } else {
        quote! {
            self.state = #state_type::__Entered;
        }
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
                <*> => A;
                A + E(_) => A;
            }
        };

        let model = analyze::analyze(ast).unwrap();
        let lower_model = lower::lower(model);
        let _code = generate(&lower_model);
    }
}
