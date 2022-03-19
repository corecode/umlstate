use umlstate::*;

struct E(u32);
struct E2;

umlstate! {
    machine Basic {
        state A;
        state B;
        state C
        A + E(n) => B / ctx.called = true
            if n > 0;
        B + E2 => C;
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
    let r = b.process(E2);
    assert_eq!(r, ProcessResult::Unhandled);
    let r = b.process(E(0));
    assert_eq!(r, ProcessResult::Unhandled);
    assert!(b.state_config().any(|s| matches!(s, BasicState::A)));
    assert!(!b.context.called);
    let r = b.process(E(5));
    assert_eq!(r, ProcessResult::Handled);
    assert!(b.state_config().any(|s| matches!(s, BasicState::B)));
    assert!(b.context.called);
    b.process(E2 {});
    assert!(b.state_config().any(|s| matches!(s, BasicState::C)));
}
