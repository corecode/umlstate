use std::collections::HashMap;

use syn::Result;

use crate::parse;

pub struct Model {
    pub items: Vec<Machine>,
}

pub struct Machine {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub initial_transition: Transition,
    pub context: Option<syn::Ident>,
    pub states: HashMap<syn::Ident, State>,
    pub machines: HashMap<syn::Ident, Machine>,
}

pub struct State {
    pub ident: syn::Ident,
    pub is_machine: bool,
    pub out_transitions: Vec<Transition>,
}

pub struct Transition {
    pub event_path: Option<syn::Path>,
    pub event_pat: Option<syn::Pat>,
    pub target: syn::Ident,
    pub target_machine: Option<syn::Ident>,
    pub action: Option<Box<syn::Expr>>,
    pub guard: Option<Box<syn::Expr>>,
}

pub fn analyze(ast: parse::UmlState) -> Result<Model> {
    Ok(Model {
        items: {
            let mut items = Vec::new();
            for item in ast.items {
                items.push(analyze_machine(&item)?);
            }
            items
        },
    })
}

fn analyze_machine(machine: &parse::Machine) -> Result<Machine> {
    let mut states = HashMap::new();
    let mut machines = HashMap::new();
    let mut context: Option<syn::Ident> = None;
    let mut initial_transition = None;

    for it in &machine.items {
        match it {
            parse::MachineItem::State(state) => {
                let old = states.insert(
                    state.ident.clone(),
                    State {
                        ident: state.ident.clone(),
                        is_machine: false,
                        out_transitions: vec![],
                    },
                );
                if old.is_some() {
                    return Err(syn::Error::new_spanned(
                        &state.ident,
                        "duplicate declaration of state",
                    ));
                }
            }
            parse::MachineItem::Machine(machine) => {
                let old = machines.insert(machine.ident.clone(), analyze_machine(machine)?);
                if old.is_some() {
                    return Err(syn::Error::new_spanned(
                        &machine.ident,
                        "duplicate declaration of machine",
                    ));
                }
                let old = states.insert(
                    machine.ident.clone(),
                    State {
                        ident: machine.ident.clone(),
                        is_machine: true,
                        out_transitions: vec![],
                    },
                );
                if old.is_some() {
                    return Err(syn::Error::new_spanned(
                        &machine.ident,
                        "machine declared as state before",
                    ));
                }
            }
            parse::MachineItem::Context(c) => {
                if context.is_some() {
                    return Err(syn::Error::new_spanned(
                        &c,
                        "duplicate declaration of context",
                    ));
                }
                context = Some(c.ident.clone());
            }
            parse::MachineItem::Transition(_) => (),
        }
    }

    for it in &machine.items {
        match it {
            parse::MachineItem::Transition(
                transition @ parse::ItemTransition {
                    source: parse::TransitionSource::Initial(_),
                    ..
                },
            ) => {
                if !states.contains_key(&transition.target) {
                    return Err(syn::Error::new_spanned(
                        &transition.target,
                        "transition target is not a declared state",
                    ));
                }

                initial_transition = Some(Transition {
                    target: transition.target.clone(),
                    target_machine: machines
                        .get(&transition.target)
                        .as_ref()
                        .map(|m| m.ident.clone()),
                    event_path: None,
                    event_pat: None,
                    action: transition.action.as_ref().map(|(_, a)| a.expr.clone()),
                    guard: None,
                })
            }
            parse::MachineItem::Transition(
                transition @ parse::ItemTransition {
                    source: parse::TransitionSource::State(source),
                    ..
                },
            ) => {
                if !states.contains_key(&transition.target) {
                    return Err(syn::Error::new_spanned(
                        &transition.target,
                        "transition target is not a declared state",
                    ));
                }

                let state = states.get_mut(source).ok_or_else(|| {
                    syn::Error::new_spanned(
                        &transition.source,
                        "transition source is not a declared state",
                    )
                })?;

                let event = &transition
                    .event
                    .as_ref()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(&transition.source, "transition requires event")
                    })?
                    .1;
                let event_path = match &event.pat {
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
                let event_pat = match &event.pat {
                    syn::Pat::Ident(_) => None,
                    _ => Some(event.pat.clone()),
                };

                state.out_transitions.push(Transition {
                    target: transition.target.clone(),
                    target_machine: machines
                        .get(&transition.target)
                        .as_ref()
                        .map(|m| m.ident.clone()),
                    event_path: Some(event_path),
                    event_pat,
                    action: transition.action.as_ref().map(|(_, a)| a.expr.clone()),
                    guard: transition.guard.as_ref().map(|(_, g)| g.expr.clone()),
                })
            }
            parse::MachineItem::State(_) => (),
            parse::MachineItem::Machine(_) => (),
            parse::MachineItem::Context(_) => (),
        }
    }

    let initial_transition = initial_transition.ok_or_else(|| {
        syn::Error::new_spanned(
            &machine,
            "missing initial transition. help: you need one `<*> =>` transition",
        )
    })?;

    Ok(Machine {
        vis: machine.vis.clone(),
        ident: machine.ident.clone(),
        generics: machine.generics.clone(),
        context,
        initial_transition,
        states,
        machines,
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

                <*> => A / bar();
                A + E => A;
                A + E(n) => B if n > 0;
                A + E(_) => A;

                machine B {
                    state A;
                    <*> => A;
                    A + E3 => A;
                }
            }
        };

        let model = analyze(ast).unwrap();

        assert_eq!(model.items.len(), 1);

        let m = &model.items[0];
        assert_eq!(m.ident, "Foo");
        assert_eq!(m.states.len(), 2);

        let s = m.states.values().find(|s| s.ident == "A").unwrap();
        assert_eq!(s.out_transitions.len(), 3);

        let t = &s.out_transitions[0];
        assert_eq!(t.target, "A");
        assert!(matches!(t.event_pat, None));

        let t = &s.out_transitions[2];
        assert_eq!(t.target, "A");

        let t = &s.out_transitions[1];
        assert_eq!(t.target, "B");

        assert!(matches!(t.event_pat, Some(syn::Pat::TupleStruct(ref p))
                     if p.path.get_ident().unwrap() == "E"));
    }
}
