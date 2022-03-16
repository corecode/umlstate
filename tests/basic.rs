use umlstate::umlstate;

struct E;

umlstate! {
    machine Basic {
        state A;
        state B;
        A + E => B / ctx.called = true;
    }
}

struct BasicContext {
    called: bool,
}

#[test]
fn basic() {
    let mut b = Basic::new(BasicContext { called: false });
    assert_eq!(b.state_config().count(), 1);
    assert!(b.state_config().any(|s| matches!(s, BasicState::A)));
    b.process(E {});
    assert!(b.state_config().any(|s| matches!(s, BasicState::B)));
    assert!(b.context.called);
}
