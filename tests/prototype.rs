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

    pub(crate) struct Machine<'a> {
        pub context: MyMachineContext<'a>,
        state: State,
    }

    impl<'a> Machine<'a> {
        pub fn new(context: MyMachineContext<'a>) -> Self {
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
                        ctx.on_b(n);
                        self.state = State::State1;
                        umlstate::ProcessResult::Handled
                    }
                    _ => umlstate::ProcessResult::Unhandled,
                },
            }
        }
    }

    impl<'a> EventProcessor<EventA> for Machine<'a> {
        fn process(&mut self, event: EventA) -> umlstate::ProcessResult {
            self.process_internal(Event::EventA(event))
        }
    }

    impl<'a> EventProcessor<EventB> for Machine<'a> {
        fn process(&mut self, event: EventB) -> umlstate::ProcessResult {
            self.process_internal(Event::EventB(event))
        }
    }
}

use mymachine_mod::Machine as MyMachine;
use mymachine_mod::State as MyMachineState;

struct MyMachineContext<'a> {
    dataref: &'a mut u32,
}

impl<'a> MyMachineContext<'a> {
    fn on_b(&mut self, n: u32) {
        eprintln!("got event B({})", n);
        *self.dataref = n;
    }
    fn is_even_p(&self, n: u32) -> bool {
        n % 2 == 0
    }
}

#[test]
fn prototype() {
    let mut data: u32 = 0;
    let ctx = MyMachineContext { dataref: &mut data };
    let mut m = MyMachine::new(ctx);
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
    assert_eq!(data, 4);
}
