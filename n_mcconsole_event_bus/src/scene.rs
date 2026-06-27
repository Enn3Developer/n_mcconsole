use crate::Ctx;
use crate::action::{Action, KeyFlow};
use crate::app::App;
use crate::outbox::Outbox;
use crate::registrar::Registrar;
use crossterm::event::KeyEvent;
use n_mcconsole_core::AsAny;
use ratatui::Frame;
use ratatui::layout::Rect;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SceneId(pub u64);

pub type SceneSpawner = Box<dyn FnOnce(&mut App)>;

pub trait Scene: AsAny + 'static {
    fn register(registrar: &mut Registrar<Self>)
    where
        Self: Sized;

    fn handle_key(&mut self, key: KeyEvent, ctx: &Ctx, out: &mut Outbox) -> Action;

    fn view(&self, f: &mut Frame, area: Rect, ctx: &Ctx);

    fn on_mount(&mut self, _ctx: &Ctx, _out: &mut Outbox) {}

    fn intercept_key(&mut self, _key: KeyEvent) -> KeyFlow {
        KeyFlow::Pass
    }

    fn active_child(&self) -> Option<SceneId> {
        None
    }

    fn child_areas(&self, _area: Rect) -> Vec<(SceneId, Rect)> {
        Vec::new()
    }

    fn child_ids(&self) -> Vec<SceneId> {
        Vec::new()
    }
}
