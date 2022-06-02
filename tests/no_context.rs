use umlstate::*;

#[derive(Clone)]
struct E(u32);
#[derive(Clone)]
struct E2;
#[derive(Clone)]
struct E3;

umlstate! {
    pub(crate) machine NoContext {
        state A;
        state C;

        <*> => A;
        A + E(n) => B if n > 0;
        B + E2 => C;

        state B {
            state A;
            state X;

            <*> => A;
            A + E3 => X;
        }
    }
}

#[test]
fn no_context() {
    let mut b = NoContext::new();
    b.enter();
    assert!(b.state() == Some(NoContextState::A));
    let r = b.process(E2);
    assert_eq!(r, ProcessResult::Unhandled);
    let r = b.process(E(0));
    assert_eq!(r, ProcessResult::Unhandled);
    assert!(b.state() == Some(NoContextState::A));
    let r = b.process(E(5));
    assert_eq!(r, ProcessResult::Handled);
    assert!(b.state() == Some(NoContextState::B));
    b.process(E3 {});
    assert!(b.state() == Some(NoContextState::B));
    b.process(E3 {});
    assert!(b.state() == Some(NoContextState::B));
    b.process(E2 {});
    assert!(b.state() == Some(NoContextState::C));
}
