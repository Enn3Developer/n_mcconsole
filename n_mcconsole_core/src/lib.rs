use std::any::Any;

pub mod command;
pub mod config;
pub mod executor;
pub mod message;

pub trait AsAny {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
