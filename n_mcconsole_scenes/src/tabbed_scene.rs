use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use n_mcconsole_event_bus::Ctx;
use n_mcconsole_event_bus::action::{Action, KeyFlow};
use n_mcconsole_event_bus::app::App;
use n_mcconsole_event_bus::outbox::Outbox;
use n_mcconsole_event_bus::registrar::Registrar;
use n_mcconsole_event_bus::scene::{Scene, SceneId};
use ratatui::Frame;
use ratatui::prelude::{Constraint, Layout, Line, Rect, Style};
use ratatui::widgets::Tabs;

pub struct Tab {
    pub title: String,
    pub sid: SceneId,
}

pub struct TabBuilder<'a> {
    tabs: Vec<Tab>,
    app: &'a mut App,
}

impl<'a> TabBuilder<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { tabs: vec![], app }
    }

    pub fn add_tab<S: Scene>(mut self, scene: S, title: impl Into<String>) -> Self {
        let sid = self.app.register_scene(scene);
        self.tabs.push(Tab {
            sid,
            title: title.into(),
        });
        self
    }

    pub fn build(self) -> Vec<Tab> {
        self.tabs
    }
}

pub struct TabbedScene {
    tabs: Vec<Tab>,
    active: usize,
}

impl TabbedScene {
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self { tabs, active: 0 }
    }

    fn split(area: Rect) -> [Rect; 2] {
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area)
    }

    fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active + 1) % self.tabs.len();
        }
    }

    fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active + self.tabs.len() - 1) % self.tabs.len();
        }
    }
}

impl Scene for TabbedScene {
    fn register(_registrar: &mut Registrar<Self>) {}

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &Ctx, _out: &mut Outbox) -> Action {
        if self.tabs.is_empty() {
            Action::Pop
        } else {
            Action::None
        }
    }

    fn view(&self, f: &mut Frame, area: Rect, _ctx: &Ctx) {
        let [bar, _body] = Self::split(area);
        let titles: Vec<Line> = self
            .tabs
            .iter()
            .map(|t| Line::raw(t.title.clone()))
            .collect();
        let widget = Tabs::new(titles)
            .select(self.active)
            .highlight_style(Style::new().bold().reversed())
            .divider(" │ ");
        f.render_widget(widget, bar);
    }

    fn intercept_key(&mut self, key: KeyEvent) -> KeyFlow {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (ctrl, key.code) {
            (true, KeyCode::Tab) => {
                self.next_tab();
                KeyFlow::Consumed
            }
            (true, KeyCode::BackTab) => {
                self.prev_tab();
                KeyFlow::Consumed
            }
            (true, KeyCode::Char(c)) if ('1'..='9').contains(&c) => {
                let idx = (c as u8 - b'1') as usize;
                if idx < self.tabs.len() {
                    self.active = idx;
                }
                KeyFlow::Consumed
            }
            _ => KeyFlow::Pass,
        }
    }

    fn active_child(&self) -> Option<SceneId> {
        self.tabs.get(self.active).map(|t| t.sid)
    }

    fn child_areas(&self, area: Rect) -> Vec<(SceneId, Rect)> {
        let [_bar, body] = Self::split(area);
        match self.tabs.get(self.active) {
            Some(t) => vec![(t.sid, body)],
            None => Vec::new(),
        }
    }

    fn child_ids(&self) -> Vec<SceneId> {
        self.tabs.iter().map(|t| t.sid).collect()
    }
}
