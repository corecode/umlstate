use umlstate::umlstate;

umlstate! {
    machine Foo {
        state a;
        A + E => B;
    }
}

fn main() {}
