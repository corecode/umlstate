trait EventProcessor<E> {
    fn process(&mut self, event: E);
}

struct EventA;
struct EventB(u32);

mod mymachine_mod {
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
        pub context: MyMachineContext,
        state: State,
    }

    impl Machine {
        pub fn new(context: MyMachineContext) -> Self {
            Machine {
                context,
                state: State::State1,
            }
        }

        pub fn state_config(&self) -> std::vec::IntoIter<&State> {
            vec![&self.state].into_iter()
        }

        fn process_internal(&mut self, event: Event) {
            let ctx = &self.context;
            match self.state {
                State::State1 => match event {
                    Event::EventA(_event @ _) => {
                        self.state = State::State2;
                    }
                    _ => (),
                },
                State::State2 => match event {
                    Event::EventB(_event @ EventB(n)) if ctx.is_even_p(n) => {
                        let ctx = &mut self.context;
                        ctx.on_b();
                        self.state = State::State1;
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

use mymachine_mod::Machine as MyMachine;
use mymachine_mod::State as MyMachineState;

struct MyMachineContext;

impl MyMachineContext {
    fn on_b(&mut self) {
        eprintln!("got event B");
    }
    fn is_even_p(&self, n: u32) -> bool {
        n % 2 == 0
    }
}

#[test]
fn prototype() {
    let mut m = MyMachine::new(MyMachineContext {});
    m.process(EventB(2));
    m.state_config()
        .find(|s| matches!(s, MyMachineState::State1))
        .unwrap();
    m.process(EventA {});
    m.state_config()
        .find(|s| matches!(s, MyMachineState::State2))
        .unwrap();
    m.process(EventB(1));
    m.process(EventB(4));
}
