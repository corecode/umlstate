use std::collections::{HashMap, HashSet};

use syn::Result;

use crate::parse;

pub struct Model {
    pub items: Vec<Machine>,
}

pub struct Machine {
    pub ident: syn::Ident,
    pub initial_state: Option<syn::Ident>,
    pub states: HashMap<syn::Ident, State>,
    pub events: HashSet<syn::Ident>,
}

pub struct State {
    pub ident: syn::Ident,
    pub out_transitions: Vec<OutTransition>,
}

pub struct OutTransition {
    pub event: syn::Ident,
    pub target: syn::Ident,
    pub action: Option<Box<syn::Expr>>,
}

pub fn analyze(ast: parse::UmlState) -> Result<Model> {
    Ok(Model {
        items: {
            let mut items = Vec::new();
            for item in ast.items {
                items.push(analyze_machine(item)?);
            }
            items
        },
    })
}

fn analyze_machine(machine: parse::Machine) -> Result<Machine> {
    let mut states = HashMap::new();
    let mut events = HashSet::new();
    let mut initial_state: Option<syn::Ident> = None;

    for it in &machine.items {
        if let parse::MachineItem::State(state) = it {
            initial_state.get_or_insert(state.ident.clone());
            states.insert(
                state.ident.clone(),
                State {
                    ident: state.ident.clone(),
                    out_transitions: vec![],
                },
            );
        }
    }

    for it in &machine.items {
        if let parse::MachineItem::Transition(transition) = it {
            if !states.contains_key(&transition.target) {
                return Err(syn::Error::new_spanned(
                    &transition.target,
                    "transition target is not a declared state",
                ));
            }
            let state = states.get_mut(&transition.source).ok_or_else(|| {
                syn::Error::new_spanned(
                    &transition.source,
                    "transition source is not a declared state",
                )
            })?;
            events.insert(transition.event.clone());
            state.out_transitions.push(OutTransition {
                target: transition.target.clone(),
                event: transition.event.clone(),
                action: transition.action.as_ref().map(|(_, a)| a.expr.clone()),
            })
        }
    }

    Ok(Machine {
        ident: machine.ident,
        initial_state,
        states,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let ast: parse::UmlState = syn::parse_quote! {
            machine Foo {
                state A;
                A + E => A;
            }
        };

        let model = analyze(ast).unwrap();
        assert_eq!(model.items.len(), 1);
        let m = &model.items[0];
        assert_eq!(m.ident, "Foo");
        assert_eq!(m.states.len(), 1);
        let s = &m.states.values().next().unwrap();
        assert_eq!(s.ident, "A");
        assert_eq!(s.out_transitions.len(), 1);
        let t = &s.out_transitions[0];
        assert_eq!(t.event, "E");
        assert_eq!(t.target, "A");
    }
}
