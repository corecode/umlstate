use std::cell::RefCell;
use std::rc::Rc;
use umlstate::*;

#[derive(Clone)]
struct E1;
#[derive(Clone)]
struct E2;

umlstate! {
    machine Regions {
        fn inc_a_e1(&mut self);
        fn inc_b_e1(&mut self);

        region A {
            state S1;
            state S2;

            <*> => S1;
            S1 + E1 => S2 / ctx.inc_a_e1();
            S2 + E2 => S1;
        }

        region B {
            state S1;
            state S2;

            <*> => S1;
            S1 + E1 => S2 / ctx.inc_b_e1();
            S2 + E2 => S1;
        }
    }
}

struct Data {
    a_e1: usize,
    b_e1: usize,
}

impl RegionsContext for Data {
    fn inc_a_e1(&mut self) {
        self.a_e1 += 1;
    }

    fn inc_b_e1(&mut self) {
        self.b_e1 += 1;
    }
}

#[test]
fn region() {
    let context = Data { a_e1: 0, b_e1: 0 };
    let context = Rc::new(RefCell::new(context));
    let mut r = Regions::new(context.clone());
    r.enter();
    let h = r.process(E2);
    assert_eq!(h, ProcessResult::Unhandled);
    assert_eq!(context.borrow().a_e1, 0);
    assert_eq!(context.borrow().b_e1, 0);
    let h = r.process(E1);
    assert_eq!(h, ProcessResult::Handled);
    assert_eq!(context.borrow().a_e1, 1);
    assert_eq!(context.borrow().b_e1, 1);
}
