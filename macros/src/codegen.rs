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

    let get_ctx = state.context_type.as_ref().map(|_| {
        quote! { let ctx = self.context.borrow(); }
    });

    let invalid_event_state_str = format!("{} received event while in invalid state", state_name);

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

        let transitions = sub_state
            .out_transitions
            .iter()
            .map(|t| generate_transition(state, sub_state, t));

        quote! {
            #state_type::#state_name => {
                match self.#field_ident.process_event(event.clone()) {
                    ::umlstate::ProcessResult::Handled => ::umlstate::ProcessResult::Handled,
                    ::umlstate::ProcessResult::Unhandled => {
                        #get_ctx
                        match event.clone() {
                            #(#transitions),*
                            _ => ::umlstate::ProcessResult::Unhandled,
                        }
                    }
                }
            }
        }
    });

    let internal_transitions = state
        .internal_transitions
        .iter()
        .map(|t| generate_internal_transition(state, t));

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
    let exit_action = generate_exit(state);

    let active_state_decl = if state.states.is_empty() {
        Some(quote! { Active })
    } else {
        None
    };

    let active_arm = if state.states.is_empty() {
        quote! {
            #state_type::Active => ::umlstate::ProcessResult::Unhandled,
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
                #(#state_decl),*
                #active_state_decl
            }

            pub(in #root_path::super) struct #state_name #generics {
                #context_field
                state: ::std::option::Option<#state_type>,
                #(#state_fields),*
            }

            impl #impl_generics #state_name #ty_generics #where_clause {
                pub fn new(#context_field) -> Self {
                    Self {
                        #context_init
                        state: ::std::option::Option::None,
                        #(#states_init),*
                    }
                }

                pub fn state(&self) -> ::std::option::Option<#state_type> {
                    self.state.clone()
                }

                pub(super) fn process_event(&mut self, event: Event) -> ::umlstate::ProcessResult {
                    let state = if let ::std::option::Option::Some(s) = &self.state {
                        s
                    } else {
                        panic!(#invalid_event_state_str);
                    };

                    let result = match state {
                        #(#process_states),*
                        #active_arm
                    };

                    if result == ::umlstate::ProcessResult::Handled {
                        return result
                    }

                    #get_ctx
                    match event {
                        #(#internal_transitions),*
                        _ => ::umlstate::ProcessResult::Unhandled
                    }
                }

                pub fn enter(&mut self) {
                    #enter_action
                }

                pub fn exit(&mut self) {
                    #exit_action
                }
            }

            #(#states)*
        }
    }
}

fn generate_internal_transition(
    state: &lower::State,
    t: &lower::Transition,
) -> proc_macro2::TokenStream {
    let drop_ctx = state.context_type.as_ref().map(|_| quote! { drop(ctx); });
    let mut_ctx = state.context_type.as_ref().map(|_| {
        quote! { let mut ctx = self.context.borrow_mut(); }
    });
    let event = &t.event;
    let event_pat = &t.event_pat.as_ref().map(|p| quote! { @ #p });
    let guard = t.guard.as_ref().map(|g| quote! { if #g });
    let action = &t.action;

    quote! {
        Event::#event(event #event_pat) #guard => {
            #drop_ctx
            {
                #mut_ctx
                #action;
            }
            ::umlstate::ProcessResult::Handled
        }
    }
}

fn generate_transition(
    parent: &lower::State,
    cur_state: &lower::State,
    t: &lower::Transition,
) -> proc_macro2::TokenStream {
    let drop_ctx = parent.context_type.as_ref().map(|_| quote! { drop(ctx); });
    let mut_ctx = parent.context_type.as_ref().map(|_| {
        quote! { let mut ctx = self.context.borrow_mut(); }
    });
    let event = &t.event;
    let event_pat = &t.event_pat.as_ref().map(|p| quote! { @ #p });
    let guard = t.guard.as_ref().map(|g| quote! { if #g });
    let action = &t.action;
    let state_type = &parent.state_type;
    let cur_state_field = &cur_state.field_ident;
    let next_state_name = &t.target;
    let next_state_field = &t.target_state_field;

    quote! {
        Event::#event(event #event_pat) #guard => {
            #drop_ctx
            self.#cur_state_field.exit();
            {
                #mut_ctx
                #action;
            }
            self.state = ::std::option::Option::Some(#state_type::#next_state_name);
            self.#next_state_field.enter();
            ::umlstate::ProcessResult::Handled
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
    let state_name;
    let action;
    let entry_action = &state.entry;
    let enter_substate;

    if let Some(t) = transition {
        state_name = t.target.as_ref().unwrap().clone();
        action = &t.action;
        let field_ident = &t.target_state_field;
        enter_substate = Some(quote! {
            self.#field_ident.enter();
        });
    } else {
        state_name = quote::format_ident!("Active");
        action = &None;
        enter_substate = None;
    }

    let invalid_enter_state_str = format!("{}.enter() while in active state", &state.ident);

    quote! {
        if self.state.is_some() {
            panic!(#invalid_enter_state_str);
        }
        {
            #mut_ctx
            #action;
            self.state = ::std::option::Option::Some(#state_type::#state_name);
            #entry_action;
        }
        #enter_substate
    }
}

fn generate_exit(state: &lower::State) -> proc_macro2::TokenStream {
    let mut_ctx = state.context_type.as_ref().map(|_| {
        quote! { let mut ctx = self.context.borrow_mut(); }
    });
    let state_type = &state.state_type;
    let exit_action = &state.exit;
    let sub_state_exits = state.states.iter().map(|s| {
        let ident = &s.ident;
        let field_ident = &s.field_ident;
        quote! {
            #state_type::#ident => self.#field_ident.exit()
        }
    });
    let simple_active_arm = if state.states.is_empty() {
        quote! { _ => () }
    } else {
        quote! {}
    };

    let invalid_exit_state_str = format!("{}.exit() while in not in active state", &state.ident);

    quote! {
        let state = if let ::std::option::Option::Some(s) = &self.state {
            s
        } else {
            panic!(#invalid_exit_state_str);
        };

        match state {
            #(#sub_state_exits),*
            #simple_active_arm
        }
        {
            #mut_ctx
            self.state = ::std::option::Option::None;
            #exit_action;
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
