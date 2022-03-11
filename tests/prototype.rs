trait EventProcessor<E> {
    fn process(&mut self, event: E);
}

struct EventA;
struct EventB(u32);

enum MyMachineState {
    State1,
    State2,
}

enum MyMachineEvent {
    EventA(EventA),
    EventB(EventB),
}

struct MyMachine {
    context: MyMachineContext,
    state: MyMachineState,
}

impl MyMachine {
    pub fn new(context: MyMachineContext) -> Self {
        MyMachine {
            context,
            state: MyMachineState::State1,
        }
    }

    fn process_internal(&mut self, event: MyMachineEvent) {
        match self.state {
            MyMachineState::State1 => match &event {
                MyMachineEvent::EventA(_event) => {
                    self.state = MyMachineState::State2;
                }
                _ => (),
            },
            MyMachineState::State2 => match &event {
                MyMachineEvent::EventB(event) => {
                    let ctx = &self.context;
                    if ctx.is_even_p(event) {
                        let ctx = &mut self.context;
                        ctx.on_b();
                        self.state = MyMachineState::State1;
                    }
                }
                _ => (),
            },
        }
    }
}

impl EventProcessor<EventA> for MyMachine {
    fn process(&mut self, event: EventA) {
        self.process_internal(MyMachineEvent::EventA(event));
    }
}

impl EventProcessor<EventB> for MyMachine {
    fn process(&mut self, event: EventB) {
        self.process_internal(MyMachineEvent::EventB(event));
    }
}

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
