use crate::scene::SceneSpawner;
use n_mcconsole_core::command::Command;

pub enum Action {
    None,
    Pop,
    Quit,
    RunJob(Command),
    Push(SceneSpawner),
}

pub enum KeyFlow {
    Pass,
    Consumed,
}
