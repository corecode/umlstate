use std::cell::RefCell;
use std::rc::Rc;
use umlstate::*;

#[derive(Clone)]
struct E(u32);
#[derive(Clone)]
struct E2;
#[derive(Clone)]
struct E3;

umlstate! {
    pub(crate) machine Basic {
        ctx BasicContext;
        state A;
        state C;

        state B {
            state A;
            state X;

            <*> => A;
            A + E3 => X;
        }

        <*> => A;
        A + E(n) => B / ctx.called()
            if n > 0;
        B + E2 => C;
    }
}

trait BasicContext {
    fn called(&mut self);
}

struct BasicContextImpl {
    pub called: bool,
}

impl BasicContext for BasicContextImpl {
    fn called(&mut self) {
        self.called = true;
    }
}

#[test]
fn basic() {
    let context = BasicContextImpl { called: false };
    let context = Rc::new(RefCell::new(context));
    let mut b = Basic::new(context.clone());
    b.enter();
    assert!(b.state() == &BasicState::A);
    let r = b.process(E2);
    assert_eq!(r, ProcessResult::Unhandled);
    let r = b.process(E(0));
    assert_eq!(r, ProcessResult::Unhandled);
    assert!(b.state() == &BasicState::A);
    assert!(!context.borrow().called);
    let r = b.process(E(5));
    assert_eq!(r, ProcessResult::Handled);
    assert!(b.state() == &BasicState::B);
    assert!(context.borrow().called);
    b.process(E3 {});
    assert!(b.state() == &BasicState::B);
    b.process(E3 {});
    assert!(b.state() == &BasicState::B);
    b.process(E2 {});
    assert!(b.state() == &BasicState::C);
}
