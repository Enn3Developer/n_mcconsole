use crossterm::event::KeyEvent;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::action::Action;
use n_mcconsole_event_bus::outbox::Outbox;
use n_mcconsole_event_bus::registrar::Registrar;
use n_mcconsole_event_bus::scene::Scene;
use n_mcconsole_event_bus::{Ctx, Handle};
use n_mcconsole_jobs::mc_command_sender::{McCommandSenderJob, McCommandSentMessage};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::List;

pub struct TestScene;

impl Scene for TestScene {
    fn register(registrar: &mut Registrar<Self>) {
        McCommandSenderJob::subscribe(registrar);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &Ctx, out: &mut Outbox) -> Action {
        Action::None
    }

    fn view(&self, f: &mut Frame, area: Rect, ctx: &Ctx) {
        let Ok(files) = ctx.executor.list_dir(".") else {
            return;
        };

        f.render_widget(List::new(files), area);
    }
}

impl Handle<Tagged<McCommandSentMessage>> for TestScene {
    fn handle(&mut self, msg: &Tagged<McCommandSentMessage>, ctx: &Ctx, out: &mut Outbox) {
        let Some(msg) = msg.open(0) else {
            return;
        };
    }
}
