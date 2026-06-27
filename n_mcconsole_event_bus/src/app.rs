use crate::Ctx;
use crate::action::{Action, KeyFlow};
use crate::bus::Bus;
use crate::job::JobControl;
use crate::outbox::Outbox;
use crate::registrar::Registrar;
use crate::scene::{Scene, SceneId};
use crossterm::event::KeyEvent;
use n_mcconsole_core::config::Config;
use n_mcconsole_core::message::Envelope;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::collections::HashMap;

pub struct App {
    scenes: HashMap<SceneId, Box<dyn Scene>>,
    order: Vec<SceneId>,
    bus: Bus,
    config: Config,
    jobs: JobControl,
    pub running: bool,
    next_id: u64,
}

impl App {
    pub fn new(config: Config, jobs: JobControl) -> Self {
        Self {
            scenes: HashMap::new(),
            order: vec![],
            bus: Bus::default(),
            config,
            jobs,
            running: true,
            next_id: 1,
        }
    }

    pub fn next_scene_id(&mut self) -> SceneId {
        let id = SceneId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn ctx(&self) -> Ctx<'_> {
        Ctx {
            executor: self.jobs.executor.as_ref(),
            config: &self.config,
            jobs: &self.jobs,
        }
    }

    pub fn enqueue(&mut self, env: Envelope) {
        self.bus.queue.push_back(env);
    }

    pub fn register_scene<S: Scene>(&mut self, scene: S) -> SceneId {
        let id = self.next_scene_id();

        let mut registrar = Registrar::new();
        S::register(&mut registrar);

        for (tid, t) in registrar.subs {
            self.bus.subscribe(tid, id, t);
        }

        let mut boxed = Box::new(scene);
        let mut out = Outbox::default();
        boxed.on_mount(&self.ctx(), &mut out);
        self.bus.queue.extend(out.pending.drain(..));
        self.scenes.insert(id, boxed);

        id
    }

    pub fn push_scene<S: Scene>(&mut self, scene: S) -> SceneId {
        let id = self.register_scene(scene);
        self.order.push(id);
        id
    }

    fn remove_recursive(&mut self, id: SceneId) {
        let kids = self
            .scenes
            .get(&id)
            .map(|s| s.child_ids())
            .unwrap_or_default();

        for k in kids {
            self.remove_recursive(k);
        }

        self.scenes.remove(&id);
        self.bus.unsubscribe(id);
    }

    pub fn close_scene(&mut self, id: SceneId) {
        self.order.retain(|x| *x != id);
        self.remove_recursive(id);
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        let Some(&top) = self.order.last() else {
            return;
        };

        let mut id = top;
        let leaf = loop {
            match self.scenes.get_mut(&id) {
                Some(scene) => {
                    if matches!(scene.intercept_key(key), KeyFlow::Consumed) {
                        return;
                    }
                    match scene.active_child() {
                        Some(child) => id = child,
                        None => break id,
                    }
                }
                None => return,
            }
        };

        let mut out = Outbox::default();
        let action = {
            let ctx = Ctx {
                executor: self.jobs.executor.as_ref(),
                config: &self.config,
                jobs: &self.jobs,
            };
            match self.scenes.get_mut(&leaf) {
                Some(s) => s.handle_key(key, &ctx, &mut out),
                None => Action::None,
            }
        };
        self.bus.queue.extend(out.pending.drain(..));
        self.apply(leaf, action);
        self.dispatch_all();
    }

    fn apply(&mut self, actor: SceneId, action: Action) {
        match action {
            Action::None => {}
            Action::Pop => self.close_scene(actor),
            Action::Quit => self.running = false,
            Action::RunJob(cmd) => {
                let tag = self.jobs.next_id();
                self.jobs.spawn_oneshot(cmd, tag);
            }
            Action::Push(spawn) => spawn(self),
        }
    }

    pub fn dispatch_all(&mut self) {
        let App {
            scenes,
            bus,
            config,
            jobs,
            ..
        } = self;

        let Bus { index, queue } = bus;
        let config: &Config = config;
        let jobs: &JobControl = jobs;
        let ctx = Ctx {
            executor: jobs.executor.as_ref(),
            config,
            jobs,
        };

        let mut out = Outbox::default();

        while let Some(envelope) = queue.pop_front() {
            let Some(subs) = index.get(&envelope.tid) else {
                continue;
            };

            for (sid, t) in subs.iter().copied() {
                let Some(s) = scenes.get_mut(&sid) else {
                    continue;
                };

                t(s.as_mut(), &envelope.payload, &ctx, &mut out);
            }

            queue.extend(out.pending.drain(..));
        }
    }

    pub fn render(&self, f: &mut Frame) {
        let area = f.area();
        let ids = self.order.clone();
        for id in ids {
            self.render_scene(f, id, area);
        }
    }

    fn render_scene(&self, f: &mut Frame, id: SceneId, area: Rect) {
        if let Some(scene) = self.scenes.get(&id) {
            let ctx = self.ctx();
            scene.view(f, area, &ctx);
            for (child, child_area) in scene.child_areas(area) {
                self.render_scene(f, child, child_area);
            }
        }
    }
}
