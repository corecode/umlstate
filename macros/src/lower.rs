use std::collections::HashMap;

use quote::{format_ident, quote};

use crate::analyze;

pub struct Model {
    pub machines: Vec<TopMachine>,
}

pub struct TopMachine {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub mod_name: syn::Ident,
    pub events: Vec<(syn::Path, syn::Ident)>,
    pub context: Context,
    pub generics: syn::Generics,
    pub state: State,
}

pub struct Context {
    pub ident: syn::Ident,
    pub methods: Vec<syn::TraitItemMethod>,
    pub zst: Option<syn::Ident>,
}

pub struct State {
    pub mod_name: syn::Ident,
    pub ident: syn::Ident,
    pub root_path: proc_macro2::TokenStream,
    pub field_ident: syn::Ident,
    pub context_type: syn::Ident,
    pub state_type: syn::Ident,
    pub entry: Option<Box<syn::Expr>>,
    pub exit: Option<Box<syn::Expr>>,
    pub initial_transition: Option<Transition>,
    pub internal_transitions: Vec<Transition>,
    pub states: Vec<State>,
    pub regions: Vec<State>,
    pub out_transitions: Vec<Transition>,
}

pub struct Transition {
    pub event: Option<syn::Ident>,
    pub event_pat: Option<syn::Pat>,
    pub target: Option<syn::Ident>,
    pub target_state_field: Option<syn::Ident>,
    pub action: Option<Box<syn::Expr>>,
    pub guard: Option<Box<syn::Expr>>,
}

struct EventTracker {
    index: u32,
    map: HashMap<syn::Path, syn::Ident>,
}

impl EventTracker {
    pub fn new() -> Self {
        EventTracker {
            index: 0,
            map: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, path: &syn::Path) -> syn::Ident {
        if let Some(ident) = self.map.get(path) {
            return ident.clone();
        }
        self.index += 1;
        let ident = format_ident!("Internal{}", self.index);
        self.map.insert(path.clone(), ident.clone());
        ident
    }
}

pub fn lower(model: analyze::Model) -> Model {
    Model {
        machines: model.items.iter().map(lower_machine).collect(),
    }
}

fn lower_machine(machine: &analyze::Machine) -> TopMachine {
    let mod_name = format_ident!(
        "{}_machine",
        convert_case::Casing::to_case(&machine.ident.to_string(), convert_case::Case::Snake)
    );
    let mut events = EventTracker::new();
    let context;
    let mut generics = syn::Generics::default();

    let context_ident = format_ident!("{}Context", &machine.ident);
    context = Context {
        ident: context_ident.clone(),
        methods: machine.methods.clone(),
        zst: match machine.methods.is_empty() {
            true => Some(format_ident!("{}Dummy", &context_ident)),
            _ => None,
        },
    };

    if !machine.methods.is_empty() {
        generics.params.push_value(syn::GenericParam::Type(
            syn::parse_quote! { Context: #context_ident },
        ));
    }

    let submachine = lower_state(
        &machine.state,
        quote! { super },
        &mut events,
        &context.ident,
    );

    TopMachine {
        vis: machine.vis.clone(),
        ident: machine.ident.clone(),
        mod_name,
        events: events.map.into_iter().collect(),
        context,
        generics,
        state: submachine,
    }
}

fn state_field_ident(state: &syn::Ident) -> syn::Ident {
    format_ident!(
        "state_{}",
        convert_case::Casing::to_case(&state.to_string(), convert_case::Case::Snake)
    )
}

fn lower_state(
    state: &analyze::State,
    root_path: proc_macro2::TokenStream,
    events: &mut EventTracker,
    context: &syn::Ident,
) -> State {
    let ident = state.ident.clone();
    let mod_name = format_ident!(
        "{}_state",
        convert_case::Casing::to_case(&ident.to_string(), convert_case::Case::Snake)
    );
    let field_ident = state_field_ident(&ident);
    let state_type = format_ident!("{}State", &ident);

    let states = state
        .states
        .values()
        .map(|s| lower_state(s, quote! { #root_path::super }, events, context))
        .collect();

    let regions = state
        .regions
        .iter()
        .map(|s| lower_state(s, quote! { #root_path::super }, events, context))
        .collect();

    let initial_transition = state
        .initial_transition
        .as_ref()
        .map(|t| lower_transition(t, events));

    let internal_transitions = state
        .internal_transitions
        .iter()
        .map(|t| lower_transition(t, events))
        .collect();

    let out_transitions = state
        .out_transitions
        .iter()
        .map(|t| lower_transition(t, events))
        .collect();

    State {
        ident,
        mod_name,
        root_path,
        field_ident,
        context_type: context.clone(),
        state_type,
        entry: state.entry.clone(),
        exit: state.exit.clone(),
        internal_transitions,
        initial_transition,
        states,
        regions,
        out_transitions,
    }
}

fn lower_transition(transition: &analyze::Transition, events: &mut EventTracker) -> Transition {
    let event = transition
        .event_path
        .as_ref()
        .map(|e| events.get_or_create(e));

    let target_state_field = transition.target.as_ref().map(|t| state_field_ident(t));

    Transition {
        event,
        event_pat: transition.event_pat.clone(),
        target: transition.target.clone(),
        target_state_field,
        action: transition.action.clone(),
        guard: transition.guard.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    use super::*;

    #[test]
    fn basic() {
        let ast: parse::UmlState = syn::parse_quote! {
            machine FooBar {
                state A;
                <*> => A;
                A + E => A;
                A + E(n) => B if n > 0;
                A + E(_) => A;

                state B {
                    state A;
                    <*> => A;
                    A + E3 => A;
                }
            }
        };

        let model = analyze::analyze(ast).unwrap();
        let lowered = lower(model);

        let m = &lowered.machines[0];
        assert_eq!(m.ident, "FooBar");
        assert_eq!(m.state.field_ident, "state_foo_bar");
        assert_eq!(m.events.len(), 2);
    }
}
