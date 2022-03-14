use umlstate::umlstate;

struct E;

umlstate! {
    machine Basic {
        state A;
        A + E => A;
    }
}

struct BasicContext;

fn main() {
    let mut b = Basic::new(BasicContext);
    b.process(E {});
}
