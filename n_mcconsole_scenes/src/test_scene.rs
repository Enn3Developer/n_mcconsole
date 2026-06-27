use crossterm::event::KeyEvent;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::action::Action;
use n_mcconsole_event_bus::job::RunningJob;
use n_mcconsole_event_bus::outbox::Outbox;
use n_mcconsole_event_bus::registrar::Registrar;
use n_mcconsole_event_bus::scene::Scene;
use n_mcconsole_event_bus::{Ctx, Handle};
use n_mcconsole_jobs::list_dir::{ListDirJob, ListDirMessage};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::List;

#[derive(Default)]
pub struct TestScene {
    files: Vec<String>,
    job: Option<RunningJob>,
}

impl Scene for TestScene {
    fn register(registrar: &mut Registrar<Self>) {
        ListDirJob::subscribe(registrar);
    }

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &Ctx, _out: &mut Outbox) -> Action {
        Action::None
    }

    fn view(&self, f: &mut Frame, area: Rect, _ctx: &Ctx) {
        f.render_widget(List::new(self.files.clone()), area);
    }

    fn on_mount(&mut self, ctx: &Ctx, _out: &mut Outbox) {
        self.job = Some(ctx.jobs.start(ListDirJob::new(".")));
    }
}

impl Handle<Tagged<ListDirMessage>> for TestScene {
    fn handle(&mut self, msg: &Tagged<ListDirMessage>, _ctx: &Ctx, _out: &mut Outbox) {
        let Some(job) = self.job.as_ref() else {
            return;
        };

        let Some(msg) = job.open(msg) else {
            return;
        };

        self.files = msg.files.clone();
        self.job = None;
    }
}
