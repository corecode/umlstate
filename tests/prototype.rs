trait EventProcessor<E> {
    fn process(&mut self, event: E);
}

struct EventA;
struct EventB(u32);

mod MyMachineMod {
    use super::*;

    pub enum State {
        State1,
        State2,
    }

    enum Event {
        EventA(EventA),
        EventB(EventB),
    }

    pub(crate) struct Machine {
        context: MyMachineContext,
        state: State,
    }

    impl Machine {
        pub fn new(context: MyMachineContext) -> Self {
            Machine {
                context,
                state: State::State1,
            }
        }

        fn process_internal(&mut self, event: Event) {
            match self.state {
                State::State1 => match &event {
                    Event::EventA(_event) => {
                        self.state = State::State2;
                    }
                    _ => (),
                },
                State::State2 => match &event {
                    Event::EventB(event) => {
                        let ctx = &self.context;
                        if ctx.is_even_p(event) {
                            let ctx = &mut self.context;
                            ctx.on_b();
                            self.state = State::State1;
                        }
                    }
                    _ => (),
                },
            }
        }
    }

    impl EventProcessor<EventA> for Machine {
        fn process(&mut self, event: EventA) {
            self.process_internal(Event::EventA(event));
        }
    }

    impl EventProcessor<EventB> for Machine {
        fn process(&mut self, event: EventB) {
            self.process_internal(Event::EventB(event));
        }
    }
}

use MyMachineMod::Machine as MyMachine;

struct MyMachineContext;

impl MyMachineContext {
    fn on_b(&mut self) {
        eprintln!("got event B");
    }
    fn is_even_p(&self, event: &EventB) -> bool {
        event.0 % 2 == 0
    }
}

#[test]
fn prototype() {
    let mut m = MyMachine::new(MyMachineContext {});
    m.process(EventB(2));
    m.process(EventA {});
    m.process(EventB(1));
    m.process(EventB(4));
}
