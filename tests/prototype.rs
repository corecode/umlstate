use umlstate::EventProcessor;

#[derive(Clone)]
struct EventA;

#[derive(Clone)]
struct EventB(u32);

#[derive(Clone)]
struct EventC;

mod mymachine_mod {
    use super::*;

    #[derive(Clone)]
    enum Event {
        EventA(EventA),
        EventB(EventB),
        EventC(EventC),
    }

    pub enum MyMachineState {
        __NotStarted,
        __Exited,
        State1,
        State2,
        SubMachine1,
    }

    struct MyMachineImpl {
        state: MyMachineState,
        submachine1: SubMachine1Impl,
    }

    impl MyMachineImpl {
        fn new() -> Self {
            MyMachineImpl {
                state: MyMachineState::__NotStarted,
                submachine1: SubMachine1Impl::new(),
            }
        }
        fn state_config(&self) -> std::vec::IntoIter<&MyMachineState> {
            vec![&self.state].into_iter()
        }

        fn process_internal(
            &mut self,
            mut_ctx: &mut MyMachineContext,
            event: Event,
        ) -> umlstate::ProcessResult {
            let ctx: &MyMachineContext = mut_ctx;
            match self.state {
                MyMachineState::State1 => match event {
                    Event::EventA(_event) => {
                        self.state = MyMachineState::State2;
                        umlstate::ProcessResult::Handled
                    }
                    _ => umlstate::ProcessResult::Unhandled,
                },
                MyMachineState::State2 => match event {
                    Event::EventA(_event) => {
                        self.state = MyMachineState::SubMachine1;
                        self.submachine1.enter(mut_ctx);
                        umlstate::ProcessResult::Handled
                    }
                    Event::EventB(_event @ EventB(n)) if ctx.is_even_p(n) => {
                        let ctx = mut_ctx;
                        ctx.on_b(n);
                        self.state = MyMachineState::State1;
                        umlstate::ProcessResult::Handled
                    }
                    _ => umlstate::ProcessResult::Unhandled,
                },
                MyMachineState::SubMachine1 => {
                    match self.submachine1.process_internal(mut_ctx, event.clone()) {
                        umlstate::ProcessResult::Handled => umlstate::ProcessResult::Handled,
                        umlstate::ProcessResult::Unhandled => match event {
                            Event::EventA(_event) => {
                                self.submachine1.exit(mut_ctx);
                                self.state = MyMachineState::State1;
                                umlstate::ProcessResult::Handled
                            }
                            _ => umlstate::ProcessResult::Unhandled,
                        },
                    }
                }
                MyMachineState::__NotStarted | MyMachineState::__Exited => {
                    panic!("MyMachine received event while in invalid state")
                }
            }
        }

        fn enter(&mut self, _ctx: &mut MyMachineContext) {
            self.state = MyMachineState::State1;
        }

        fn exit(&mut self, _ctx: &mut MyMachineContext) {
            self.state = MyMachineState::__Exited;
        }
    }

    pub enum SubMachine1State {
        __NotStarted,
        __Exited,
        StateA,
        StateB,
    }

    struct SubMachine1Impl {
        state: SubMachine1State,
    }

    impl SubMachine1Impl {
        fn new() -> Self {
            SubMachine1Impl {
                state: SubMachine1State::__NotStarted,
            }
        }

        fn state_config(&self) -> std::vec::IntoIter<&SubMachine1State> {
            vec![&self.state].into_iter()
        }

        fn enter(&mut self, _ctx: &mut MyMachineContext) {
            self.state = SubMachine1State::StateA;
        }

        fn exit(&mut self, _ctx: &mut MyMachineContext) {
            self.state = SubMachine1State::__Exited;
        }

        fn process_internal(
            &mut self,
            mut_ctx: &mut MyMachineContext,
            event: Event,
        ) -> umlstate::ProcessResult {
            let _ctx: &MyMachineContext = mut_ctx;
            match self.state {
                SubMachine1State::StateA => match event {
                    Event::EventC(_event) => {
                        self.state = SubMachine1State::StateB;
                        umlstate::ProcessResult::Handled
                    }
                    _ => umlstate::ProcessResult::Unhandled,
                },
                SubMachine1State::StateB => match event {
                    _ => umlstate::ProcessResult::Unhandled,
                },
                SubMachine1State::__NotStarted | SubMachine1State::__Exited => {
                    panic!("SubMachine1 received event while in invalid state")
                }
            }
        }
    }

    pub(crate) struct Machine<'a> {
        pub context: MyMachineContext<'a>,
        machine: MyMachineImpl,
    }

    impl<'a> Machine<'a> {
        pub fn new(context: MyMachineContext<'a>) -> Self {
            Machine {
                context,
                machine: MyMachineImpl::new(),
            }
        }

        pub fn start(&mut self) {
            self.machine.enter(&mut self.context);
        }

        pub fn state_config(&self) -> std::vec::IntoIter<&MyMachineState> {
            self.machine.state_config()
        }
    }

    impl<'a> EventProcessor<EventA> for Machine<'a> {
        fn process(&mut self, event: EventA) -> umlstate::ProcessResult {
            self.machine
                .process_internal(&mut self.context, Event::EventA(event))
        }
    }

    impl<'a> EventProcessor<EventB> for Machine<'a> {
        fn process(&mut self, event: EventB) -> umlstate::ProcessResult {
            self.machine
                .process_internal(&mut self.context, Event::EventB(event))
        }
    }

    impl<'a> EventProcessor<EventC> for Machine<'a> {
        fn process(&mut self, event: EventC) -> umlstate::ProcessResult {
            self.machine
                .process_internal(&mut self.context, Event::EventC(event))
        }
    }
}

use mymachine_mod::Machine as MyMachine;
use mymachine_mod::MyMachineState;

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
    m.start();
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

    m.state_config()
        .find(|s| matches!(s, MyMachineState::State1))
        .unwrap();
    m.process(EventA {});
    m.state_config()
        .find(|s| matches!(s, MyMachineState::State2))
        .unwrap();
    m.process(EventA {});
    m.state_config()
        .find(|s| matches!(s, MyMachineState::SubMachine1))
        .unwrap();
    // m.state_config()
    //     .find(|s| matches!(s, SubMachine1State::StateA))
    //     .unwrap();
    m.process(EventC {});
    m.state_config()
        .find(|s| matches!(s, MyMachineState::SubMachine1))
        .unwrap();
    // m.state_config()
    //     .find(|s| matches!(s, SubMachine1State::StateB))
    //     .unwrap();
    m.process(EventA {});
    m.state_config()
        .find(|s| matches!(s, MyMachineState::State1))
        .unwrap();

    assert_eq!(data, 4);
}
