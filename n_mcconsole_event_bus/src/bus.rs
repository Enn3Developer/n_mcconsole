use crate::Thunk;
use crate::scene::SceneId;
use n_mcconsole_core::message::Envelope;
use std::any::TypeId;
use std::collections::{HashMap, VecDeque};

/// An event bus used to deliver messages
#[derive(Default)]
pub struct Bus {
    pub(crate) index: HashMap<TypeId, Vec<(SceneId, Thunk)>>,
    pub(crate) queue: VecDeque<Envelope>,
}

impl Bus {
    /// Subscribe a scene to a message type using the specified function to handle it
    pub fn subscribe(&mut self, tid: TypeId, id: SceneId, t: Thunk) {
        self.index.entry(tid).or_default().push((id, t));
    }

    /// Unsubscribe a scene
    pub fn unsubscribe(&mut self, id: SceneId) {
        for v in self.index.values_mut() {
            v.retain(|(s, _)| *s != id);
        }
    }
}
