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
    let generics = &machine.generics;
    let mod_name = &machine.mod_name;
    let state_mod_name = &machine.state.mod_name;

    let event_decl = machine.events.iter().map(|(path, ident)| {
        quote! {
            #ident(#path)
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let context_use;
    let context_zst;
    let context_field;
    let context_arg_sig;
    let context_field_init;
    let context_arg;

    let context_ident = &machine.context.ident;
    let context_methods = &machine.context.methods;
    let context_decl = quote! {
        pub trait #context_ident {
            #(#context_methods)*
        }
    };

    if let Some(zst) = &machine.context.zst {
        context_use = None;
        context_zst = Some(quote! {
            struct #zst;
            impl #context_ident for #zst {}
        });
        context_field = quote! { #zst };
        context_arg_sig = quote! {};
        context_field_init = quote! { #zst };
        context_arg = quote! { &self.context };
    } else {
        context_use = Some(quote! {
            #vis use #mod_name::#context_ident;
        });
        context_zst = None;
        context_arg_sig = quote! { context: Context };
        context_field = quote! { Context };
        context_arg = quote! { &self.context };
        context_field_init = quote! { context };
    }

    let process_impls = machine.events.iter().map(|(path, event_ident)| {
        quote! {
            impl #impl_generics ::umlstate::EventProcessor<#path> for #ident #ty_generics #where_clause {
                fn process(&mut self, event: #path) -> ::umlstate::ProcessResult {
                    self.state.process_event(#context_arg, Event::#event_ident(event))
                }
            }
        }
    });

    let state_decl = generate_state(&machine.state);
    let topmachine_state = &machine.state.state_type;

    quote! {
        mod #mod_name {
            use super::*;
            use ::std::ops::DerefMut;

            #[derive(Clone)]
            enum Event {
                #(#event_decl),*
            }

            #context_decl
            #context_zst

            pub struct #ident #impl_generics #where_clause {
                context: #context_field,
                state: #state_mod_name::#state_ident,
            }

            impl #impl_generics #ident #ty_generics #where_clause {
                pub fn new(#context_arg_sig) -> Self {
                    Self {
                        context: #context_field_init,
                        state: #state_mod_name::#state_ident::new(),
                    }
                }

                pub fn state(&self) -> ::std::option::Option<#state_mod_name::#topmachine_state> {
                    self.state.state()
                }

                pub fn enter(&mut self) {
                    self.state.enter(#context_arg);
                }

                pub fn exit(&mut self) {
                    self.state.exit(#context_arg);
                }
            }

            #(#process_impls)*

            #state_decl
        }

        #vis use #mod_name::#state_mod_name::#topmachine_state;
        #vis use #mod_name::#ident;
        #context_use
    }
}

fn generate_state(state: &lower::State) -> proc_macro2::TokenStream {
    let state_name = &state.ident;
    let root_path = &state.root_path;
    let mod_name = &state.mod_name;
    let state_type = &state.state_type;
    let context_type = &state.context_type;

    let invalid_event_state_str = format!("{} received event while in invalid state", state_name);

    let state_decl = state.states.iter().map(|s| s.ident.clone());

    let states_or_regions = if state.states.len() > 0 {
        &state.states
    } else {
        &state.regions
    };

    let state_fields = states_or_regions.iter().map(|s| {
        let state_mod = &s.mod_name;
        let state_ident = &s.ident;
        let field_ident = &s.field_ident;
        quote! {
            #field_ident: #state_mod::#state_ident
        }
    });

    let states = states_or_regions.iter().map(|m| generate_state(m));

    let states_init = states_or_regions.iter().map(|s| {
        let type_ident = &s.ident;
        let mod_name = &s.mod_name;
        let field_ident = &s.field_ident;
        quote! {
            #field_ident: #mod_name::#type_ident::new()
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
                match self.#field_ident.process_event(ctx, event.clone()) {
                    ::umlstate::ProcessResult::Handled => ::umlstate::ProcessResult::Handled,
                    ::umlstate::ProcessResult::Unhandled => {
                        match event.clone() {
                            #(#transitions),*
                            _ => ::umlstate::ProcessResult::Unhandled,
                        }
                    }
                }
            }
        }
    });

    let process_regions = state.regions.iter().map(|r| {
        let field_ident = &r.field_ident;

        quote! {
            {
                let r = self.#field_ident.process_event(ctx, event.clone());
                if r == ::umlstate::ProcessResult::Handled {
                    result = r;
                }
            }
        }
    });

    let internal_transitions = state
        .internal_transitions
        .iter()
        .map(|t| generate_internal_transition(state, t));

    let enter_action = generate_entry(state, state.initial_transition.as_ref());
    let exit_action = generate_exit(state);

    let active_state_decl = if state.states.is_empty() {
        Some(quote! { Active })
    } else {
        None
    };

    let active_arm = if state.states.is_empty() {
        quote! {
            #state_type::Active => {
                let mut result = ::umlstate::ProcessResult::Unhandled;
                #(#process_regions)*
                result
            }
        }
    } else {
        quote! {}
    };

    quote! {
        pub mod #mod_name {
            use super::*;

            #[derive(Clone, Debug, PartialEq)]
            pub enum #state_type {
                #(#state_decl),*
                #active_state_decl
            }

            pub(in #root_path::super) struct #state_name {
                state: ::std::option::Option<#state_type>,
                #(#state_fields),*
            }

            impl #state_name {
                pub fn new() -> Self {
                    Self {
                        state: ::std::option::Option::None,
                        #(#states_init),*
                    }
                }

                pub fn state(&self) -> ::std::option::Option<#state_type> {
                    self.state.clone()
                }

                pub(super) fn process_event(&mut self, ctx: &impl #context_type, event: Event) -> ::umlstate::ProcessResult {
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

                    match event {
                        #(#internal_transitions),*
                        _ => ::umlstate::ProcessResult::Unhandled
                    }
                }

                pub(super) fn enter(&mut self, ctx: &impl #context_type) {
                    #enter_action
                }

                pub(super) fn exit(&mut self, ctx: &impl #context_type) {
                    #exit_action
                }
            }

            #(#states)*
        }
    }
}

fn generate_internal_transition(
    _state: &lower::State,
    t: &lower::Transition,
) -> proc_macro2::TokenStream {
    let event = &t.event;
    let event_pat = &t.event_pat.as_ref().map(|p| quote! { @ #p });
    let guard = t.guard.as_ref().map(|g| quote! { if #g });
    let action = &t.action;

    quote! {
        Event::#event(event #event_pat) #guard => {
            {
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
            self.#cur_state_field.exit(ctx);
            {
                #action;
            }
            self.state = ::std::option::Option::Some(#state_type::#next_state_name);
            self.#next_state_field.enter(ctx);
            ::umlstate::ProcessResult::Handled
        }
    }
}

fn generate_entry(
    state: &lower::State,
    transition: Option<&lower::Transition>,
) -> proc_macro2::TokenStream {
    let state_type = &state.state_type;
    let state_name;
    let action;
    let entry_action = &state.entry;
    let enter_substate;

    if let Some(t) = transition {
        state_name = t.target.as_ref().unwrap().clone();
        action = &t.action;
        let field_ident = &t.target_state_field;
        enter_substate = quote! {
            self.#field_ident.enter(ctx);
        };
    } else {
        state_name = quote::format_ident!("Active");
        action = &None;
        let enter_regions = state.regions.iter().map(|s| {
            let field_ident = &s.field_ident;
            quote! {
                self.#field_ident.enter(ctx);
            }
        });
        enter_substate = quote! { #(#enter_regions)* };
    }

    let invalid_enter_state_str = format!("{}.enter() while in active state", &state.ident);

    quote! {
        if self.state.is_some() {
            panic!(#invalid_enter_state_str);
        }
        {
            #action;
            self.state = ::std::option::Option::Some(#state_type::#state_name);
            #entry_action;
        }
        #enter_substate
    }
}

fn generate_exit(state: &lower::State) -> proc_macro2::TokenStream {
    let state_type = &state.state_type;
    let exit_action = &state.exit;
    let sub_state_exits = state.states.iter().map(|s| {
        let ident = &s.ident;
        let field_ident = &s.field_ident;
        quote! {
            #state_type::#ident => self.#field_ident.exit(ctx)
        }
    });
    let region_exits = state.regions.iter().map(|s| {
        let field_ident = &s.field_ident;
        quote! {
            self.#field_ident.exit(ctx);
        }
    });
    let simple_active_arm = if state.states.is_empty() {
        quote! {
            _ => {
                #(#region_exits)*
            }
        }
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
