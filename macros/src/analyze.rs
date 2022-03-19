use std::collections::HashMap;

use quote::format_ident;
use syn::Result;

use crate::parse;

pub struct Model {
    pub items: Vec<Machine>,
}

pub struct Machine {
    pub ident: syn::Ident,
    pub initial_state: Option<syn::Ident>,
    pub states: HashMap<syn::Ident, State>,
    pub events: HashMap<syn::Path, syn::Ident>,
}

pub struct State {
    pub ident: syn::Ident,
    pub out_transitions: Vec<OutTransition>,
}

pub struct OutTransition {
    pub event: syn::Ident,
    pub event_pat: Option<syn::Pat>,
    pub target: syn::Ident,
    pub action: Option<Box<syn::Expr>>,
    pub guard: Option<Box<syn::Expr>>,
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
    let mut events = HashMap::new();
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

    let mut event_id: u32 = 0;
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

            let event_path = match &transition.event.pat {
                syn::Pat::Path(p) => p.path.clone(),
                syn::Pat::Struct(s) => s.path.clone(),
                syn::Pat::TupleStruct(ts) => ts.path.clone(),
                syn::Pat::Ident(i) => syn::Path {
                    leading_colon: None,
                    segments: {
                        let mut segments = syn::punctuated::Punctuated::new();
                        segments.push_value(syn::PathSegment {
                            ident: i.ident.clone(),
                            arguments: syn::PathArguments::None,
                        });
                        segments
                    },
                },
                _ => panic!("parsed invalid event pattern"),
            };
            let event_pat = match &transition.event.pat {
                syn::Pat::Ident(_) => None,
                _ => Some(transition.event.pat.clone()),
            };

            let internal_event = events
                .get(&event_path)
                .cloned()
                .or_else(|| {
                    let new_id = format_ident!("EventInternal_{}", event_id);
                    event_id += 1;
                    events.insert(event_path.clone(), new_id.clone());
                    Some(new_id)
                })
                .unwrap();

            state.out_transitions.push(OutTransition {
                target: transition.target.clone(),
                event: internal_event.clone(),
                event_pat,
                action: transition.action.as_ref().map(|(_, a)| a.expr.clone()),
                guard: transition.guard.as_ref().map(|(_, g)| g.expr.clone()),
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
                A + E(_) => A;
            }
        };

        let model = analyze(ast).unwrap();

        assert_eq!(model.items.len(), 1);

        let m = &model.items[0];
        assert_eq!(m.ident, "Foo");
        assert_eq!(m.events.len(), 1);
        assert_eq!(m.states.len(), 1);

        let s = m.states.values().next().unwrap();
        assert_eq!(s.ident, "A");
        assert_eq!(s.out_transitions.len(), 2);

        let (e_path, e_ident) = m.events.iter().next().unwrap();
        assert_eq!(e_path.get_ident().unwrap(), "E");

        let t = &s.out_transitions[0];
        assert_eq!(t.target, "A");
        assert_eq!(e_ident, &t.event);
        assert!(matches!(t.event_pat, None));

        let t = &s.out_transitions[1];
        assert_eq!(t.target, "A");

        assert!(matches!(t.event_pat, Some(syn::Pat::TupleStruct(ref p))
                     if p.path.get_ident().unwrap() == "E"));
        assert_eq!(e_ident, &t.event);
    }
}
