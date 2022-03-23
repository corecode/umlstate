use umlstate::*;

#[derive(Clone)]
struct E;

umlstate! {
    machine Generics<'a> {
        state A;
        A + E => A / *ctx.called = true;
    }
}

struct GenericsContext<'a> {
    called: &'a mut bool,
}

#[test]
fn basic() {
    let mut called = false;
    let mut b = Generics::new(GenericsContext {
        called: &mut called,
    });
    b.process(E);
    assert!(*b.context.called);
}
