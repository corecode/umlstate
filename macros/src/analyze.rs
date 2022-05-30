use std::collections::HashMap;

use syn::Result;

use crate::parse;

pub struct Model {
    pub items: Vec<Machine>,
}

pub struct Machine {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub context: Option<syn::Ident>,
    pub state: State,
}

pub struct State {
    pub ident: syn::Ident,
    pub states: HashMap<syn::Ident, State>,
    pub initial_transition: Option<Transition>,
    pub out_transitions: Vec<Transition>,
}

pub struct Transition {
    pub event_path: Option<syn::Path>,
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
    let mut context: Option<syn::Ident> = None;
    let mut items = vec![];

    for it in &machine.items {
        match it {
            parse::MachineItem::Context(c) => {
                if context.is_some() {
                    return Err(syn::Error::new_spanned(
                        &c,
                        "duplicate declaration of context",
                    ));
                }
                context = Some(c.ident.clone());
            }
            parse::MachineItem::StateItem(i) => items.push(i.clone()),
        }
    }

    Ok(Machine {
        vis: machine.vis.clone(),
        ident: machine.ident.clone(),
        context,
        state: analyze_state(machine.ident.clone(), &items, &machine)?,
    })
}

fn analyze_state(
    ident: syn::Ident,
    items: &Vec<parse::StateItem>,
    range: &dyn quote::ToTokens,
) -> Result<State> {
    let mut states = HashMap::new();
    let mut initial_transition = None;

    for it in items {
        match it {
            parse::StateItem::State(sub_state) => {
                let old = states.insert(
                    sub_state.ident.clone(),
                    analyze_state(sub_state.ident.clone(), &sub_state.items, &sub_state)?,
                );
                if old.is_some() {
                    return Err(syn::Error::new_spanned(
                        &sub_state.ident,
                        "duplicate declaration of state",
                    ));
                }
            }
            parse::StateItem::Transition(_) => (),
        }
    }

    for it in items {
        match it {
            parse::StateItem::Transition(
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
                    action: transition.action.as_ref().map(|(_, a)| a.expr.clone()),
                    event_pat: None,
                    event_path: None,
                    guard: None,
                })
            }
            parse::StateItem::Transition(
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

                let sub_state = states.get_mut(source).ok_or_else(|| {
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

                sub_state.out_transitions.push(Transition {
                    target: transition.target.clone(),
                    event_path: Some(event_path),
                    event_pat,
                    action: transition.action.as_ref().map(|(_, a)| a.expr.clone()),
                    guard: transition.guard.as_ref().map(|(_, g)| g.expr.clone()),
                })
            }
            parse::StateItem::State(_) => (),
        }
    }

    if states.len() > 0 && initial_transition.is_none() {
        return Err(syn::Error::new_spanned(
            range,
            "missing initial transition. help: you need one `<*> =>` transition",
        ));
    }

    Ok(State {
        ident,
        states,
        initial_transition,
        out_transitions: vec![],
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

                state B {
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
        assert_eq!(m.state.states.len(), 2);

        let s = m.state.states.values().find(|s| s.ident == "A").unwrap();
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
