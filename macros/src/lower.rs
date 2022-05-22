use std::collections::HashMap;

use quote::format_ident;

use crate::analyze;

pub struct Model {
    pub machines: Vec<TopMachine>,
}

pub struct TopMachine {
    pub events: Vec<(syn::Path, syn::Ident)>,
    pub machine: SubMachine,
}

pub struct SubMachine {
    pub vis: syn::Visibility,
    pub type_ident: syn::Ident,
    pub field_ident: syn::Ident,
    pub generics: syn::Generics,
    pub context_type: Option<syn::Ident>,
    pub state_type: syn::Ident,
    pub initial_state: syn::Ident,
    pub states: Vec<State>,
    pub machines: Vec<SubMachine>,
}

pub struct State {
    pub ident: syn::Ident,
    pub submachine_field: Option<syn::Ident>,
    pub out_transitions: Vec<OutTransition>,
}

pub struct OutTransition {
    pub event: syn::Ident,
    pub event_pat: Option<syn::Pat>,
    pub target: syn::Ident,
    pub action: Option<Box<syn::Expr>>,
    pub guard: Option<Box<syn::Expr>>,
    pub target_machine: Option<syn::Ident>,
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
        let ident = quote::format_ident!("EventInternal{}", self.index);
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
    let mut events = EventTracker::new();
    let submachine = lower_submachine(machine, &mut events, "");

    TopMachine {
        events: events.map.into_iter().collect(),
        machine: submachine,
    }
}

fn state_field_ident(state: &syn::Ident) -> syn::Ident {
    quote::format_ident!(
        "machine_{}",
        convert_case::Casing::to_case(&state.to_string(), convert_case::Case::Snake)
    )
}

fn lower_submachine(
    machine: &analyze::Machine,
    events: &mut EventTracker,
    prefix: &str,
) -> SubMachine {
    let type_ident = machine.ident.clone();
    let field_ident = state_field_ident(&type_ident);
    let context_type = machine.context.clone();
    let state_type = format_ident!("{}{}State", prefix, &type_ident);
    let mut generics = machine.generics.clone();

    if let Some(ctx) = &context_type {
        generics.params.push_value(syn::GenericParam::Type(
            syn::parse_quote! { __ContextT: #ctx },
        ));
    }

    let machines = machine
        .machines
        .values()
        .map(|m| lower_submachine(m, events, format!("{}{}", prefix, &type_ident).as_str()))
        .collect();

    let states = machine
        .states
        .values()
        .map(|v| State {
            ident: v.ident.clone(),
            submachine_field: match v.is_machine {
                false => None,
                true => Some(state_field_ident(&v.ident)),
            },
            out_transitions: v
                .out_transitions
                .iter()
                .map(|t| {
                    let mut t = lower_transition(t, events);
                    if let Some(analyze::State {
                        is_machine: true, ..
                    }) = machine.states.get(&t.target)
                    {
                        t.target_machine = Some(state_field_ident(&t.target))
                    }
                    t
                })
                .collect(),
        })
        .collect();

    SubMachine {
        vis: machine.vis.clone(),
        type_ident,
        field_ident,
        generics,
        context_type,
        state_type,
        initial_state: machine.initial_state.clone(),
        states,
        machines,
    }
}

fn lower_transition(
    transition: &analyze::OutTransition,
    events: &mut EventTracker,
) -> OutTransition {
    let event = events.get_or_create(&transition.event_path);

    OutTransition {
        event,
        event_pat: transition.event_pat.clone(),
        target: transition.target.clone(),
        action: transition.action.clone(),
        guard: transition.guard.clone(),
        target_machine: None,
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
                A + E => A;
                A + E(n) => B if n > 0;
                A + E(_) => A;

                machine B {
                    state A;
                    A + E3 => A;
                }
            }
        };

        let model = analyze::analyze(ast).unwrap();
        let lowered = lower(model);

        let m = &lowered.machines[0];
        assert_eq!(m.machine.type_ident, "FooBar");
        assert_eq!(m.machine.field_ident, "machine_foo_bar");
        assert_eq!(m.events.len(), 2);
    }
}
