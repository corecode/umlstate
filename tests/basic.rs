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

        E(n) / ctx.internal(n);
    }
}

trait BasicContext {
    fn called(&mut self);
    fn internal(&mut self, n: u32);
}

struct BasicContextImpl {
    pub called: bool,
    pub n: Option<u32>,
}

impl BasicContext for BasicContextImpl {
    fn called(&mut self) {
        self.called = true;
    }

    fn internal(&mut self, n: u32) {
        self.n = Some(n);
    }
}

#[test]
fn basic() {
    let context = BasicContextImpl {
        called: false,
        n: None,
    };
    let context = Rc::new(RefCell::new(context));
    let mut b = Basic::new(context.clone());
    b.enter();
    assert!(b.state() == &BasicState::A);
    let r = b.process(E2);
    assert_eq!(r, ProcessResult::Unhandled);
    let r = b.process(E(0));
    assert_eq!(r, ProcessResult::Handled);
    assert!(b.state() == &BasicState::A);
    assert!(!context.borrow().called);
    assert_eq!(context.borrow().n, Some(0));
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
