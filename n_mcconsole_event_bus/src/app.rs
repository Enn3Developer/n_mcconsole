use crate::Ctx;
use crate::action::{Action, KeyFlow};
use crate::bus::Bus;
use crate::job::JobControl;
use crate::outbox::Outbox;
use crate::registrar::Registrar;
use crate::scene::{Scene, SceneId};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use n_mcconsole_core::config::Config;
use n_mcconsole_core::message::Envelope;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::collections::HashMap;

/// The central app manager, handles scenes, input and has a central bus system to deliver messages to subscribers
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

    /// Push messages to the event bus to later be delivered
    pub fn enqueue(&mut self, env: Envelope) {
        self.bus.queue.push_back(env);
    }

    /// Register a scene
    ///
    /// If you want to register a scene and render it, use [App::push_scene] instead
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

    /// Register a scene and push it onto the stack to render it
    pub fn push_scene<S: Scene>(&mut self, scene: S) -> SceneId {
        let id = self.register_scene(scene);
        self.order.push(id);
        id
    }

    /// Remove a scene and all its children scenes
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

    /// Close a scene and remove it together with its children scenes
    pub fn close_scene(&mut self, id: SceneId) {
        self.order.retain(|x| *x != id);
        self.remove_recursive(id);
    }

    /// Push a key event to be handled by the app
    pub fn handle_input(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if let (true, KeyCode::Char('c')) = (ctrl, key.code) {
            self.apply(SceneId(0), Action::Quit);
            return;
        }

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

    /// Reads all the messages in the bus and delivers to subscribers
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

                // &* here means to dereference through the Box so we get the internal data
                t(s.as_mut(), &*envelope.payload, &ctx, &mut out);
            }

            queue.extend(out.pending.drain(..));
        }
    }

    /// Render all the scenes in the stack
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
