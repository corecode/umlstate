use std::cell::RefCell;
use std::rc::Rc;
use umlstate::EventProcessor;

#[derive(Clone)]
struct EventA;

#[derive(Clone)]
struct EventB(u32);

#[derive(Clone)]
struct EventC;

mod mymachine_mod {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    enum Event {
        EventA(EventA),
        EventB(EventB),
        EventC(EventC),
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum MyMachineState {
        __NotStarted,
        __Exited,
        State1,
        State2,
        SubMachine1,
    }

    pub trait MyMachineContext {
        fn on_b(&mut self, n: u32);
        fn is_even_p(&self, n: u32) -> bool;
    }

    pub struct MyMachineImpl<T: MyMachineContext> {
        context: Rc<RefCell<T>>,
        state: MyMachineState,
        sub_machine1: SubMachine1Impl<T>,
    }

    impl<T: MyMachineContext> MyMachineImpl<T> {
        pub fn new(context: Rc<RefCell<T>>) -> Self {
            MyMachineImpl {
                context: context.clone(),
                state: MyMachineState::__NotStarted,
                sub_machine1: SubMachine1Impl::new(context.clone()),
            }
        }

        pub fn state(&self) -> &MyMachineState {
            &self.state
        }

        pub fn sub_machine1(&self) -> &SubMachine1Impl<T> {
            &self.sub_machine1
        }

        fn process_event(&mut self, event: Event) -> umlstate::ProcessResult {
            let ctx = self.context.borrow();
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
                        self.sub_machine1.enter();
                        umlstate::ProcessResult::Handled
                    }
                    Event::EventB(_event @ EventB(n)) if ctx.is_even_p(n) => {
                        drop(ctx);
                        let mut ctx = self.context.borrow_mut();
                        ctx.on_b(n);
                        drop(ctx);
                        self.state = MyMachineState::State1;
                        umlstate::ProcessResult::Handled
                    }
                    _ => umlstate::ProcessResult::Unhandled,
                },
                MyMachineState::SubMachine1 => {
                    match self.sub_machine1.process_event(event.clone()) {
                        umlstate::ProcessResult::Handled => umlstate::ProcessResult::Handled,
                        umlstate::ProcessResult::Unhandled => match event {
                            Event::EventA(_event) => {
                                self.sub_machine1.exit();
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

        pub fn enter(&mut self) {
            self.state = MyMachineState::State1;
        }

        pub fn exit(&mut self) {
            self.state = MyMachineState::__Exited;
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum SubMachine1State {
        __NotStarted,
        __Exited,
        StateA,
        StateB,
    }

    pub struct SubMachine1Impl<T: MyMachineContext> {
        context: Rc<RefCell<T>>,
        state: SubMachine1State,
    }

    impl<T: MyMachineContext> SubMachine1Impl<T> {
        fn new(context: Rc<RefCell<T>>) -> Self {
            SubMachine1Impl {
                context: context.clone(),
                state: SubMachine1State::__NotStarted,
            }
        }

        pub fn state(&self) -> &SubMachine1State {
            &self.state
        }

        pub fn enter(&mut self) {
            self.state = SubMachine1State::StateA;
        }

        pub fn exit(&mut self) {
            self.state = SubMachine1State::__Exited;
        }

        fn process_event(&mut self, event: Event) -> umlstate::ProcessResult {
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

    impl<T: MyMachineContext> EventProcessor<EventA> for MyMachineImpl<T> {
        fn process(&mut self, event: EventA) -> umlstate::ProcessResult {
            self.process_event(Event::EventA(event))
        }
    }

    impl<T: MyMachineContext> EventProcessor<EventB> for MyMachineImpl<T> {
        fn process(&mut self, event: EventB) -> umlstate::ProcessResult {
            self.process_event(Event::EventB(event))
        }
    }

    impl<T: MyMachineContext> EventProcessor<EventC> for MyMachineImpl<T> {
        fn process(&mut self, event: EventC) -> umlstate::ProcessResult {
            self.process_event(Event::EventC(event))
        }
    }
}

use mymachine_mod::MyMachineContext;
use mymachine_mod::MyMachineImpl as MyMachine;
use mymachine_mod::MyMachineState;
use mymachine_mod::SubMachine1State;

struct MyMachineContextImpl<'a> {
    dataref: &'a mut u32,
}

impl<'a> MyMachineContext for MyMachineContextImpl<'a> {
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
    let ctx = MyMachineContextImpl { dataref: &mut data };
    let mut m = MyMachine::new(Rc::new(RefCell::new(ctx)));
    m.enter();
    let r = m.process(EventB(2));
    assert_eq!(r, umlstate::ProcessResult::Unhandled);
    assert_eq!(m.state(), &MyMachineState::State1);
    let r = m.process(EventA {});
    assert_eq!(r, umlstate::ProcessResult::Handled);
    assert_eq!(m.state(), &MyMachineState::State2);
    m.process(EventB(1));
    m.process(EventB(4));

    assert_eq!(m.state(), &MyMachineState::State1);
    m.process(EventA {});
    assert_eq!(m.state(), &MyMachineState::State2);
    m.process(EventA {});
    assert_eq!(m.state(), &MyMachineState::SubMachine1);
    assert_eq!(m.sub_machine1().state(), &SubMachine1State::StateA);
    m.process(EventC {});
    assert_eq!(m.state(), &MyMachineState::SubMachine1);
    assert_eq!(m.sub_machine1().state(), &SubMachine1State::StateB);
    m.process(EventA {});
    assert_eq!(m.state(), &MyMachineState::State1);

    assert_eq!(data, 4);
}
