trait EventProcessor<T, E> {
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

trait MyMachineContext {
    fn on_b(&mut self);
    fn is_even_p(&self, event: &EventB) -> bool;
}

struct MyMachine<T: MyMachineContext> {
    context: T,
    state: MyMachineState,
}

impl<T: MyMachineContext> MyMachine<T> {
    pub fn new(context: T) -> Self {
        MyMachine {
            context,
            state: MyMachineState::State1,
        }
    }

    fn process_internal(&mut self, event: MyMachineEvent) {
        match self.state {
            MyMachineState::State1 => match &event {
                MyMachineEvent::EventA(event) => {
                    self.state = MyMachineState::State2;
                }
                _ => (),
            },
            MyMachineState::State2 => match &event {
                MyMachineEvent::EventB(event) => {
                    if self.context.is_even_p(&event) {
                        self.context.on_b();
                        self.state = MyMachineState::State1;
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

impl<T: MyMachineContext> EventProcessor<T, EventA> for MyMachine<T> {
    fn process(&mut self, event: EventA) {
        self.process_internal(MyMachineEvent::EventA(event));
    }
}

impl<T: MyMachineContext> EventProcessor<T, EventB> for MyMachine<T> {
    fn process(&mut self, event: EventB) {
        self.process_internal(MyMachineEvent::EventB(event));
    }
}

struct Context;

impl MyMachineContext for Context {
    fn on_b(&mut self) {
        eprintln!("got event B");
    }
    fn is_even_p(&self, event: &EventB) -> bool {
        event.0 % 2 == 0
    }
}

#[test]
fn prototype() {
    let mut m = MyMachine::new(Context {});
    m.process(EventB(2));
    m.process(EventA {});
    m.process(EventB(1));
    m.process(EventB(4));
}
