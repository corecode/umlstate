use umlstate::*;

#[derive(Clone)]
struct E(u32);
#[derive(Clone)]
struct E2;
#[derive(Clone)]
struct E3;

umlstate! {
    machine Basic {
        state A;
        state C;

        A + E(n) => B / ctx.called = true
            if n > 0;
        B + E2 => C;

        machine B {
            state A;
            state X;

            A + E3 => X;
        }
    }
}

struct BasicContext {
    called: bool,
}

#[test]
fn basic() {
    let mut b = Basic::new(BasicContext { called: false });
    b.start();
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
    b.process(E3 {});
    assert!(b.state_config().any(|s| matches!(s, BasicState::B)));
    b.process(E3 {});
    assert!(b.state_config().any(|s| matches!(s, BasicState::B)));
    b.process(E2 {});
    assert!(b.state_config().any(|s| matches!(s, BasicState::C)));
}
