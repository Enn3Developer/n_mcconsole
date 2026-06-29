use crate::scene::SceneSpawner;
use n_mcconsole_core::command::Command;

/// Additional information you can pass onto the central app manager
pub enum Action {
    /// No action
    None,
    /// Close this scene
    Pop,
    /// Quit the app
    Quit,
    /// Run this command
    RunJob(Command),
    /// Register this scene and push it onto the stack to render it above this scene
    Push(SceneSpawner),
}

/// Used by the central app manager to know whether a key event was consumed or not
pub enum KeyFlow {
    Pass,
    Consumed,
}
