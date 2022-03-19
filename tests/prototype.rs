use umlstate::EventProcessor;

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

        fn process_internal(&mut self, event: Event) -> umlstate::ProcessResult {
            let ctx = &self.context;
            match self.state {
                State::State1 => match event {
                    Event::EventA(_event) => {
                        self.state = State::State2;
                        umlstate::ProcessResult::Handled
                    }
                    _ => umlstate::ProcessResult::Unhandled,
                },
                State::State2 => match event {
                    Event::EventB(_event @ EventB(n)) if ctx.is_even_p(n) => {
                        let ctx = &mut self.context;
                        ctx.on_b();
                        self.state = State::State1;
                        umlstate::ProcessResult::Handled
                    }
                    _ => umlstate::ProcessResult::Unhandled,
                },
            }
        }
    }

    impl EventProcessor<EventA> for Machine {
        fn process(&mut self, event: EventA) -> umlstate::ProcessResult {
            self.process_internal(Event::EventA(event))
        }
    }

    impl EventProcessor<EventB> for Machine {
        fn process(&mut self, event: EventB) -> umlstate::ProcessResult {
            self.process_internal(Event::EventB(event))
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
    let r = m.process(EventB(2));
    assert_eq!(r, umlstate::ProcessResult::Unhandled);
    m.state_config()
        .find(|s| matches!(s, MyMachineState::State1))
        .unwrap();
    let r = m.process(EventA {});
    assert_eq!(r, umlstate::ProcessResult::Handled);
    m.state_config()
        .find(|s| matches!(s, MyMachineState::State2))
        .unwrap();
    m.process(EventB(1));
    m.process(EventB(4));
}
