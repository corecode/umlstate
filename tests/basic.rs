use umlstate::umlstate;

umlstate! {
    machine Basic {
    }
}

struct BasicContext;

fn main() {
    let _b = Basic::new(BasicContext);
}
