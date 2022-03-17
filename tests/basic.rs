use umlstate::umlstate;

struct E(bool);

umlstate! {
    machine Basic {
        state A;
        state B;
        A + E(b) => A / ctx.called = b;
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
    b.process(E(false));
    assert!(b.state_config().any(|s| matches!(s, BasicState::A)));
    assert!(!b.context.called);
    b.process(E(true));
    // assert!(b.state_config().any(|s| matches!(s, BasicState::B)));
    assert!(b.context.called);
}
