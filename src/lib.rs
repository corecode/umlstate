pub use umlstate_macros::umlstate;

pub trait EventProcessor<E> {
    fn process(&mut self, event: E) -> ProcessResult;
}

#[derive(Debug, PartialEq)]
pub enum ProcessResult {
    Handled,
    Unhandled,
}
