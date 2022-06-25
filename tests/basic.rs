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
    pub machine Basic {
        fn called(&self);
        fn internal(&self, n: u32);

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

struct BasicContextData {
    pub called: bool,
    pub n: Option<u32>,
}

struct BasicContextImpl {
    pub data: Rc<RefCell<BasicContextData>>,
}

impl BasicContext for BasicContextImpl {
    fn called(&self) {
        self.data.borrow_mut().called = true;
    }

    fn internal(&self, n: u32) {
        self.data.borrow_mut().n = Some(n);
    }
}

#[test]
fn basic() {
    let data = Rc::new(RefCell::new(BasicContextData {
        called: false,
        n: None,
    }));
    let mut b = Basic::new(BasicContextImpl { data: data.clone() });
    b.enter();
    assert!(b.state() == Some(BasicState::A));
    let r = b.process(E2);
    assert_eq!(r, ProcessResult::Unhandled);
    let r = b.process(E(0));
    assert_eq!(r, ProcessResult::Handled);
    assert!(b.state() == Some(BasicState::A));
    assert!(!data.borrow().called);
    assert_eq!(data.borrow().n, Some(0));
    let r = b.process(E(5));
    assert_eq!(r, ProcessResult::Handled);
    assert!(b.state() == Some(BasicState::B));
    assert!(data.borrow().called);
    b.process(E3 {});
    assert!(b.state() == Some(BasicState::B));
    b.process(E3 {});
    assert!(b.state() == Some(BasicState::B));
    b.process(E2 {});
    assert!(b.state() == Some(BasicState::C));
}
