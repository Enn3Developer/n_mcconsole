use crate::scene::Scene;
use crate::{Handle, Thunk, thunk};
use n_mcconsole_core::message::Message;
use std::any::TypeId;
use std::marker::PhantomData;

pub struct Registrar<S> {
    pub(crate) subs: Vec<(TypeId, Thunk)>,
    _s: PhantomData<S>,
}

impl<S: Scene> Registrar<S> {
    pub(crate) fn new() -> Self {
        Registrar {
            subs: vec![],
            _s: PhantomData,
        }
    }

    pub fn on<T: Message>(&mut self)
    where
        S: Handle<T>,
    {
        self.subs.push((TypeId::of::<T>(), thunk::<S, T>));
    }
}
