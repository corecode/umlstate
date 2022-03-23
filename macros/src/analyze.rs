use std::collections::HashMap;

use syn::Result;

use crate::parse;

pub struct Model {
    pub items: Vec<Machine>,
}

pub struct Machine {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub initial_state: syn::Ident,
    pub states: HashMap<syn::Ident, State>,
    pub machines: HashMap<syn::Ident, Machine>,
}

pub struct State {
    pub ident: syn::Ident,
    pub is_machine: bool,
    pub out_transitions: Vec<OutTransition>,
}

pub struct OutTransition {
    pub event_path: syn::Path,
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
                items.push(analyze_machine(&item)?);
            }
            items
        },
    })
}

fn analyze_machine(machine: &parse::Machine) -> Result<Machine> {
    let mut states = HashMap::new();
    let mut machines = HashMap::new();
    let mut initial_state: Option<syn::Ident> = None;

    for it in &machine.items {
        match it {
            parse::MachineItem::State(state) => {
                initial_state.get_or_insert(state.ident.clone());
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
                initial_state.get_or_insert(machine.ident.clone());
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
            parse::MachineItem::Transition(_) => (),
        }
    }

    for it in &machine.items {
        match it {
            parse::MachineItem::Transition(transition) => {
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

                state.out_transitions.push(OutTransition {
                    target: transition.target.clone(),
                    event_path,
                    event_pat,
                    action: transition.action.as_ref().map(|(_, a)| a.expr.clone()),
                    guard: transition.guard.as_ref().map(|(_, g)| g.expr.clone()),
                })
            }
            parse::MachineItem::State(_) => (),
            parse::MachineItem::Machine(_) => (),
        }
    }

    Ok(Machine {
        ident: machine.ident.clone(),
        generics: machine.generics.clone(),
        initial_state: initial_state
            .ok_or_else(|| syn::Error::new_spanned(&machine, "no initial state declared"))?,
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
                A + E => A;
                A + E(n) => B if n > 0;
                A + E(_) => A;

                machine B {
                    state A;
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
