use crate::job::JobControl;
use crate::outbox::Outbox;
use n_mcconsole_core::config::Config;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Message;
use scene::Scene;
use std::any::Any;

pub mod action;
pub mod app;
pub mod bus;
pub mod event;
pub mod job;
pub mod outbox;
pub mod registrar;
pub mod scene;

pub struct Ctx<'a> {
    pub executor: &'a dyn Executor,
    pub config: &'a Config,
    pub jobs: &'a JobControl,
}

pub trait Handle<T: Message> {
    fn handle(&mut self, msg: &T, ctx: &Ctx, out: &mut Outbox);
}

type Thunk = fn(&mut dyn Scene, &dyn Any, &Ctx, &mut Outbox);

fn thunk<S, T>(scene: &mut dyn Scene, payload: &dyn Any, ctx: &Ctx, out: &mut Outbox)
where
    S: Scene + Handle<T>,
    T: Message,
{
    let s = scene
        .as_any_mut()
        .downcast_mut::<S>()
        .expect("thunk: scene type mismatch");
    let m = payload
        .downcast_ref::<T>()
        .expect("thunk: payload type mismatch");

    s.handle(m, ctx, out);
}
